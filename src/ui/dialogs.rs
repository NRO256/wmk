use crate::platform::win32::{
    BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS, BROWSEINFOW, CLSCTX_INPROC_SERVER,
    CLSID_FILE_OPEN_DIALOG, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoTaskMemFree, CoUninitialize, DWORD, FOS_FORCEFILESYSTEM, FOS_PATHMUSTEXIST, FOS_PICKFOLDERS,
    GetOpenFileNameW, GetSaveFileNameW, HRESULT_CANCELLED, HWND, IDYES, IFileOpenDialog,
    IID_IFILE_OPEN_DIALOG, IShellItem, MB_DEFBUTTON2, MB_ICONERROR, MB_ICONINFORMATION,
    MB_ICONQUESTION, MB_OK, MB_YESNO, MessageBoxW, OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR,
    OFN_OVERWRITEPROMPT, OFN_PATHMUSTEXIST, OPENFILENAMEW, SHBrowseForFolderW,
    SHGetPathFromIDListW, SIGDN_FILESYSPATH, from_wide_ptr, from_wide_slice, to_wide_null,
    with_wide,
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

struct ComInitGuard(bool);

impl ComInitGuard {
    fn new() -> Self {
        let hr = unsafe { CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED) };
        Self(hr >= 0)
    }
}

impl Drop for ComInitGuard {
    fn drop(&mut self) {
        if self.0 {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

struct FileOpenDialogGuard(*mut IFileOpenDialog);

impl Drop for FileOpenDialogGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                ((*(*self.0).lpVtbl).Release)(self.0.cast());
            }
        }
    }
}

struct ShellItemGuard(*mut IShellItem);

impl Drop for ShellItemGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                ((*(*self.0).lpVtbl).Release)(self.0.cast());
            }
        }
    }
}

fn browse_folder_legacy(hwnd: HWND, title: &str) -> Option<PathBuf> {
    with_wide(title, |title_w| unsafe {
        let mut display = [0u16; 260];
        let mut bi = BROWSEINFOW {
            hwndOwner: hwnd,
            pszDisplayName: display.as_mut_ptr(),
            lpszTitle: title_w,
            ulFlags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE,
            ..Default::default()
        };
        let pidl = SHBrowseForFolderW(&raw mut bi);
        if pidl.is_null() {
            return None;
        }
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
    })
}

pub fn browse_folder(hwnd: HWND, title: &str) -> Option<PathBuf> {
    let _com = ComInitGuard::new();
    let clsid = CLSID_FILE_OPEN_DIALOG;
    let iid = IID_IFILE_OPEN_DIALOG;
    let mut raw_dialog: *mut std::ffi::c_void = std::ptr::null_mut();
    let hr_create = unsafe {
        CoCreateInstance(
            &raw const clsid,
            std::ptr::null_mut(),
            CLSCTX_INPROC_SERVER,
            &raw const iid,
            &raw mut raw_dialog,
        )
    };
    if hr_create < 0 || raw_dialog.is_null() {
        return browse_folder_legacy(hwnd, title);
    }

    let dialog = FileOpenDialogGuard(raw_dialog.cast::<IFileOpenDialog>());
    unsafe {
        let mut opts: DWORD = 0;
        if ((*(*dialog.0).lpVtbl).GetOptions)(dialog.0.cast(), &raw mut opts) >= 0 {
            let new_opts = opts | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST;
            ((*(*dialog.0).lpVtbl).SetOptions)(dialog.0.cast(), new_opts);
        }
        let title_w = to_wide_null(title);
        ((*(*dialog.0).lpVtbl).SetTitle)(dialog.0.cast(), title_w.as_ptr());

        let hr_show = ((*(*dialog.0).lpVtbl).Show)(dialog.0.cast(), hwnd);
        if hr_show == HRESULT_CANCELLED || hr_show < 0 {
            return None;
        }

        let mut raw_item: *mut IShellItem = std::ptr::null_mut();
        if ((*(*dialog.0).lpVtbl).GetResult)(dialog.0.cast(), &raw mut raw_item) < 0
            || raw_item.is_null()
        {
            return None;
        }
        let item = ShellItemGuard(raw_item);

        let mut path_ptr: *mut u16 = std::ptr::null_mut();
        if ((*(*item.0).lpVtbl).GetDisplayName)(item.0.cast(), SIGDN_FILESYSPATH, &raw mut path_ptr)
            < 0
            || path_ptr.is_null()
        {
            return None;
        }
        let path = from_wide_ptr(path_ptr);
        CoTaskMemFree(path_ptr.cast());
        if path.is_empty() {
            None
        } else {
            Some(PathBuf::from(path))
        }
    }
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
