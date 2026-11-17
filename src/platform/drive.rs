use crate::platform::win32::{
    DRIVE_FIXED, DRIVE_REMOVABLE, GetDriveTypeW, GetLogicalDriveStringsW,
};

/// 利用可能な固定ドライブ・リムーバブルドライブの一覧（例: `["C:", "D:"]`）を返す。
pub fn list_available_drives() -> Vec<String> {
    let mut buf = [0u16; 512];
    let len = unsafe { GetLogicalDriveStringsW(buf.len() as u32, buf.as_mut_ptr()) };
    if len == 0 || (len as usize) >= buf.len() {
        return vec!["C:".to_string()];
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
        if dtype == DRIVE_FIXED || dtype == DRIVE_REMOVABLE {
            drives.push(format!("{letter}:"));
        }
    }

    if drives.is_empty() {
        drives.push("C:".to_string());
    }
    drives
}
