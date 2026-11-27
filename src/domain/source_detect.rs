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

/// `WIM:` / `ESD:` プレフィックスおよび末尾の `:数字` インデックス指定を取り除く。
fn strip_image_prefix_and_index(
    trimmed: &str,
) -> Result<(&'static str, &str, Option<u32>), String> {
    let upper = trimmed.to_ascii_uppercase();
    let prefix = if upper.starts_with("WIM:") {
        "WIM:"
    } else if upper.starts_with("ESD:") {
        "ESD:"
    } else {
        return Ok(("", trimmed, None));
    };

    let rest = &trimmed[4..];
    if rest.is_empty() {
        return Err("イメージファイルのパスが指定されていません。".to_string());
    }

    if let Some(colon_pos) = rest.rfind(':') {
        let rest_bytes = rest.as_bytes();
        let is_drive_colon = colon_pos == 1
            && rest_bytes[0].is_ascii_alphabetic()
            && matches!(rest_bytes.get(2), Some(b'\\') | Some(b'/'));
        let suffix = &rest[colon_pos + 1..];
        if !is_drive_colon
            && colon_pos > 0
            && !suffix.is_empty()
            && suffix.chars().all(|c| c.is_ascii_digit())
        {
            let parsed: u32 = suffix.parse().unwrap_or(0);
            return Ok((prefix, &rest[..colon_pos], Some(parsed)));
        }
    }

    Ok((prefix, rest, None))
}

/// DISM `/Get-ImageInfo /ImageFile:<パス>` に渡す実イメージファイルパスを解決・検証する。
pub fn resolve_image_file_path(raw_input: &str) -> Result<String, String> {
    let trimmed = raw_input.trim().trim_matches('"');
    validate_safe_path_chars(trimmed)?;

    let (_, stripped, _) = strip_image_prefix_and_index(trimmed)?;
    let normalized = trim_trailing_backslash(stripped);
    let lower = normalized.to_ascii_lowercase();
    let path = Path::new(&normalized);

    if (lower.ends_with(".wim") || lower.ends_with(".esd")) && !path.is_dir() {
        return Ok(normalized);
    }

    if let Some(found) = detect_portable_image_in_dir(path) {
        return Ok(found.to_string_lossy().into_owned());
    }

    Err(format!(
        "指定された場所から install.wim または install.esd が見つかりません: {normalized}"
    ))
}

/// DISM `/Source:` に渡す値（フォルダ、または `WIM:<path>:<idx>` / `ESD:<path>:<idx>`）を構築・検証する。
pub fn format_dism_source_value(raw_input: &str, index: u32) -> Result<String, String> {
    let trimmed = raw_input.trim().trim_matches('"');
    validate_safe_path_chars(trimmed)?;

    let idx = if index == 0 { 1 } else { index };
    let (prefix, stripped, explicit_idx) = strip_image_prefix_and_index(trimmed)?;
    if !prefix.is_empty() {
        let final_idx = match explicit_idx {
            Some(0) => 1,
            Some(n) => n,
            None => idx,
        };
        return Ok(format!("{prefix}{stripped}:{final_idx}"));
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
        assert_eq!(format_dism_source_value("WIM:a:1", 2).unwrap(), "WIM:a:1");
        assert_eq!(
            format_dism_source_value(r"WIM:C:\123", 2).unwrap(),
            r"WIM:C:\123:2"
        );
        assert_eq!(
            format_dism_source_value(r"WIM:E:\sources\install.wim:0", 3).unwrap(),
            r"WIM:E:\sources\install.wim:1"
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

    #[test]
    fn test_resolve_image_file_path() {
        assert_eq!(
            resolve_image_file_path(r"WIM:E:\sources\install.wim:2").unwrap(),
            r"E:\sources\install.wim"
        );
        assert_eq!(
            resolve_image_file_path(r"esd:D:\Images\install.esd").unwrap(),
            r"D:\Images\install.esd"
        );
        assert!(resolve_image_file_path("").is_err());
        assert!(resolve_image_file_path(r#"C:\bad"path\install.wim"#).is_err());

        let base = std::env::temp_dir().join(format!("wmk_resolve_test_{}", std::process::id()));
        let sources = base.join("sources");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&sources).unwrap();
        let img = sources.join("install.esd");
        std::fs::write(&img, b"dummy").unwrap();

        let resolved = resolve_image_file_path(&base.to_string_lossy()).unwrap();
        assert_eq!(PathBuf::from(resolved), img);

        let _ = std::fs::remove_dir_all(&base);
    }
}
