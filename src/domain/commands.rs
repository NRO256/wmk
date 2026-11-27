use crate::domain::source_detect::{format_dism_source_value, resolve_image_file_path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MaintenanceAction {
    DismRestoreHealth,
    DismRestoreHealthWithSource { source: String, index: u32 },
    DismImageInfo { source: String, index: u32 },
    SfcScannow,
    ChkdskRepair { drive: String },
    NtpResync,
    SystemInfo,
    NetFullRepair,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandStep {
    pub exe_name: &'static str,
    pub args: Vec<String>,
    pub stdin_response: Option<&'static str>,
    pub display_label: String,
}

impl MaintenanceAction {
    pub fn title(&self) -> &'static str {
        match self {
            Self::DismRestoreHealth => "DISM イメージ修復 (オンライン)",
            Self::DismRestoreHealthWithSource { .. } => "DISM イメージ修復 (修復ソース指定)",
            Self::DismImageInfo { .. } => "DISM イメージ情報取得",
            Self::SfcScannow => "システムファイルチェッカー (sfc /scannow)",
            Self::ChkdskRepair { .. } => "ディスクチェックと修復 (chkdsk /r)",
            Self::NtpResync => "NTP 時刻同期 (w32tm /resync)",
            Self::SystemInfo => "Windows バージョン・システム情報 (systeminfo)",
            Self::NetFullRepair => "ネットワーク一括修復",
        }
    }

    /// 破壊的・再起動を伴うコマンドの実行前確認メッセージを返す。
    pub fn confirmation_message(&self) -> Option<String> {
        match self {
            Self::ChkdskRepair { drive } => Some(format!(
                "ドライブ {drive} に対して chkdsk /r を実行します。\n\
                 使用中のドライブ（C: 等）の場合、次回のシステム再起動時にチェックを実行するよう自動予約（Y応答）します。\n\
                 処理には長時間かかる場合があります。実行してもよろしいですか？"
            )),
            Self::NetFullRepair => Some(
                "ネットワーク設定のリセット（Winsock・IP設定リセット・DNSキャッシュ削除）を実行します。\n\
                 反映にはPCの再起動が必要になる場合があります。実行してもよろしいですか？"
                    .to_string(),
            ),
            _ => None,
        }
    }
}

/// ドライブ指定（"C", "C:", "C:\" 等）を `"C:"` 形式へ正規化・検証する。
pub fn normalize_drive_letter(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    let first = trimmed
        .chars()
        .next()
        .ok_or_else(|| "ドライブが選択されていません。".to_string())?;
    if !first.is_ascii_alphabetic() {
        return Err("無効なドライブ文字です。".to_string());
    }
    let rest = &trimmed[first.len_utf8()..];
    if !rest.is_empty() && rest != ":" && rest != ":\\" && rest != ":/" {
        return Err("ドライブ指定は 'C:' の形式で入力してください。".to_string());
    }
    Ok(format!("{}:", first.to_ascii_uppercase()))
}

/// アクションから実際に起動するコマンドステップ列を構築する。
pub fn build_command_steps(action: &MaintenanceAction) -> Result<Vec<CommandStep>, String> {
    match action {
        MaintenanceAction::DismRestoreHealth => Ok(vec![CommandStep {
            exe_name: "dism.exe",
            args: vec![
                "/Online".to_string(),
                "/Cleanup-Image".to_string(),
                "/RestoreHealth".to_string(),
            ],
            stdin_response: None,
            display_label: "DISM.exe /Online /Cleanup-Image /RestoreHealth".to_string(),
        }]),
        MaintenanceAction::DismRestoreHealthWithSource { source, index } => {
            let formatted_source = format_dism_source_value(source, *index)?;
            let source_arg = format!("/Source:{formatted_source}");
            Ok(vec![CommandStep {
                exe_name: "dism.exe",
                args: vec![
                    "/Online".to_string(),
                    "/Cleanup-Image".to_string(),
                    "/RestoreHealth".to_string(),
                    source_arg.clone(),
                    "/LimitAccess".to_string(),
                ],
                stdin_response: None,
                display_label: format!(
                    "DISM.exe /Online /Cleanup-Image /RestoreHealth {source_arg} /LimitAccess"
                ),
            }])
        }
        MaintenanceAction::DismImageInfo { source, index } => {
            let image_path = resolve_image_file_path(source)?;
            let idx = if *index == 0 { 1 } else { *index };
            let image_arg = format!("/ImageFile:{image_path}");
            let index_arg = format!("/Index:{idx}");
            Ok(vec![CommandStep {
                exe_name: "dism.exe",
                args: vec![
                    "/Get-ImageInfo".to_string(),
                    image_arg.clone(),
                    index_arg.clone(),
                ],
                stdin_response: None,
                display_label: format!("DISM.exe /Get-ImageInfo {image_arg} {index_arg}"),
            }])
        }
        MaintenanceAction::SfcScannow => Ok(vec![CommandStep {
            exe_name: "sfc.exe",
            args: vec!["/scannow".to_string()],
            stdin_response: None,
            display_label: "sfc.exe /scannow".to_string(),
        }]),
        MaintenanceAction::ChkdskRepair { drive } => {
            let d = normalize_drive_letter(drive)?;
            Ok(vec![CommandStep {
                exe_name: "chkdsk.exe",
                args: vec![d.clone(), "/r".to_string()],
                stdin_response: Some("Y\r\n"),
                display_label: format!("chkdsk.exe {d} /r"),
            }])
        }
        MaintenanceAction::NtpResync => Ok(vec![CommandStep {
            exe_name: "w32tm.exe",
            args: vec!["/resync".to_string()],
            stdin_response: None,
            display_label: "w32tm.exe /resync".to_string(),
        }]),
        MaintenanceAction::SystemInfo => Ok(vec![CommandStep {
            exe_name: "systeminfo.exe",
            args: Vec::new(),
            stdin_response: None,
            display_label: "systeminfo.exe".to_string(),
        }]),
        MaintenanceAction::NetFullRepair => Ok(vec![
            CommandStep {
                exe_name: "netsh.exe",
                args: vec!["winsock".to_string(), "reset".to_string()],
                stdin_response: None,
                display_label: "netsh.exe winsock reset".to_string(),
            },
            CommandStep {
                exe_name: "netsh.exe",
                args: vec!["int".to_string(), "ip".to_string(), "reset".to_string()],
                stdin_response: None,
                display_label: "netsh.exe int ip reset".to_string(),
            },
            CommandStep {
                exe_name: "ipconfig.exe",
                args: vec!["/flushdns".to_string()],
                stdin_response: None,
                display_label: "ipconfig.exe /flushdns".to_string(),
            },
        ]),
    }
}

/// ログバッファが上限バイト数を超えた場合、行境界を優先して先頭からトリミングする。
pub fn trim_log_buffer(buf: &mut String, max_bytes: usize) {
    if buf.len() <= max_bytes {
        return;
    }
    let excess = buf.len() - max_bytes;
    let mut cut_pos = excess;
    while cut_pos < buf.len() && !buf.is_char_boundary(cut_pos) {
        cut_pos += 1;
    }
    if let Some(rel_nl) = buf[cut_pos..].find('\n') {
        cut_pos += rel_nl + 1;
    }
    buf.drain(..cut_pos);
}

/// 実行アクションと終了コードに応じた案内メッセージを返す。
pub fn describe_exit_code(action: &MaintenanceAction, code: i32) -> &'static str {
    match action {
        MaintenanceAction::ChkdskRepair { .. } => match code {
            0 => "正常終了（エラーは検出されませんでした）。",
            1 => "エラーが検出・修復されたか、次回再起動時のチェックが予約されました。",
            2 => "ディスクのクリーンアップを実行しました。",
            3 => "チェックが未完了、またはエラーが発生しました。ログを確認してください。",
            _ => "異常終了しました。ログ内容を確認してください。",
        },
        _ => match code {
            0 => "正常終了しました。",
            3010 => "正常終了しました（変更を完全に反映するには PC の再起動が必要です）。",
            _ => "警告またはエラーで終了しました。ログ内容を確認してください。",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_drive_letter() {
        assert_eq!(normalize_drive_letter("c").unwrap(), "C:");
        assert_eq!(normalize_drive_letter("D:\\").unwrap(), "D:");
        assert!(normalize_drive_letter("1:").is_err());
        assert!(normalize_drive_letter("CD:").is_err());
    }

    #[test]
    fn test_dism_source_command_steps() {
        let action = MaintenanceAction::DismRestoreHealthWithSource {
            source: r"C:\My Portable\install.wim".to_string(),
            index: 2,
        };
        let steps = build_command_steps(&action).unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].exe_name, "dism.exe");
        assert_eq!(
            steps[0].args,
            vec![
                "/Online",
                "/Cleanup-Image",
                "/RestoreHealth",
                r"/Source:WIM:C:\My Portable\install.wim:2",
                "/LimitAccess",
            ]
        );
    }

    #[test]
    fn test_net_full_repair_has_three_steps() {
        let steps = build_command_steps(&MaintenanceAction::NetFullRepair).unwrap();
        assert_eq!(steps.len(), 3);
        assert_eq!(steps[0].exe_name, "netsh.exe");
        assert_eq!(steps[1].exe_name, "netsh.exe");
        assert_eq!(steps[2].exe_name, "ipconfig.exe");
    }

    #[test]
    fn test_chkdsk_has_stdin_and_confirmation() {
        let action = MaintenanceAction::ChkdskRepair {
            drive: "C:\\".to_string(),
        };
        assert!(action.confirmation_message().is_some());
        let steps = build_command_steps(&action).unwrap();
        assert_eq!(steps[0].args, vec!["C:", "/r"]);
        assert_eq!(steps[0].stdin_response, Some("Y\r\n"));
    }

    #[test]
    fn test_describe_exit_code() {
        assert!(describe_exit_code(&MaintenanceAction::DismRestoreHealth, 0).contains("正常終了"));
        assert!(describe_exit_code(&MaintenanceAction::DismRestoreHealth, 3010).contains("再起動"));
        assert!(describe_exit_code(&MaintenanceAction::SfcScannow, 1).contains("エラー"));
        assert!(
            describe_exit_code(
                &MaintenanceAction::ChkdskRepair {
                    drive: "C:".to_string()
                },
                1
            )
            .contains("再起動")
        );
    }

    #[test]
    fn test_trim_log_buffer_respects_line_boundary_and_utf8() {
        let mut buf = "あいう\r\n12345\r\n67890\r\n".to_string();
        trim_log_buffer(&mut buf, 14);
        assert!(buf.len() <= 14);
        assert_eq!(buf, "67890\r\n");
    }

    #[test]
    fn test_dism_image_info_command_steps() {
        let action = MaintenanceAction::DismImageInfo {
            source: r"WIM:E:\sources\install.wim:2".to_string(),
            index: 1,
        };
        let steps = build_command_steps(&action).unwrap();
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].exe_name, "dism.exe");
        assert_eq!(
            steps[0].args,
            vec![
                "/Get-ImageInfo",
                r"/ImageFile:E:\sources\install.wim",
                "/Index:1",
            ]
        );
    }
}
