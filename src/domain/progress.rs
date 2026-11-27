use crate::domain::commands::MaintenanceAction;

/// アクションが標準出力からの確定進捗（0〜100%）解析に対応しているかを返す。
pub fn supports_determinate_progress(action: &MaintenanceAction) -> bool {
    matches!(
        action,
        MaintenanceAction::DismRestoreHealth
            | MaintenanceAction::DismRestoreHealthWithSource { .. }
            | MaintenanceAction::SfcScannow
            | MaintenanceAction::ChkdskRepair { .. }
    )
}

/// 実行コマンドの出力1行から進捗率（0〜100）を抽出する。
pub fn extract_progress_percent(action: &MaintenanceAction, line: &str) -> Option<u8> {
    match action {
        MaintenanceAction::DismRestoreHealth
        | MaintenanceAction::DismRestoreHealthWithSource { .. } => parse_dism_progress(line),
        MaintenanceAction::SfcScannow => parse_sfc_progress(line),
        MaintenanceAction::ChkdskRepair { .. } => parse_chkdsk_progress(line),
        MaintenanceAction::NtpResync
        | MaintenanceAction::SystemInfo
        | MaintenanceAction::NetFullRepair => None,
    }
}

fn clamp_percent_f64(val: f64) -> Option<u8> {
    if !val.is_finite() || !(0.0..=100.0).contains(&val) {
        return None;
    }
    Some((val.round() as i64).clamp(0, 100) as u8)
}

fn parse_percent_token_before_unit(text: &str, unit: &str) -> Option<u8> {
    let mut search_start = 0;
    while let Some(rel_pos) = text[search_start..].find(unit) {
        let unit_pos = search_start + rel_pos;
        let before = text[..unit_pos].trim_end();
        let num_start = before
            .char_indices()
            .rev()
            .take_while(|&(_, ch)| ch.is_ascii_digit() || ch == '.')
            .last()
            .map(|(idx, _)| idx);

        if let Some(start) = num_start {
            let prev_ok = start == 0
                || !before[..start]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_alphanumeric());
            let num_str = &before[start..];
            if prev_ok
                && num_str.chars().any(|c| c.is_ascii_digit())
                && num_str.matches('.').count() <= 1
                && let Ok(val) = num_str.parse::<f64>()
                && let Some(pct) = clamp_percent_f64(val)
            {
                return Some(pct);
            }
        }
        search_start = unit_pos + unit.len();
    }
    None
}

fn parse_dism_progress(line: &str) -> Option<u8> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('[')?.strip_suffix(']')?;
    if !inner.contains('%') {
        return None;
    }
    if !inner
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '=' | ' ' | '.' | '%'))
    {
        return None;
    }
    parse_percent_token_before_unit(inner, "%")
}

fn parse_sfc_progress(line: &str) -> Option<u8> {
    let lower = line.to_ascii_lowercase();
    let is_sfc_progress = (line.contains("検証") && line.contains("完了"))
        || (lower.contains("verification") && lower.contains("complete"));
    if !is_sfc_progress {
        return None;
    }
    parse_percent_token_before_unit(line, "%")
}

fn parse_chkdsk_progress(line: &str) -> Option<u8> {
    let lower = line.to_ascii_lowercase();

    // ステージ進捗と合計進捗が併記される行では合計進捗を優先する
    for marker in ["合計:", "合計：", "total:"] {
        if let Some(idx) = lower.find(marker) {
            let tail = &line[idx + marker.len()..];
            if let Some(pct) = parse_percent_token_before_unit(tail, "%") {
                return Some(pct);
            }
        }
    }

    let has_keyword = line.contains("完了")
        || line.contains("パーセント")
        || line.contains("ステージ")
        || lower.contains("complete")
        || lower.contains("percent")
        || lower.contains("stage");
    if !has_keyword {
        return None;
    }

    parse_percent_token_before_unit(line, "%")
        .or_else(|| parse_percent_token_before_unit(line, "パーセント"))
        .or_else(|| parse_percent_token_before_unit(&lower, "percent"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports_determinate_progress() {
        assert!(supports_determinate_progress(
            &MaintenanceAction::DismRestoreHealth
        ));
        assert!(supports_determinate_progress(
            &MaintenanceAction::DismRestoreHealthWithSource {
                source: "C:\\src".to_string(),
                index: 1,
            }
        ));
        assert!(supports_determinate_progress(
            &MaintenanceAction::SfcScannow
        ));
        assert!(supports_determinate_progress(
            &MaintenanceAction::ChkdskRepair {
                drive: "C:".to_string(),
            }
        ));
        assert!(!supports_determinate_progress(
            &MaintenanceAction::NtpResync
        ));
        assert!(!supports_determinate_progress(
            &MaintenanceAction::SystemInfo
        ));
        assert!(!supports_determinate_progress(
            &MaintenanceAction::NetFullRepair
        ));
    }

    #[test]
    fn test_parse_dism_progress() {
        let act = MaintenanceAction::DismRestoreHealth;
        assert_eq!(
            extract_progress_percent(
                &act,
                "[=                          2.0%                           ]"
            ),
            Some(2)
        );
        assert_eq!(
            extract_progress_percent(
                &act,
                "[===============            25.6%                          ]"
            ),
            Some(26)
        );
        assert_eq!(
            extract_progress_percent(
                &act,
                "[==========================100.0%==========================]"
            ),
            Some(100)
        );
        assert_eq!(extract_progress_percent(&act, "[   0.0%   ]"), Some(0));
        assert_eq!(
            extract_progress_percent(&act, "[情報] 50% ではありません"),
            None
        );
        assert_eq!(
            extract_progress_percent(&act, "操作は正常に完了しました。"),
            None
        );
    }

    #[test]
    fn test_parse_sfc_progress() {
        let act = MaintenanceAction::SfcScannow;
        assert_eq!(
            extract_progress_percent(&act, "検証 0% が完了しました。"),
            Some(0)
        );
        assert_eq!(
            extract_progress_percent(&act, "検証 45% が完了しました。"),
            Some(45)
        );
        assert_eq!(
            extract_progress_percent(&act, "Verification 100% complete."),
            Some(100)
        );
        assert_eq!(
            extract_progress_percent(&act, "システム スキャンを開始しています。"),
            None
        );
        assert_eq!(
            extract_progress_percent(&act, "検証 150% が完了しました。"),
            None
        );
    }

    #[test]
    fn test_parse_chkdsk_progress() {
        let act = MaintenanceAction::ChkdskRepair {
            drive: "C:".to_string(),
        };
        assert_eq!(
            extract_progress_percent(
                &act,
                "進捗状況: 合計 100 中 25 が完了。ステージ: 50% 、合計: 25%"
            ),
            Some(25)
        );
        assert_eq!(
            extract_progress_percent(&act, "Stage: 80%, Total: 40%"),
            Some(40)
        );
        assert_eq!(
            extract_progress_percent(&act, "42 パーセント完了しました。"),
            Some(42)
        );
        assert_eq!(
            extract_progress_percent(&act, "67 percent completed."),
            Some(67)
        );
        assert_eq!(extract_progress_percent(&act, "90% 完了"), Some(90));
        assert_eq!(
            extract_progress_percent(
                &act,
                "ステージ 1: 基本ファイル システム構造を検査しています..."
            ),
            None
        );
    }

    #[test]
    fn test_non_progress_actions_return_none() {
        assert_eq!(
            extract_progress_percent(&MaintenanceAction::SystemInfo, "メモリ使用率: 50%"),
            None
        );
        assert_eq!(
            extract_progress_percent(&MaintenanceAction::NetFullRepair, "100%"),
            None
        );
    }
}
