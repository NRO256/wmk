use crate::platform::win32::{
    BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS, BROWSEINFOW, COINIT_APARTMENTTHREADED,
    CoInitializeEx, CoTaskMemFree, CoUninitialize, GetOpenFileNameW, GetSaveFileNameW, HWND, IDYES,
    MB_DEFBUTTON2, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION, MB_OK, MB_YESNO, MessageBoxW,
    OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR, OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW,
    SHBrowseForFolderW, SHGetPathFromIDListW, from_wide_slice, with_wide,
};
use std::path::PathBuf;

fn message_box(hwnd: HWND, title: &str, message: &str, flags: u32) -> i32 {
    with_wide(title, |t| {
        with_wide(message, |m| unsafe { MessageBoxW(hwnd, m, t, flags) })
    })
}

pub fn show_info(hwnd: HWND, title: &str, message: &str) {
    message_box(hwnd, title, message, MB_OK | MB_ICONINFORMATION);
}

pub fn show_error(hwnd: HWND, title: &str, message: &str) {
    message_box(hwnd, title, message, MB_OK | MB_ICONERROR);
}

pub fn confirm_yes_no(hwnd: HWND, title: &str, message: &str) -> bool {
    message_box(
        hwnd,
        title,
        message,
        MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2,
    ) == IDYES
}

pub fn browse_folder(hwnd: HWND, title: &str) -> Option<PathBuf> {
    with_wide(title, |title_w| unsafe {
        let com_hr = CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED);
        let com_initialized = com_hr >= 0;

        let mut display = [0u16; 260];
        let mut bi = BROWSEINFOW {
            hwndOwner: hwnd,
            pszDisplayName: display.as_mut_ptr(),
            lpszTitle: title_w,
            ulFlags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE,
            ..Default::default()
        };
        let pidl = SHBrowseForFolderW(&raw mut bi);
        let result = if !pidl.is_null() {
            let mut path_buf = [0u16; 1024];
            let ok = SHGetPathFromIDListW(pidl, path_buf.as_mut_ptr());
            CoTaskMemFree(pidl);
            if ok != 0 {
                let s = from_wide_slice(&path_buf);
                if s.is_empty() {
                    None
                } else {
                    Some(PathBuf::from(s))
                }
            } else {
                None
            }
        } else {
            None
        };

        if com_initialized {
            CoUninitialize();
        }
        result
    })
}

pub fn open_image_file(hwnd: HWND) -> Option<PathBuf> {
    let filter: Vec<u16> =
        "Windows イメージ (*.wim;*.esd)\0*.wim;*.esd\0すべてのファイル (*.*)\0*.*\0\0"
            .encode_utf16()
            .collect();
    let mut file_buf = [0u16; 1024];

    let ok = with_wide("修復ソースイメージを選択", |title_ptr| {
        let mut ofn = OPENFILENAMEW {
            lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
            hwndOwner: hwnd,
            lpstrFilter: filter.as_ptr(),
            lpstrFile: file_buf.as_mut_ptr(),
            nMaxFile: file_buf.len() as u32,
            lpstrTitle: title_ptr,
            Flags: OFN_FILEMUSTEXIST | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR,
            ..Default::default()
        };
        unsafe { GetOpenFileNameW(&raw mut ofn) }
    });
    if ok != 0 {
        let s = from_wide_slice(&file_buf);
        if !s.is_empty() {
            return Some(PathBuf::from(s));
        }
    }
    None
}

pub fn save_log_file(hwnd: HWND) -> Option<PathBuf> {
    let filter: Vec<u16> = "テキストファイル (*.txt)\0*.txt\0すべてのファイル (*.*)\0*.*\0\0"
        .encode_utf16()
        .collect();
    let mut file_buf = [0u16; 1024];
    for (dst, unit) in file_buf.iter_mut().zip("wmk_log.txt".encode_utf16()) {
        *dst = unit;
    }

    let ok = with_wide("実行ログを保存", |title_ptr| {
        with_wide("txt", |def_ext_ptr| {
            let mut ofn = OPENFILENAMEW {
                lStructSize: std::mem::size_of::<OPENFILENAMEW>() as u32,
                hwndOwner: hwnd,
                lpstrFilter: filter.as_ptr(),
                lpstrFile: file_buf.as_mut_ptr(),
                nMaxFile: file_buf.len() as u32,
                lpstrTitle: title_ptr,
                lpstrDefExt: def_ext_ptr,
                Flags: OFN_OVERWRITEPROMPT | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR,
                ..Default::default()
            };
            unsafe { GetSaveFileNameW(&raw mut ofn) }
        })
    });
    if ok != 0 {
        let s = from_wide_slice(&file_buf);
        if !s.is_empty() {
            return Some(PathBuf::from(s));
        }
    }
    None
}
