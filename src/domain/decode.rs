#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextEncodingState {
    Auto,
    Utf8,
    Cp932,
    Utf16Le,
}

/// `\r` と `\n` を区切りとして行単位で文字列を切り出し、UTF-8 / CP932 / UTF-16LE を自動判定するデコーダ。
pub struct StreamLineDecoder<F>
where
    F: Fn(&[u8]) -> String,
{
    state: TextEncodingState,
    buffer: Vec<u8>,
    utf16_pending: Option<u8>,
    cp932_decoder: F,
}

impl<F> StreamLineDecoder<F>
where
    F: Fn(&[u8]) -> String,
{
    pub fn new(cp932_decoder: F) -> Self {
        Self {
            state: TextEncodingState::Auto,
            buffer: Vec::new(),
            utf16_pending: None,
            cp932_decoder,
        }
    }

    #[cfg(test)]
    pub fn state(&self) -> TextEncodingState {
        self.state
    }

    /// 受信したバイト列を投入し、確定した行のベクタを返す。
    pub fn push_bytes(&mut self, chunk: &[u8]) -> Vec<String> {
        if chunk.is_empty() {
            return Vec::new();
        }

        if self.state == TextEncodingState::Auto && Self::looks_like_utf16le(&self.buffer, chunk) {
            self.state = TextEncodingState::Utf16Le;
            if self.buffer.is_empty() {
                return self.push_utf16le(chunk);
            }
            let mut combined = std::mem::take(&mut self.buffer);
            combined.extend_from_slice(chunk);
            return self.push_utf16le(&combined);
        }

        if self.state == TextEncodingState::Utf16Le {
            return self.push_utf16le(chunk);
        }

        let mut lines = Vec::new();
        for &b in chunk {
            if b == b'\r' || b == b'\n' {
                if !self.buffer.is_empty() {
                    let line =
                        Self::decode_line_bytes(&mut self.state, &self.cp932_decoder, &self.buffer);
                    self.buffer.clear();
                    lines.push(line);
                }
            } else {
                self.buffer.push(b);
                if self.buffer.len() >= 65_536 {
                    let line =
                        Self::decode_line_bytes(&mut self.state, &self.cp932_decoder, &self.buffer);
                    self.buffer.clear();
                    lines.push(line);
                }
            }
        }
        lines
    }

    /// ストリーム終了時に残ったバッファをフラッシュする。
    pub fn finish(&mut self) -> Option<String> {
        if self.state == TextEncodingState::Utf16Le {
            self.utf16_pending = None;
            if self.buffer.len() < 2 {
                self.buffer.clear();
                return None;
            }
            let line = Self::decode_utf16le_units(&self.buffer);
            self.buffer.clear();
            return if line.is_empty() { None } else { Some(line) };
        }

        if self.buffer.is_empty() {
            return None;
        }
        let line = Self::decode_line_bytes(&mut self.state, &self.cp932_decoder, &self.buffer);
        self.buffer.clear();
        Some(line)
    }

    fn looks_like_utf16le(prefix: &[u8], chunk: &[u8]) -> bool {
        if prefix.is_empty() && chunk.len() >= 2 && chunk[0] == 0xFF && chunk[1] == 0xFE {
            return true;
        }
        if prefix.len() == 1 && prefix[0] == 0xFF && !chunk.is_empty() && chunk[0] == 0xFE {
            return true;
        }
        let total_len = prefix.len() + chunk.len();
        if total_len < 2 || !total_len.is_multiple_of(2) {
            return false;
        }

        let mut iter = prefix.iter().copied().chain(chunk.iter().copied());
        let mut ascii_utf16le_pairs = 0usize;
        let mut even_nuls = 0usize;
        while let (Some(lo), Some(hi)) = (iter.next(), iter.next()) {
            if lo == 0 {
                even_nuls += 1;
            } else if hi == 0 {
                ascii_utf16le_pairs += 1;
            }
        }

        if total_len == 2 {
            ascii_utf16le_pairs == 1
        } else {
            ascii_utf16le_pairs >= 2 && ascii_utf16le_pairs > even_nuls
        }
    }

    fn push_utf16le(&mut self, chunk: &[u8]) -> Vec<String> {
        let mut combined = Vec::with_capacity(chunk.len() + 1);
        if let Some(prev) = self.utf16_pending.take() {
            combined.push(prev);
        }
        combined.extend_from_slice(chunk);

        let usable_len = (combined.len() / 2) * 2;
        if usable_len < combined.len() {
            self.utf16_pending = Some(combined[usable_len]);
        }

        let mut lines = Vec::new();
        let mut idx = 0;
        if self.buffer.is_empty() && usable_len >= 2 && combined[0] == 0xFF && combined[1] == 0xFE {
            idx = 2;
        }

        while idx + 1 < usable_len {
            let unit = u16::from_le_bytes([combined[idx], combined[idx + 1]]);
            if unit == 0x000D || unit == 0x000A {
                if !self.buffer.is_empty() {
                    let line = Self::decode_utf16le_units(&self.buffer);
                    self.buffer.clear();
                    if !line.is_empty() {
                        lines.push(line);
                    }
                }
            } else {
                self.buffer.push(combined[idx]);
                self.buffer.push(combined[idx + 1]);
                if self.buffer.len() >= 65_536 {
                    let line = Self::decode_utf16le_units(&self.buffer);
                    self.buffer.clear();
                    if !line.is_empty() {
                        lines.push(line);
                    }
                }
            }
            idx += 2;
        }
        lines
    }

    fn decode_utf16le_units(bytes: &[u8]) -> String {
        let units = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&pair| u16::from_le_bytes(pair))
            .filter(|&u| u != 0 && u != 0xFEFF);
        char::decode_utf16(units)
            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect()
    }

    fn decode_line_bytes(state: &mut TextEncodingState, cp932_decoder: &F, bytes: &[u8]) -> String {
        let has_non_ascii = bytes.iter().any(|&b| b >= 0x80);
        if !has_non_ascii {
            return String::from_utf8_lossy(bytes).into_owned();
        }

        match *state {
            TextEncodingState::Cp932 => cp932_decoder(bytes),
            TextEncodingState::Utf8 => String::from_utf8_lossy(bytes).into_owned(),
            TextEncodingState::Utf16Le => Self::decode_utf16le_units(bytes),
            TextEncodingState::Auto => match std::str::from_utf8(bytes) {
                Ok(s) => {
                    *state = TextEncodingState::Utf8;
                    s.to_string()
                }
                Err(_) => {
                    *state = TextEncodingState::Cp932;
                    cp932_decoder(bytes)
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_cp932(bytes: &[u8]) -> String {
        format!("[CP932:{}bytes]", bytes.len())
    }

    #[test]
    fn test_ascii_does_not_lock_encoding_and_splits_cr_lf() {
        let mut dec = StreamLineDecoder::new(mock_cp932);
        let lines = dec.push_bytes(b"Progress 10%\rProgress 20%\r\nDone");
        assert_eq!(lines, vec!["Progress 10%", "Progress 20%"]);
        assert_eq!(dec.state(), TextEncodingState::Auto);
        assert_eq!(dec.finish(), Some("Done".to_string()));
    }

    #[test]
    fn test_cp932_latches_on_invalid_utf8() {
        let mut dec = StreamLineDecoder::new(mock_cp932);
        let lines = dec.push_bytes(&[b'A', b'B', b'\n', 0x83, 0x65, b'\n']);
        assert_eq!(lines[0], "AB");
        assert_eq!(lines[1], "[CP932:2bytes]");
        assert_eq!(dec.state(), TextEncodingState::Cp932);

        let next = dec.push_bytes("あ\n".as_bytes());
        assert_eq!(next[0], "[CP932:3bytes]");
    }

    #[test]
    fn test_utf8_latches_on_valid_non_ascii_utf8() {
        let mut dec = StreamLineDecoder::new(mock_cp932);
        let lines = dec.push_bytes("準備中...\r\n".as_bytes());
        assert_eq!(lines, vec!["準備中..."]);
        assert_eq!(dec.state(), TextEncodingState::Utf8);
    }

    #[test]
    fn test_utf16le_with_split_chunks() {
        let mut dec = StreamLineDecoder::new(mock_cp932);
        let text = "検証 100%\r\n完了";
        let mut utf16_bytes: Vec<u8> = Vec::new();
        for unit in text.encode_utf16() {
            utf16_bytes.extend_from_slice(&unit.to_le_bytes());
        }
        // 奇数バイト境界で分割して投入
        let mid = 5;
        let mut lines = dec.push_bytes(&utf16_bytes[..mid]);
        lines.extend(dec.push_bytes(&utf16_bytes[mid..]));
        assert_eq!(dec.state(), TextEncodingState::Utf16Le);
        assert_eq!(lines, vec!["検証 100%"]);
        assert_eq!(dec.finish(), Some("完了".to_string()));
    }

    #[test]
    fn test_stray_nul_does_not_latch_utf16le() {
        let mut dec = StreamLineDecoder::new(mock_cp932);
        // 奇数長や単発NULを含むUTF-8行はUTF-16LEに誤判定しない
        let lines = dec.push_bytes(b"abc\0\r\n");
        assert_eq!(lines, vec!["abc\0"]);
        assert_eq!(dec.state(), TextEncodingState::Auto);

        let utf8_lines = dec.push_bytes("正常\n".as_bytes());
        assert_eq!(utf8_lines, vec!["正常"]);
        assert_eq!(dec.state(), TextEncodingState::Utf8);
    }

    #[test]
    fn test_utf16le_flushes_at_65536_bytes_and_finish_ignores_bom_or_nul_only() {
        let mut dec = StreamLineDecoder::new(mock_cp932);
        let mut payload = vec![0xFF, 0xFE];
        for _ in 0..32_768 {
            payload.extend_from_slice(&[b'A', 0x00]);
        }
        let lines = dec.push_bytes(&payload);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].len(), 32_768);
        assert_eq!(dec.finish(), None);

        // BOM のみ、NUL のみの状態で finish() しても Some("") を返さない
        let mut dec_empty = StreamLineDecoder::new(mock_cp932);
        assert!(dec_empty.push_bytes(&[0xFF, 0xFE, 0x00, 0x00]).is_empty());
        assert_eq!(dec_empty.finish(), None);
    }
}
