/// DISM `/Get-ImageInfo` 出力から抽出したイメージ情報。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DismImageInfo {
    pub name: Option<String>,
    pub architecture: Option<String>,
    pub version: Option<String>,
    pub service_pack_build: Option<String>,
    pub edition: Option<String>,
}

impl DismImageInfo {
    /// クライアント OS かどうか（`Server` を含まないか）を判定し、
    /// ビルド番号から分かりやすい Windows バージョン表記（`25H2` 等）を返す。
    pub fn friendly_version(&self) -> Option<&'static str> {
        let is_server = self
            .name
            .as_deref()
            .is_some_and(|n| n.to_ascii_lowercase().contains("server"))
            || self
                .edition
                .as_deref()
                .is_some_and(|e| e.to_ascii_lowercase().contains("server"));
        if is_server {
            return None;
        }
        let build = self.build_number()?;
        map_build_to_friendly_version(build)
    }

    /// `version` フィールド（例: `10.0.26300` や `10.0.26300.9711`）からビルド番号を抽出する。
    pub fn build_number(&self) -> Option<u32> {
        let ver = self.version.as_deref()?;
        let mut parts = ver.split('.');
        let _major = parts.next()?;
        let _minor = parts.next()?;
        let build_str = parts.next()?.trim();
        build_str.parse::<u32>().ok()
    }

    /// ビルド番号と Service Pack ビルド番号を組み合わせた表示用ビルド文字列を返す。
    pub fn display_build(&self) -> Option<String> {
        let ver = self.version.as_deref()?;
        let parts: Vec<&str> = ver.split('.').collect();
        if parts.len() >= 4 {
            return Some(format!("{}.{}", parts[2].trim(), parts[3].trim()));
        }
        if let Some(build) = self.build_number() {
            if let Some(sp) = self
                .service_pack_build
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                return Some(format!("{build}.{sp}"));
            }
            return Some(build.to_string());
        }
        None
    }

    /// UI の `[要約]` 行および UIA 読み上げ用の要約文字列を生成する。
    pub fn format_summary(&self) -> Option<String> {
        let title = self
            .name
            .as_deref()
            .or(self.edition.as_deref())
            .map(str::trim)
            .filter(|s| !s.is_empty());

        let has_version = self
            .version
            .as_deref()
            .is_some_and(|v| !v.trim().is_empty());
        if title.is_none() && !has_version {
            return None;
        }

        let name_part = title.unwrap_or("Windows イメージ");
        let arch = self
            .architecture
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());

        let friendly = self.friendly_version();
        let build_disp = self.display_build();
        let raw_ver = self
            .version
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());

        let ver_part = match (friendly, build_disp, raw_ver, arch) {
            (Some(fv), Some(bd), _, Some(ar)) => format!("バージョン: {fv} (ビルド {bd} / {ar})"),
            (Some(fv), Some(bd), _, None) => format!("バージョン: {fv} (ビルド {bd})"),
            (Some(fv), None, _, Some(ar)) => format!("バージョン: {fv} ({ar})"),
            (Some(fv), None, _, None) => format!("バージョン: {fv}"),
            (None, Some(bd), Some(rv), Some(ar)) => {
                format!("バージョン: {rv} (ビルド {bd} / {ar})")
            }
            (None, Some(bd), Some(rv), None) => format!("バージョン: {rv} (ビルド {bd})"),
            (None, Some(bd), None, Some(ar)) => format!("ビルド {bd} ({ar})"),
            (None, Some(bd), None, None) => format!("ビルド {bd}"),
            (None, None, Some(rv), Some(ar)) => format!("バージョン: {rv} ({ar})"),
            (None, None, Some(rv), None) => format!("バージョン: {rv}"),
            (None, None, None, Some(ar)) => format!("アーキテクチャ: {ar}"),
            (None, None, None, None) => return Some(name_part.to_string()),
        };

        Some(format!("{name_part} / {ver_part}"))
    }
}

/// Windows 10 / 11 クライアントビルド番号を分かりやすい機能更新バージョン表記へ変換する。
pub fn map_build_to_friendly_version(build: u32) -> Option<&'static str> {
    match build {
        // Windows 11
        22000 => Some("21H2"),
        22621 => Some("22H2 (または 23H2 ベース)"),
        22631 => Some("23H2"),
        26100 => Some("24H2 (または 25H2 ベース)"),
        26200 => Some("25H2"),
        26300 => Some("26H2"),
        28000 => Some("26H1"),
        // Windows 10
        10240 => Some("1507"),
        10586 => Some("1511"),
        14393 => Some("1607"),
        15063 => Some("1703"),
        16299 => Some("1709"),
        17134 => Some("1803"),
        17763 => Some("1809"),
        18362 => Some("1903"),
        18363 => Some("1909"),
        19041 => Some("2004 (または 20H2〜22H2 ベース)"),
        19042 => Some("20H2"),
        19043 => Some("21H1"),
        19044 => Some("21H2"),
        19045 => Some("22H2"),
        _ => None,
    }
}

/// DISM `/Get-ImageInfo` の出力行スライスを解析して `DismImageInfo` を返す。
pub fn parse_dism_image_info<S: AsRef<str>>(lines: &[S]) -> DismImageInfo {
    let mut info = DismImageInfo::default();

    for line in lines {
        let trimmed = line.as_ref().trim();
        let Some((raw_key, raw_val)) = trimmed.split_once(':').or_else(|| trimmed.split_once('：'))
        else {
            continue;
        };
        let key = raw_key.trim();
        let val = raw_val.trim();
        if val.is_empty() {
            continue;
        }

        // ヘッダー行の DISM ツール自体のバージョンを除外するため、
        // 「インデックス / Index」が出現した時点でそれ以前の一時値をリセットする。
        if key == "インデックス" || key.eq_ignore_ascii_case("Index") {
            info = DismImageInfo::default();
            continue;
        }

        if key == "名前" || key.eq_ignore_ascii_case("Name") {
            info.name = Some(val.to_string());
        } else if key == "アーキテクチャ" || key.eq_ignore_ascii_case("Architecture") {
            info.architecture = Some(val.to_string());
        } else if key == "バージョン" || key.eq_ignore_ascii_case("Version") {
            info.version = Some(val.to_string());
        } else if key == "Service Pack のビルド" || key.eq_ignore_ascii_case("Service Pack Build")
        {
            info.service_pack_build = Some(val.to_string());
        } else if key == "エディション" || key.eq_ignore_ascii_case("Edition") {
            info.edition = Some(val.to_string());
        }
    }

    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_japanese_dism_output_ignores_tool_header_version() {
        let lines = [
            "展開イメージのサービスと管理ツール",
            "バージョン: 10.0.26100.1150",
            "イメージの詳細: E:\\sources\\install.wim",
            "インデックス: 1",
            "名前: Windows 11 Pro",
            "アーキテクチャ: x64",
            "バージョン: 10.0.26300",
            "Service Pack のビルド: 9711",
            "エディション: Professional",
        ];
        let info = parse_dism_image_info(&lines);
        assert_eq!(info.name.as_deref(), Some("Windows 11 Pro"));
        assert_eq!(info.architecture.as_deref(), Some("x64"));
        assert_eq!(info.version.as_deref(), Some("10.0.26300"));
        assert_eq!(info.service_pack_build.as_deref(), Some("9711"));
        assert_eq!(info.edition.as_deref(), Some("Professional"));
        assert_eq!(
            info.format_summary().as_deref(),
            Some("Windows 11 Pro / バージョン: 26H2 (ビルド 26300.9711 / x64)")
        );
    }

    #[test]
    fn test_parse_english_dism_output() {
        let lines = [
            "Deployment Image Servicing and Management tool",
            "Version: 10.0.22621.1",
            "Index : 2",
            "Name : Windows 11 Home",
            "Architecture : x64",
            "Version : 10.0.26200",
            "Service Pack Build : 550",
            "Edition : Core",
        ];
        let info = parse_dism_image_info(&lines);
        assert_eq!(info.name.as_deref(), Some("Windows 11 Home"));
        assert_eq!(
            info.format_summary().as_deref(),
            Some("Windows 11 Home / バージョン: 25H2 (ビルド 26200.550 / x64)")
        );
    }

    #[test]
    fn test_build_mapping_win10_and_win11() {
        assert_eq!(map_build_to_friendly_version(22000), Some("21H2"));
        assert_eq!(
            map_build_to_friendly_version(22621),
            Some("22H2 (または 23H2 ベース)")
        );
        assert_eq!(map_build_to_friendly_version(22631), Some("23H2"));
        assert_eq!(
            map_build_to_friendly_version(26100),
            Some("24H2 (または 25H2 ベース)")
        );
        assert_eq!(map_build_to_friendly_version(26200), Some("25H2"));
        assert_eq!(map_build_to_friendly_version(26300), Some("26H2"));
        assert_eq!(map_build_to_friendly_version(28000), Some("26H1"));
        assert_eq!(map_build_to_friendly_version(19045), Some("22H2"));
        assert_eq!(
            map_build_to_friendly_version(19041),
            Some("2004 (または 20H2〜22H2 ベース)")
        );
        assert_eq!(map_build_to_friendly_version(10240), Some("1507"));
        assert_eq!(map_build_to_friendly_version(30000), None);
    }

    #[test]
    fn test_server_and_unknown_future_build_do_not_mislabel() {
        let server_lines = [
            "Index : 1",
            "Name : Windows Server 2025 Standard",
            "Architecture : x64",
            "Version : 10.0.26100",
            "Service Pack Build : 1742",
            "Edition : ServerStandard",
        ];
        let server_info = parse_dism_image_info(&server_lines);
        assert_eq!(server_info.friendly_version(), None);
        let server_summary = server_info.format_summary().unwrap();
        assert!(!server_summary.contains("24H2"));
        assert!(server_summary.contains("10.0.26100"));
        assert!(server_summary.contains("26100.1742"));

        let future_lines = [
            "インデックス: 1",
            "名前: Windows Next",
            "アーキテクチャ: arm64",
            "バージョン: 10.0.31000",
            "Service Pack のビルド: 100",
        ];
        let future_info = parse_dism_image_info(&future_lines);
        assert_eq!(future_info.friendly_version(), None);
        assert_eq!(
            future_info.format_summary().as_deref(),
            Some("Windows Next / バージョン: 10.0.31000 (ビルド 31000.100 / arm64)")
        );
    }
}
