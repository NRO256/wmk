#![allow(clippy::upper_case_acronyms)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::c_void;

pub type HWND = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type HMENU = *mut c_void;
pub type HFONT = *mut c_void;
pub type HBRUSH = *mut c_void;
pub type HGDIOBJ = *mut c_void;
pub type HICON = *mut c_void;
pub type HANDLE = *mut c_void;
pub type UINT_PTR = usize;
pub type DWORD_PTR = usize;
pub type LRESULT = isize;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type BOOL = i32;
pub type DWORD = u32;
pub type WORD = u16;
pub type UINT = u32;
pub type LONG = i32;
pub type ATOM = u16;

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

// Window Messages
pub const WM_CREATE: UINT = 0x0001;
pub const WM_DESTROY: UINT = 0x0002;
pub const WM_SIZE: UINT = 0x0005;
pub const WM_CLOSE: UINT = 0x0010;
pub const WM_SETFONT: UINT = 0x0030;
pub const WM_GETDLGCODE: UINT = 0x0087;
pub const WM_NCDESTROY: UINT = 0x0082;
pub const WM_COMMAND: UINT = 0x0111;
pub const WM_KEYDOWN: UINT = 0x0100;
pub const WM_CHAR: UINT = 0x0102;
pub const WM_DPICHANGED: UINT = 0x02E0;
pub const WM_APP: UINT = 0x8000;

pub const DLGC_WANTALLKEYS: isize = 0x0004;
pub const DLGC_WANTTAB: isize = 0x0002;
pub const VK_CONTROL: i32 = 0x11;
pub const VK_ESCAPE: usize = 0x1B;

// Window Styles
pub const WS_OVERLAPPED: DWORD = 0x0000_0000;
pub const WS_CAPTION: DWORD = 0x00C0_0000;
pub const WS_SYSMENU: DWORD = 0x0008_0000;
pub const WS_THICKFRAME: DWORD = 0x0004_0000;
pub const WS_MINIMIZEBOX: DWORD = 0x0002_0000;
pub const WS_MAXIMIZEBOX: DWORD = 0x0001_0000;
pub const WS_OVERLAPPEDWINDOW: DWORD =
    WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
pub const WS_CHILD: DWORD = 0x4000_0000;
pub const WS_VISIBLE: DWORD = 0x1000_0000;
pub const WS_TABSTOP: DWORD = 0x0001_0000;
pub const WS_VSCROLL: DWORD = 0x0020_0000;
pub const WS_HSCROLL: DWORD = 0x0010_0000;
pub const WS_BORDER: DWORD = 0x0080_0000;
pub const WS_EX_CLIENTEDGE: DWORD = 0x0000_0200;

// Button / Edit / ComboBox Styles & Messages
pub const BS_PUSHBUTTON: DWORD = 0x0000_0000;
pub const BN_CLICKED: WORD = 0;

pub const ES_LEFT: DWORD = 0x0000;
pub const ES_MULTILINE: DWORD = 0x0004;
pub const ES_AUTOVSCROLL: DWORD = 0x0040;
pub const ES_AUTOHSCROLL: DWORD = 0x0080;
pub const ES_READONLY: DWORD = 0x0800;
pub const EM_GETSEL: UINT = 0x00B0;
pub const EM_SETSEL: UINT = 0x00B1;
pub const EM_REPLACESEL: UINT = 0x00C2;
pub const EM_SETLIMITTEXT: UINT = 0x00C5;

pub const CBS_DROPDOWNLIST: DWORD = 0x0003;
pub const CBS_HASSTRINGS: DWORD = 0x0200;
pub const CB_ADDSTRING: UINT = 0x0143;
pub const CB_GETCURSEL: UINT = 0x0147;
pub const CB_SETCURSEL: UINT = 0x014E;

// Misc
pub const CW_USEDEFAULT: i32 = -0x8000_0000_i32;
pub const SW_SHOW: i32 = 5;
pub const COLOR_BTNFACE: i32 = 15;
pub const IDC_ARROW: u16 = 32512;
pub const IDI_APPLICATION: u16 = 32512;

pub const MB_OK: UINT = 0x0000_0000;
pub const MB_YESNO: UINT = 0x0000_0004;
pub const MB_DEFBUTTON2: UINT = 0x0000_0100;
pub const MB_ICONERROR: UINT = 0x0000_0010;
pub const MB_ICONQUESTION: UINT = 0x0000_0020;
pub const MB_ICONINFORMATION: UINT = 0x0000_0040;
pub const IDYES: i32 = 6;

pub const CP_932: UINT = 932;

pub const ES_CONTINUOUS: DWORD = 0x8000_0000;
pub const ES_SYSTEM_REQUIRED: DWORD = 0x0000_0001;
pub const INFINITE: DWORD = 0xFFFF_FFFF;

pub const DRIVE_REMOVABLE: UINT = 2;
pub const DRIVE_FIXED: UINT = 3;

pub const OFN_FILEMUSTEXIST: DWORD = 0x0000_1000;
pub const OFN_PATHMUSTEXIST: DWORD = 0x0000_0800;
pub const OFN_OVERWRITEPROMPT: DWORD = 0x0000_0002;
pub const OFN_NOCHANGEDIR: DWORD = 0x0000_0008;

pub const BIF_RETURNONLYFSDIRS: UINT = 0x0000_0001;
pub const BIF_NEWDIALOGSTYLE: UINT = 0x0000_0040;
pub const COINIT_APARTMENTTHREADED: DWORD = 0x2;

pub const ICC_STANDARD_CLASSES: DWORD = 0x0000_4000;

pub const FW_NORMAL: i32 = 400;
pub const DEFAULT_CHARSET: DWORD = 1;
pub const OUT_DEFAULT_PRECIS: DWORD = 0;
pub const CLIP_DEFAULT_PRECIS: DWORD = 0;
pub const CLEARTYPE_QUALITY: DWORD = 5;
pub const DEFAULT_PITCH: DWORD = 0;
pub const FIXED_PITCH: DWORD = 1;

pub type WNDPROC = unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT;
pub type SUBCLASSPROC =
    unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM, UINT_PTR, DWORD_PTR) -> LRESULT;

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: UINT,
    pub style: UINT,
    pub lpfnWndProc: Option<WNDPROC>,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HICON,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: *const u16,
    pub lpszClassName: *const u16,
    pub hIconSm: HICON,
}

impl Default for WNDCLASSEXW {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
#[derive(Default)]
pub struct POINT {
    pub x: LONG,
    pub y: LONG,
}

#[repr(C)]
#[derive(Default)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: DWORD,
    pub pt: POINT,
}

#[repr(C)]
#[derive(Default)]
pub struct RECT {
    pub left: LONG,
    pub top: LONG,
    pub right: LONG,
    pub bottom: LONG,
}

#[repr(C)]
pub struct INITCOMMONCONTROLSEX {
    pub dwSize: DWORD,
    pub dwICC: DWORD,
}

#[repr(C)]
pub struct OPENFILENAMEW {
    pub lStructSize: DWORD,
    pub hwndOwner: HWND,
    pub hInstance: HINSTANCE,
    pub lpstrFilter: *const u16,
    pub lpstrCustomFilter: *mut u16,
    pub nMaxCustFilter: DWORD,
    pub nFilterIndex: DWORD,
    pub lpstrFile: *mut u16,
    pub nMaxFile: DWORD,
    pub lpstrFileTitle: *mut u16,
    pub nMaxFileTitle: DWORD,
    pub lpstrInitialDir: *const u16,
    pub lpstrTitle: *const u16,
    pub Flags: DWORD,
    pub nFileOffset: WORD,
    pub nFileExtension: WORD,
    pub lpstrDefExt: *const u16,
    pub lCustData: LPARAM,
    pub lpfnHook: *const c_void,
    pub lpTemplateName: *const u16,
    pub pvReserved: *mut c_void,
    pub dwReserved: DWORD,
    pub FlagsEx: DWORD,
}

impl Default for OPENFILENAMEW {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[repr(C)]
pub struct BROWSEINFOW {
    pub hwndOwner: HWND,
    pub pidlRoot: *mut c_void,
    pub pszDisplayName: *mut u16,
    pub lpszTitle: *const u16,
    pub ulFlags: UINT,
    pub lpfn: *const c_void,
    pub lParam: LPARAM,
    pub iImage: i32,
}

impl Default for BROWSEINFOW {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    pub fn GetSystemDirectoryW(lpBuffer: *mut u16, uSize: UINT) -> UINT;
    pub fn GetLogicalDriveStringsW(nBufferLength: DWORD, lpBuffer: *mut u16) -> DWORD;
    pub fn GetDriveTypeW(lpRootPathName: *const u16) -> UINT;
    pub fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: *const u8,
        cbMultiByte: i32,
        lpWideCharStr: *mut u16,
        cchWideChar: i32,
    ) -> i32;
    pub fn SetThreadExecutionState(esFlags: DWORD) -> DWORD;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD) -> DWORD;
}

#[link(name = "user32")]
unsafe extern "system" {
    pub fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> ATOM;
    pub fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: DWORD,
        x: i32,
        y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: *const c_void,
    ) -> HWND;
    pub fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
    pub fn UpdateWindow(hWnd: HWND) -> BOOL;
    pub fn DestroyWindow(hWnd: HWND) -> BOOL;
    pub fn PostQuitMessage(nExitCode: i32);
    pub fn GetMessageW(
        lpMsg: *mut MSG,
        hWnd: HWND,
        wMsgFilterMin: UINT,
        wMsgFilterMax: UINT,
    ) -> BOOL;
    pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    pub fn IsDialogMessageW(hDlg: HWND, lpMsg: *const MSG) -> BOOL;
    pub fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    pub fn SetWindowTextW(hWnd: HWND, lpString: *const u16) -> BOOL;
    pub fn GetWindowTextLengthW(hWnd: HWND) -> i32;
    pub fn GetWindowTextW(hWnd: HWND, lpString: *mut u16, nMaxCount: i32) -> i32;
    pub fn EnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL;
    pub fn SetFocus(hWnd: HWND) -> HWND;
    pub fn GetFocus() -> HWND;
    pub fn GetParent(hWnd: HWND) -> HWND;
    pub fn IsWindowEnabled(hWnd: HWND) -> BOOL;
    pub fn GetKeyState(nVirtKey: i32) -> i16;
    pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn MoveWindow(
        hWnd: HWND,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        bRepaint: BOOL,
    ) -> BOOL;
    pub fn MessageBoxW(hWnd: HWND, lpText: *const u16, lpCaption: *const u16, uType: UINT) -> i32;
    pub fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: *const u16) -> HICON;
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: *const u16) -> HICON;
    pub fn GetDpiForWindow(hwnd: HWND) -> UINT;
    pub fn GetDlgItem(hDlg: HWND, nIDDlgItem: i32) -> HWND;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    pub fn CreateFontW(
        cHeight: i32,
        cWidth: i32,
        cEscapement: i32,
        cOrientation: i32,
        cWeight: i32,
        bItalic: DWORD,
        bUnderline: DWORD,
        bStrikeOut: DWORD,
        iCharSet: DWORD,
        iOutPrecision: DWORD,
        iClipPrecision: DWORD,
        iQuality: DWORD,
        iPitchAndFamily: DWORD,
        pszFaceName: *const u16,
    ) -> HFONT;
    pub fn DeleteObject(ho: HGDIOBJ) -> BOOL;
}

#[link(name = "comctl32")]
unsafe extern "system" {
    pub fn InitCommonControlsEx(picce: *const INITCOMMONCONTROLSEX) -> BOOL;
    pub fn SetWindowSubclass(
        hWnd: HWND,
        pfnSubclass: SUBCLASSPROC,
        uIdSubclass: UINT_PTR,
        dwRefData: DWORD_PTR,
    ) -> BOOL;
    pub fn DefSubclassProc(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn RemoveWindowSubclass(
        hWnd: HWND,
        pfnSubclass: SUBCLASSPROC,
        uIdSubclass: UINT_PTR,
    ) -> BOOL;
}

#[link(name = "comdlg32")]
unsafe extern "system" {
    pub fn GetOpenFileNameW(arg1: *mut OPENFILENAMEW) -> BOOL;
    pub fn GetSaveFileNameW(arg1: *mut OPENFILENAMEW) -> BOOL;
}

#[link(name = "shell32")]
unsafe extern "system" {
    pub fn SHBrowseForFolderW(lpbi: *mut BROWSEINFOW) -> *mut c_void;
    pub fn SHGetPathFromIDListW(pidl: *const c_void, pszPath: *mut u16) -> BOOL;
}

#[link(name = "ole32")]
unsafe extern "system" {
    pub fn CoTaskMemFree(pv: *mut c_void);
    pub fn CoInitializeEx(pvReserved: *mut c_void, dwCoInit: DWORD) -> i32;
    pub fn CoUninitialize();
}

pub fn to_wide_null(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn with_wide<R>(s: &str, f: impl FnOnce(*const u16) -> R) -> R {
    let mut stack_buf = [0u16; 256];
    let mut len = 0;
    for unit in s.encode_utf16() {
        if len + 1 >= stack_buf.len() {
            let heap_buf = to_wide_null(s);
            return f(heap_buf.as_ptr());
        }
        stack_buf[len] = unit;
        len += 1;
    }
    stack_buf[len] = 0;
    f(stack_buf.as_ptr())
}

pub fn from_wide_slice(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

/// 64bit符号拡張バグ回避のため必ず u16 -> usize -> *const u16 と変換する。
pub const fn make_int_resource_w(id: u16) -> *const u16 {
    id as usize as *const u16
}

/// Win32 MultiByteToWideChar(CP_932) を用いて Shift-JIS バイト列をデコードする。
pub fn decode_cp932(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    let Ok(len_i32) = i32::try_from(bytes.len()) else {
        return String::from_utf8_lossy(bytes).into_owned();
    };
    unsafe {
        let needed =
            MultiByteToWideChar(CP_932, 0, bytes.as_ptr(), len_i32, std::ptr::null_mut(), 0);
        if needed <= 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }
        let needed_usize = needed as usize;
        let mut stack_buf = [0u16; 512];
        let mut heap_buf = Vec::new();
        let buf: &mut [u16] = if needed_usize <= stack_buf.len() {
            &mut stack_buf[..needed_usize]
        } else {
            heap_buf.resize(needed_usize, 0);
            &mut heap_buf
        };
        let written =
            MultiByteToWideChar(CP_932, 0, bytes.as_ptr(), len_i32, buf.as_mut_ptr(), needed);
        if written <= 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }
        String::from_utf16_lossy(&buf[..written as usize])
    }
}

pub(crate) fn get_window_text(hwnd: HWND) -> String {
    if hwnd.is_null() {
        return String::new();
    }
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len <= 0 {
            return String::new();
        }
        let needed = (len as usize) + 1;
        let mut stack_buf = [0u16; 256];
        let mut heap_buf = Vec::new();
        let buf: &mut [u16] = if needed <= stack_buf.len() {
            &mut stack_buf[..needed]
        } else {
            heap_buf.resize(needed, 0);
            &mut heap_buf
        };
        let read = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        if read <= 0 {
            return String::new();
        }
        String::from_utf16_lossy(&buf[..read as usize])
    }
}

pub(crate) fn set_window_text(hwnd: HWND, text: &str) {
    if hwnd.is_null() {
        return;
    }
    with_wide(text, |ptr| unsafe {
        SetWindowTextW(hwnd, ptr);
    });
}
