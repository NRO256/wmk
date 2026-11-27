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
pub type HMODULE = *mut c_void;
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
pub type HRESULT = i32;
pub type ATOM = u16;

pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

// Window Messages
pub const WM_CREATE: UINT = 0x0001;
pub const WM_DESTROY: UINT = 0x0002;
pub const WM_SIZE: UINT = 0x0005;
pub const WM_SETREDRAW: UINT = 0x000B;
pub const WM_CLOSE: UINT = 0x0010;
pub const WM_GETMINMAXINFO: UINT = 0x0024;
pub const WM_SETFONT: UINT = 0x0030;
pub const WM_GETDLGCODE: UINT = 0x0087;
pub const WM_NCDESTROY: UINT = 0x0082;
pub const WM_COMMAND: UINT = 0x0111;
pub const WM_TIMER: UINT = 0x0113;
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
pub const EM_LINESCROLL: UINT = 0x00B6;
pub const EM_LINEINDEX: UINT = 0x00BB;
pub const EM_REPLACESEL: UINT = 0x00C2;
pub const EM_SETLIMITTEXT: UINT = 0x00C5;
pub const EM_LINEFROMCHAR: UINT = 0x00C9;
pub const EM_GETFIRSTVISIBLELINE: UINT = 0x00CE;

pub const CBS_DROPDOWNLIST: DWORD = 0x0003;
pub const CBS_HASSTRINGS: DWORD = 0x0200;
pub const CB_ADDSTRING: UINT = 0x0143;
pub const CB_GETCURSEL: UINT = 0x0147;
pub const CB_SETCURSEL: UINT = 0x014E;

// Misc
pub const CW_USEDEFAULT: i32 = -0x8000_0000_i32;
pub const SW_HIDE: i32 = 0;
pub const SW_SHOW: i32 = 5;
pub const PBM_SETPOS: UINT = 0x0402;
pub const PBM_SETRANGE32: UINT = 0x0406;
pub const S_OK: i32 = 0;
pub const LOAD_LIBRARY_SEARCH_SYSTEM32: DWORD = 0x0000_0800;
pub const COLOR_BTNFACE: i32 = 15;
pub const IDC_ARROW: u16 = 32512;
pub const IDI_APPLICATION: u16 = 32512;
pub const IDI_SHIELD: u16 = 32518;

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

pub const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION: i32 = 9;
pub const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: DWORD = 0x0000_2000;

pub const DRIVE_REMOVABLE: UINT = 2;
pub const DRIVE_FIXED: UINT = 3;
pub const DRIVE_CDROM: UINT = 5;

pub const OFN_FILEMUSTEXIST: DWORD = 0x0000_1000;
pub const OFN_PATHMUSTEXIST: DWORD = 0x0000_0800;
pub const OFN_OVERWRITEPROMPT: DWORD = 0x0000_0002;
pub const OFN_NOCHANGEDIR: DWORD = 0x0000_0008;

pub const BIF_RETURNONLYFSDIRS: UINT = 0x0000_0001;
pub const BIF_NEWDIALOGSTYLE: UINT = 0x0000_0040;
pub const COINIT_APARTMENTTHREADED: DWORD = 0x2;
pub const CLSCTX_INPROC_SERVER: DWORD = 1;
pub const FOS_PICKFOLDERS: DWORD = 0x0000_0020;
pub const FOS_FORCEFILESYSTEM: DWORD = 0x0000_0040;
pub const FOS_PATHMUSTEXIST: DWORD = 0x0000_0800;
pub const SIGDN_FILESYSPATH: DWORD = 0x8005_8000;
pub const HRESULT_CANCELLED: HRESULT = 0x8007_04C7_u32 as HRESULT;

pub const ICC_STANDARD_CLASSES: DWORD = 0x0000_4000;
pub const ICC_PROGRESS_CLASS: DWORD = 0x0000_0020;

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
#[derive(Default)]
pub struct MINMAXINFO {
    pub ptReserved: POINT,
    pub ptMaxSize: POINT,
    pub ptMaxPosition: POINT,
    pub ptMinTrackSize: POINT,
    pub ptMaxTrackSize: POINT,
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

#[repr(C)]
#[derive(Default)]
pub struct JOBOBJECT_BASIC_LIMIT_INFORMATION {
    pub PerProcessUserTimeLimit: i64,
    pub PerJobUserTimeLimit: i64,
    pub LimitFlags: DWORD,
    pub MinimumWorkingSetSize: usize,
    pub MaximumWorkingSetSize: usize,
    pub ActiveProcessLimit: DWORD,
    pub Affinity: usize,
    pub PriorityClass: DWORD,
    pub SchedulingClass: DWORD,
}

#[repr(C)]
#[derive(Default)]
pub struct IO_COUNTERS {
    pub ReadOperationCount: u64,
    pub WriteOperationCount: u64,
    pub OtherOperationCount: u64,
    pub ReadTransferCount: u64,
    pub WriteTransferCount: u64,
    pub OtherTransferCount: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
    pub BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION,
    pub IoInfo: IO_COUNTERS,
    pub ProcessMemoryLimit: usize,
    pub JobMemoryLimit: usize,
    pub PeakProcessMemoryUsed: usize,
    pub PeakJobMemoryUsed: usize,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct SYSTEMTIME {
    pub wYear: WORD,
    pub wMonth: WORD,
    pub wDayOfWeek: WORD,
    pub wDay: WORD,
    pub wHour: WORD,
    pub wMinute: WORD,
    pub wSecond: WORD,
    pub wMilliseconds: WORD,
}

#[repr(C)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

pub const CLSID_FILE_OPEN_DIALOG: GUID = GUID {
    data1: 0xDC1C_5A9C,
    data2: 0xE88A,
    data3: 0x4dde,
    data4: [0xA5, 0xA1, 0x60, 0xF8, 0x2A, 0x20, 0xAE, 0xF7],
};

pub const IID_IFILE_OPEN_DIALOG: GUID = GUID {
    data1: 0xd57c_7288,
    data2: 0xd4ad,
    data3: 0x4768,
    data4: [0xbe, 0x02, 0x9d, 0x96, 0x95, 0x32, 0xd9, 0x60],
};

#[repr(C)]
pub struct IFileOpenDialog {
    pub lpVtbl: *const IFileOpenDialogVtbl,
}

#[repr(C)]
pub struct IFileOpenDialogVtbl {
    pub QueryInterface: *const c_void,
    pub AddRef: *const c_void,
    pub Release: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub Show: unsafe extern "system" fn(this: *mut c_void, hwndOwner: HWND) -> HRESULT,
    pub SetFileTypes: *const c_void,
    pub SetFileTypeIndex: *const c_void,
    pub GetFileTypeIndex: *const c_void,
    pub Advise: *const c_void,
    pub Unadvise: *const c_void,
    pub SetOptions: unsafe extern "system" fn(this: *mut c_void, fos: DWORD) -> HRESULT,
    pub GetOptions: unsafe extern "system" fn(this: *mut c_void, pfos: *mut DWORD) -> HRESULT,
    pub SetDefaultFolder: *const c_void,
    pub SetFolder: *const c_void,
    pub GetFolder: *const c_void,
    pub GetCurrentSelection: *const c_void,
    pub SetFileName: *const c_void,
    pub GetFileName: *const c_void,
    pub SetTitle: unsafe extern "system" fn(this: *mut c_void, pszTitle: *const u16) -> HRESULT,
    pub SetOkButtonLabel: *const c_void,
    pub SetFileNameLabel: *const c_void,
    pub GetResult:
        unsafe extern "system" fn(this: *mut c_void, ppsi: *mut *mut IShellItem) -> HRESULT,
    pub AddPlace: *const c_void,
    pub SetDefaultExtension: *const c_void,
    pub Close: *const c_void,
    pub SetClientGuid: *const c_void,
    pub ClearClientData: *const c_void,
    pub SetFilter: *const c_void,
}

#[repr(C)]
pub struct IShellItem {
    pub lpVtbl: *const IShellItemVtbl,
}

#[repr(C)]
pub struct IShellItemVtbl {
    pub QueryInterface: *const c_void,
    pub AddRef: *const c_void,
    pub Release: unsafe extern "system" fn(this: *mut c_void) -> u32,
    pub BindToHandler: *const c_void,
    pub GetParent: *const c_void,
    pub GetDisplayName: unsafe extern "system" fn(
        this: *mut c_void,
        sigdnName: DWORD,
        ppszName: *mut *mut u16,
    ) -> HRESULT,
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
    pub fn CreateJobObjectW(lpJobAttributes: *const c_void, lpName: *const u16) -> HANDLE;
    pub fn SetInformationJobObject(
        hJob: HANDLE,
        JobObjectInformationClass: i32,
        lpJobObjectInformation: *const c_void,
        cbJobObjectInformationLength: DWORD,
    ) -> BOOL;
    pub fn AssignProcessToJobObject(hJob: HANDLE, hProcess: HANDLE) -> BOOL;
    pub fn TerminateJobObject(hJob: HANDLE, uExitCode: UINT) -> BOOL;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn LoadLibraryExW(lpLibFileName: *const u16, hFile: HANDLE, dwFlags: DWORD) -> HMODULE;
    pub fn GetProcAddress(hModule: HMODULE, lpProcName: *const i8) -> *mut c_void;
    pub fn GetLocalTime(lpSystemTime: *mut SYSTEMTIME);
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
    pub fn SetTimer(
        hWnd: HWND,
        nIDEvent: UINT_PTR,
        uElapse: UINT,
        lpTimerFunc: *const c_void,
    ) -> UINT_PTR;
    pub fn KillTimer(hWnd: HWND, uIDEvent: UINT_PTR) -> BOOL;
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
    pub fn AdjustWindowRectExForDpi(
        lpRect: *mut RECT,
        dwStyle: DWORD,
        bMenu: BOOL,
        dwExStyle: DWORD,
        dpi: UINT,
    ) -> BOOL;
    pub fn InvalidateRect(hWnd: HWND, lpRect: *const RECT, bErase: BOOL) -> BOOL;
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
    pub fn CoInitializeEx(pvReserved: *mut c_void, dwCoInit: DWORD) -> HRESULT;
    pub fn CoUninitialize();
    pub fn CoCreateInstance(
        rclsid: *const GUID,
        pUnkOuter: *mut c_void,
        dwClsContext: DWORD,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> HRESULT;
}

#[link(name = "oleaut32")]
unsafe extern "system" {
    pub fn SysAllocString(psz: *const u16) -> *const u16;
    pub fn SysFreeString(bstrString: *const u16);
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

/// NUL終端された UTF-16 ポインタから String を復元する。
pub unsafe fn from_wide_ptr(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }
    String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(ptr, len) })
}

pub fn format_local_time_hms() -> String {
    let mut st = SYSTEMTIME::default();
    unsafe {
        GetLocalTime(&raw mut st);
    }
    format!("{:02}:{:02}:{:02}", st.wHour, st.wMinute, st.wSecond)
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
