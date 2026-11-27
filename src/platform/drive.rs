use crate::platform::win32::{
    DRIVE_CDROM, DRIVE_FIXED, DRIVE_REMOVABLE, GetDriveTypeW, GetLogicalDriveStringsW, UINT,
};
use std::path::PathBuf;

fn list_drives_by_types(allowed_types: &[UINT]) -> Vec<String> {
    let mut buf = [0u16; 512];
    let len = unsafe { GetLogicalDriveStringsW(buf.len() as u32, buf.as_mut_ptr()) };
    if len == 0 || (len as usize) >= buf.len() {
        return Vec::new();
    }

    let mut drives = Vec::new();
    let slice = &buf[..len as usize];
    for root in slice.split(|&c| c == 0) {
        let Some(&first) = root.first() else {
            continue;
        };
        if first > 0x7F {
            continue;
        }
        let letter = (first as u8 as char).to_ascii_uppercase();
        if !letter.is_ascii_uppercase() {
            continue;
        }
        let root_w = [letter as u16, b':' as u16, b'\\' as u16, 0];
        let dtype = unsafe { GetDriveTypeW(root_w.as_ptr()) };
        if allowed_types.contains(&dtype) {
            drives.push(format!("{letter}:"));
        }
    }

    drives
}

/// 利用可能な固定ドライブ・リムーバブルドライブの一覧（例: `["C:", "D:"]`）を返す。
pub fn list_available_drives() -> Vec<String> {
    let mut drives = list_drives_by_types(&[DRIVE_FIXED, DRIVE_REMOVABLE]);
    if drives.is_empty() {
        drives.push("C:".to_string());
    }
    drives
}

/// マウント済み光学ドライブ（`DRIVE_CDROM`）から `sources\install.wim` または `sources\install.esd` を探索する。
pub fn find_mounted_iso_image() -> Option<PathBuf> {
    for drive in list_drives_by_types(&[DRIVE_CDROM]) {
        let wim = PathBuf::from(format!("{drive}\\sources\\install.wim"));
        if wim.is_file() {
            return Some(wim);
        }
        let esd = PathBuf::from(format!("{drive}\\sources\\install.esd"));
        if esd.is_file() {
            return Some(esd);
        }
    }
    None
}
