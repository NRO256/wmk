use std::path::{Path, PathBuf};

/// EXEと同階層（または `sources` サブフォルダ）からポータブル修復イメージを検出する。
pub fn detect_portable_image_in_dir(dir: &Path) -> Option<PathBuf> {
    let candidates = [
        dir.join("install.wim"),
        dir.join("install.esd"),
        dir.join("sources").join("install.wim"),
        dir.join("sources").join("install.esd"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

/// 末尾のバックスラッシュ（ドライブルート `C:\` を除く）を除去する。
pub fn trim_trailing_backslash(path: &str) -> String {
    let trimmed = path.trim().trim_matches('"');
    let bytes = trimmed.as_bytes();
    if bytes.len() == 3 && bytes[1] == b':' && (bytes[2] == b'\\' || bytes[2] == b'/') {
        return trimmed.to_string();
    }
    trimmed.trim_end_matches(['\\', '/']).to_string()
}

/// 危険な制御文字・引用符が含まれていないか検証する。
pub fn validate_safe_path_chars(raw: &str) -> Result<(), String> {
    if raw.is_empty() {
        return Err("パスが空です。".to_string());
    }
    if raw
        .chars()
        .any(|c| c == '"' || c == '\r' || c == '\n' || c == '\0')
    {
        return Err("パスに使用できない文字（引用符や制御文字）が含まれています。".to_string());
    }
    Ok(())
}

/// DISM `/Source:` に渡す値（フォルダ、または `WIM:<path>:<idx>` / `ESD:<path>:<idx>`）を構築・検証する。
pub fn format_dism_source_value(raw_input: &str, index: u32) -> Result<String, String> {
    let trimmed = raw_input.trim().trim_matches('"');
    validate_safe_path_chars(trimmed)?;

    let idx = if index == 0 { 1 } else { index };
    let upper = trimmed.to_ascii_uppercase();

    if upper.starts_with("WIM:") || upper.starts_with("ESD:") {
        let prefix = &upper[..4];
        let rest = &trimmed[4..];
        if rest.is_empty() {
            return Err("イメージファイルのパスが指定されていません。".to_string());
        }
        // 末尾に `:数字` が既に付いているか確認
        if let Some(colon_pos) = rest.rfind(':') {
            let suffix = &rest[colon_pos + 1..];
            // ドライブレターのコロン（例: C:\...）ではないことを確認
            if colon_pos > 1 && !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()) {
                return Ok(format!("{prefix}{rest}"));
            }
        }
        return Ok(format!("{prefix}{rest}:{idx}"));
    }

    let normalized = trim_trailing_backslash(trimmed);
    let lower = normalized.to_ascii_lowercase();
    if lower.ends_with(".wim") {
        Ok(format!("WIM:{normalized}:{idx}"))
    } else if lower.ends_with(".esd") {
        Ok(format!("ESD:{normalized}:{idx}"))
    } else {
        Ok(normalized)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_wim_and_esd_paths() {
        assert_eq!(
            format_dism_source_value(r"C:\Tools\install.wim", 1).unwrap(),
            r"WIM:C:\Tools\install.wim:1"
        );
        assert_eq!(
            format_dism_source_value(r"D:\My Images\install.esd", 3).unwrap(),
            r"ESD:D:\My Images\install.esd:3"
        );
    }

    #[test]
    fn test_format_prefixed_source_preserves_or_appends_index() {
        assert_eq!(
            format_dism_source_value(r"WIM:E:\sources\install.wim:2", 1).unwrap(),
            r"WIM:E:\sources\install.wim:2"
        );
        assert_eq!(
            format_dism_source_value(r"esd:E:\sources\install.esd", 4).unwrap(),
            r"ESD:E:\sources\install.esd:4"
        );
    }

    #[test]
    fn test_format_folder_trims_trailing_slash() {
        assert_eq!(
            format_dism_source_value(r"E:\sources\sxs\", 1).unwrap(),
            r"E:\sources\sxs"
        );
        assert_eq!(format_dism_source_value(r"E:\", 1).unwrap(), r"E:\");
    }

    #[test]
    fn test_rejects_quotes_and_empty() {
        assert!(format_dism_source_value("", 1).is_err());
        assert!(format_dism_source_value(r#"C:\bad"path\install.wim"#, 1).is_err());
    }

    #[test]
    fn test_detect_portable_image_in_dir() {
        let base = std::env::temp_dir().join(format!("wmk_test_{}", std::process::id()));
        let sources = base.join("sources");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&sources).unwrap();

        assert!(detect_portable_image_in_dir(&base).is_none());

        let sub_wim = sources.join("install.wim");
        std::fs::write(&sub_wim, b"dummy").unwrap();
        assert_eq!(detect_portable_image_in_dir(&base), Some(sub_wim));

        let root_esd = base.join("install.esd");
        std::fs::write(&root_esd, b"dummy").unwrap();
        assert_eq!(detect_portable_image_in_dir(&base), Some(root_esd));

        let root_wim = base.join("install.wim");
        std::fs::write(&root_wim, b"dummy").unwrap();
        assert_eq!(detect_portable_image_in_dir(&base), Some(root_wim));

        let _ = std::fs::remove_dir_all(&base);
    }
}
