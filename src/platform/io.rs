use std::fs;
use std::path::{Path, PathBuf};

/// 現在の実行ファイルが置かれているディレクトリを返す。
pub fn get_exe_directory() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
}

/// ログ文字列を UTF-8 でファイルへ保存する。
pub fn save_text_utf8(path: &Path, content: &str) -> Result<(), String> {
    fs::write(path, content.as_bytes())
        .map_err(|e| format!("ログの保存に失敗しました ({}): {e}", path.display()))
}
