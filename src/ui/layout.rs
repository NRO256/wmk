use crate::platform::drive::list_available_drives;
use crate::platform::win32::{
    BS_PUSHBUTTON, CB_ADDSTRING, CB_SETCURSEL, CBS_DROPDOWNLIST, CBS_HASSTRINGS, CLEARTYPE_QUALITY,
    CLIP_DEFAULT_PRECIS, CreateFontW, CreateWindowExW, DEFAULT_CHARSET, DEFAULT_PITCH,
    EM_SETLIMITTEXT, ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_LEFT, ES_MULTILINE, ES_READONLY,
    FIXED_PITCH, FW_NORMAL, GetClientRect, GetDlgItem, HFONT, HINSTANCE, HMENU, HWND, LPARAM,
    MoveWindow, OUT_DEFAULT_PRECIS, PBS_MARQUEE, RECT, SW_HIDE, SendMessageW, SetWindowSubclass,
    ShowWindow, TRUE, UINT_PTR, WM_SETFONT, WPARAM, WS_BORDER, WS_CHILD, WS_EX_CLIENTEDGE,
    WS_HSCROLL, WS_TABSTOP, WS_VISIBLE, WS_VSCROLL, with_wide,
};

// 共通・上部バー
pub(super) const ID_LBL_STATUS: usize = 101;
pub(super) const ID_BTN_CANCEL: usize = 102;
pub(super) const ID_PROGRESS_BAR: usize = 103;

// メンテナンス操作
pub(super) const ID_BTN_DISM_ONLINE: usize = 201;
pub(super) const ID_BTN_SFC: usize = 202;
pub(super) const ID_BTN_NET_ALL: usize = 203;
pub(super) const ID_BTN_NTP_RESYNC: usize = 204;
pub(super) const ID_BTN_SYSINFO: usize = 205;

pub(super) const ID_LBL_CHKDSK_DRIVE: usize = 210;
pub(super) const ID_CMB_CHKDSK_DRIVE: usize = 211;
pub(super) const ID_BTN_CHKDSK: usize = 212;

pub(super) const ID_LBL_DISM_SOURCE: usize = 220;
pub(super) const ID_EDIT_DISM_SOURCE: usize = 221;
pub(super) const ID_BTN_DISM_BROWSE_DIR: usize = 222;
pub(super) const ID_BTN_DISM_BROWSE_IMG: usize = 223;
pub(super) const ID_LBL_DISM_INDEX: usize = 224;
pub(super) const ID_CMB_DISM_INDEX: usize = 225;
pub(super) const ID_BTN_DISM_SOURCE: usize = 226;

// ログ領域
pub(super) const ID_LBL_LOG: usize = 301;
pub(super) const ID_BTN_LOG_CLEAR: usize = 302;
pub(super) const ID_BTN_LOG_SAVE: usize = 303;
pub(super) const ID_EDIT_LOG: usize = 304;

pub(super) const LOG_SUBCLASS_ID: UINT_PTR = 1;
pub(super) const MIN_CLIENT_WIDTH: i32 = 700;
pub(super) const MIN_CLIENT_HEIGHT: i32 = 520;

pub(super) const RUNNING_DISABLE_IDS: &[usize] = &[
    ID_BTN_DISM_ONLINE,
    ID_BTN_SFC,
    ID_BTN_NET_ALL,
    ID_BTN_NTP_RESYNC,
    ID_BTN_SYSINFO,
    ID_CMB_CHKDSK_DRIVE,
    ID_BTN_CHKDSK,
    ID_EDIT_DISM_SOURCE,
    ID_BTN_DISM_BROWSE_DIR,
    ID_BTN_DISM_BROWSE_IMG,
    ID_CMB_DISM_INDEX,
    ID_BTN_DISM_SOURCE,
];

const UI_FONT_IDS: &[usize] = &[
    ID_LBL_STATUS,
    ID_BTN_CANCEL,
    ID_BTN_DISM_ONLINE,
    ID_BTN_SFC,
    ID_BTN_NET_ALL,
    ID_BTN_NTP_RESYNC,
    ID_BTN_SYSINFO,
    ID_LBL_CHKDSK_DRIVE,
    ID_CMB_CHKDSK_DRIVE,
    ID_BTN_CHKDSK,
    ID_LBL_DISM_SOURCE,
    ID_EDIT_DISM_SOURCE,
    ID_BTN_DISM_BROWSE_DIR,
    ID_BTN_DISM_BROWSE_IMG,
    ID_LBL_DISM_INDEX,
    ID_CMB_DISM_INDEX,
    ID_BTN_DISM_SOURCE,
    ID_LBL_LOG,
    ID_BTN_LOG_CLEAR,
    ID_BTN_LOG_SAVE,
];

pub(super) fn ctrl(hwnd: HWND, id: usize) -> HWND {
    unsafe { GetDlgItem(hwnd, id as i32) }
}

pub(super) fn scale(px: i32, dpi: u32) -> i32 {
    (px * (dpi as i32) + 48) / 96
}

fn create_child(
    parent: HWND,
    hinst: HINSTANCE,
    class_name: &str,
    text: &str,
    style: u32,
    ex_style: u32,
    id: usize,
) -> HWND {
    with_wide(class_name, |cls| {
        with_wide(text, |txt| unsafe {
            CreateWindowExW(
                ex_style,
                cls,
                txt,
                style | WS_CHILD | WS_VISIBLE,
                0,
                0,
                100,
                24,
                parent,
                id as HMENU,
                hinst,
                std::ptr::null(),
            )
        })
    })
}

pub(super) fn create_fonts_for_dpi(dpi: u32) -> (HFONT, HFONT) {
    let height = -scale(14, dpi);
    with_wide("BIZ UDPゴシック", |ui_face| {
        with_wide("BIZ UDゴシック", |mono_face| unsafe {
            let h_ui = CreateFontW(
                height,
                0,
                0,
                0,
                FW_NORMAL,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                DEFAULT_PITCH,
                ui_face,
            );
            let h_mono = CreateFontW(
                height,
                0,
                0,
                0,
                FW_NORMAL,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                CLEARTYPE_QUALITY,
                FIXED_PITCH,
                mono_face,
            );
            (h_ui, h_mono)
        })
    })
}

pub(super) fn apply_fonts(hwnd: HWND, hfont_ui: HFONT, hfont_mono: HFONT) {
    unsafe {
        for &id in UI_FONT_IDS {
            let h = ctrl(hwnd, id);
            if !h.is_null() {
                SendMessageW(h, WM_SETFONT, hfont_ui as WPARAM, TRUE as LPARAM);
            }
        }
        let h_log = ctrl(hwnd, ID_EDIT_LOG);
        if !h_log.is_null() {
            SendMessageW(h_log, WM_SETFONT, hfont_mono as WPARAM, TRUE as LPARAM);
        }
    }
}

fn move_ctrl(parent: HWND, id: usize, x: i32, y: i32, w: i32, h: i32) {
    let ch = ctrl(parent, id);
    if !ch.is_null() {
        unsafe {
            MoveWindow(ch, x, y, w, h, TRUE);
        }
    }
}

pub(super) fn layout_controls(hwnd: HWND, dpi: u32) {
    let mut rc = RECT::default();
    unsafe {
        GetClientRect(hwnd, &raw mut rc);
    }
    let w = (rc.right - rc.left).max(scale(MIN_CLIENT_WIDTH, dpi));
    let h = (rc.bottom - rc.top).max(scale(MIN_CLIENT_HEIGHT, dpi));

    let pad = scale(12, dpi);
    let gap = scale(8, dpi);
    let row_h = scale(30, dpi);
    let lbl_h = scale(20, dpi);
    let inner_w = w - pad * 2;
    let mut y = pad;

    // 1. 状態表示 + 実行中断 + プログレスバー
    let cancel_w = scale(140, dpi);
    let status_w = inner_w - cancel_w - gap;
    move_ctrl(hwnd, ID_LBL_STATUS, pad, y + scale(5, dpi), status_w, lbl_h);
    move_ctrl(
        hwnd,
        ID_BTN_CANCEL,
        pad + status_w + gap,
        y,
        cancel_w,
        row_h,
    );
    y += row_h + scale(4, dpi);
    let prog_h = scale(8, dpi);
    move_ctrl(hwnd, ID_PROGRESS_BAR, pad, y, inner_w, prog_h);
    y += prog_h + gap;

    // 2. 基本修復 (2列)
    let half_w = (inner_w - gap) / 2;
    move_ctrl(hwnd, ID_BTN_DISM_ONLINE, pad, y, half_w, row_h);
    move_ctrl(
        hwnd,
        ID_BTN_SFC,
        pad + half_w + gap,
        y,
        inner_w - half_w - gap,
        row_h,
    );
    y += row_h + gap;

    // 3. ネットワーク・時刻・システム情報 (3列)
    let third_w = (inner_w - gap * 2) / 3;
    move_ctrl(hwnd, ID_BTN_NET_ALL, pad, y, third_w, row_h);
    move_ctrl(
        hwnd,
        ID_BTN_NTP_RESYNC,
        pad + third_w + gap,
        y,
        third_w,
        row_h,
    );
    move_ctrl(
        hwnd,
        ID_BTN_SYSINFO,
        pad + (third_w + gap) * 2,
        y,
        inner_w - (third_w + gap) * 2,
        row_h,
    );
    y += row_h + gap + scale(4, dpi);

    // 4. CHKDSK
    let drv_lbl_w = scale(120, dpi);
    let drv_cmb_w = scale(80, dpi);
    let chkdsk_btn_w = inner_w - drv_lbl_w - drv_cmb_w - gap * 2;
    move_ctrl(
        hwnd,
        ID_LBL_CHKDSK_DRIVE,
        pad,
        y + scale(5, dpi),
        drv_lbl_w,
        lbl_h,
    );
    move_ctrl(
        hwnd,
        ID_CMB_CHKDSK_DRIVE,
        pad + drv_lbl_w + gap,
        y + scale(2, dpi),
        drv_cmb_w,
        scale(180, dpi),
    );
    move_ctrl(
        hwnd,
        ID_BTN_CHKDSK,
        pad + drv_lbl_w + drv_cmb_w + gap * 2,
        y,
        chkdsk_btn_w,
        row_h,
    );
    y += row_h + gap + scale(4, dpi);

    // 5. DISM ソース指定修復
    move_ctrl(hwnd, ID_LBL_DISM_SOURCE, pad, y, inner_w, lbl_h);
    y += lbl_h + scale(2, dpi);

    let btn_sm = scale(100, dpi);
    let edit_src_w = inner_w - (btn_sm * 2 + gap * 2);
    move_ctrl(hwnd, ID_EDIT_DISM_SOURCE, pad, y, edit_src_w, row_h);
    move_ctrl(
        hwnd,
        ID_BTN_DISM_BROWSE_DIR,
        pad + edit_src_w + gap,
        y,
        btn_sm,
        row_h,
    );
    move_ctrl(
        hwnd,
        ID_BTN_DISM_BROWSE_IMG,
        pad + edit_src_w + gap * 2 + btn_sm,
        y,
        btn_sm,
        row_h,
    );
    y += row_h + gap;

    let idx_lbl_w = scale(85, dpi);
    let idx_cmb_w = scale(60, dpi);
    let run_src_w = inner_w - idx_lbl_w - idx_cmb_w - gap * 2;
    move_ctrl(
        hwnd,
        ID_LBL_DISM_INDEX,
        pad,
        y + scale(5, dpi),
        idx_lbl_w,
        lbl_h,
    );
    move_ctrl(
        hwnd,
        ID_CMB_DISM_INDEX,
        pad + idx_lbl_w + gap,
        y + scale(2, dpi),
        idx_cmb_w,
        scale(180, dpi),
    );
    move_ctrl(
        hwnd,
        ID_BTN_DISM_SOURCE,
        pad + idx_lbl_w + idx_cmb_w + gap * 2,
        y,
        run_src_w,
        row_h,
    );
    y += row_h + gap + scale(6, dpi);

    // 6. 下部ログ領域
    let log_btn_w = scale(120, dpi);
    move_ctrl(
        hwnd,
        ID_LBL_LOG,
        pad,
        y + scale(5, dpi),
        scale(200, dpi),
        lbl_h,
    );
    move_ctrl(
        hwnd,
        ID_BTN_LOG_CLEAR,
        w - pad - log_btn_w * 2 - gap,
        y,
        log_btn_w,
        row_h,
    );
    move_ctrl(
        hwnd,
        ID_BTN_LOG_SAVE,
        w - pad - log_btn_w,
        y,
        log_btn_w,
        row_h,
    );
    y += row_h + gap;

    let log_edit_h = (h - y - pad).max(scale(120, dpi));
    move_ctrl(hwnd, ID_EDIT_LOG, pad, y, inner_w, log_edit_h);
}

pub(super) fn create_controls(hwnd: HWND, hinst: HINSTANCE) {
    // 1. 状態・中断
    create_child(hwnd, hinst, "STATIC", "状態: 待機中", 0, 0, ID_LBL_STATUS);
    create_child(
        hwnd,
        hinst,
        "BUTTON",
        "実行中断(&X)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_BTN_CANCEL,
    );
    let h_prog = create_child(
        hwnd,
        hinst,
        "msctls_progress32",
        "",
        PBS_MARQUEE,
        0,
        ID_PROGRESS_BAR,
    );
    if !h_prog.is_null() {
        unsafe {
            ShowWindow(h_prog, SW_HIDE);
        }
    }

    // 2. 基本修復
    create_child(
        hwnd,
        hinst,
        "BUTTON",
        "DISM オンライン修復(&D)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_BTN_DISM_ONLINE,
    );
    create_child(
        hwnd,
        hinst,
        "BUTTON",
        "SFC スキャン (/scannow)(&C)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_BTN_SFC,
    );

    // 3. ネットワーク・時刻・システム情報
    create_child(
        hwnd,
        hinst,
        "BUTTON",
        "ネットワーク一括修復(&N)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_BTN_NET_ALL,
    );
    create_child(
        hwnd,
        hinst,
        "BUTTON",
        "NTP 時刻同期(&T)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_BTN_NTP_RESYNC,
    );
    create_child(
        hwnd,
        hinst,
        "BUTTON",
        "システム情報取得(&V)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_BTN_SYSINFO,
    );

    // 4. CHKDSK
    create_child(
        hwnd,
        hinst,
        "STATIC",
        "CHKDSK 対象:",
        0,
        0,
        ID_LBL_CHKDSK_DRIVE,
    );
    let h_cmb_drv = create_child(
        hwnd,
        hinst,
        "COMBOBOX",
        "",
        WS_TABSTOP | CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_VSCROLL,
        0,
        ID_CMB_CHKDSK_DRIVE,
    );
    for d in list_available_drives() {
        with_wide(&d, |w| unsafe {
            SendMessageW(h_cmb_drv, CB_ADDSTRING, 0, w as LPARAM);
        });
    }
    unsafe {
        SendMessageW(h_cmb_drv, CB_SETCURSEL, 0, 0);
    }
    create_child(
        hwnd,
        hinst,
        "BUTTON",
        "CHKDSK 修復実行 (/r)(&K)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_BTN_CHKDSK,
    );

    // 5. DISM ソース指定修復
    create_child(
        hwnd,
        hinst,
        "STATIC",
        "DISM 修復ソース（フォルダ、または WIM / ESD）:",
        0,
        0,
        ID_LBL_DISM_SOURCE,
    );
    create_child(
        hwnd,
        hinst,
        "EDIT",
        "",
        WS_TABSTOP | WS_BORDER | ES_LEFT | ES_AUTOHSCROLL,
        WS_EX_CLIENTEDGE,
        ID_EDIT_DISM_SOURCE,
    );
    create_child(
        hwnd,
        hinst,
        "BUTTON",
        "フォルダ(&F)...",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_BTN_DISM_BROWSE_DIR,
    );
    create_child(
        hwnd,
        hinst,
        "BUTTON",
        "WIM/ESD(&W)...",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_BTN_DISM_BROWSE_IMG,
    );
    create_child(
        hwnd,
        hinst,
        "STATIC",
        "インデックス:",
        0,
        0,
        ID_LBL_DISM_INDEX,
    );
    let h_cmb_idx = create_child(
        hwnd,
        hinst,
        "COMBOBOX",
        "",
        WS_TABSTOP | CBS_DROPDOWNLIST | CBS_HASSTRINGS | WS_VSCROLL,
        0,
        ID_CMB_DISM_INDEX,
    );
    for idx in 1..=10 {
        with_wide(&idx.to_string(), |s| unsafe {
            SendMessageW(h_cmb_idx, CB_ADDSTRING, 0, s as LPARAM);
        });
    }
    unsafe {
        SendMessageW(h_cmb_idx, CB_SETCURSEL, 0, 0);
    }
    create_child(
        hwnd,
        hinst,
        "BUTTON",
        "DISM ソース指定修復 (/LimitAccess)(&M)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_BTN_DISM_SOURCE,
    );

    // 6. 実行ログ欄
    create_child(hwnd, hinst, "STATIC", "実行ログ:", 0, 0, ID_LBL_LOG);
    create_child(
        hwnd,
        hinst,
        "BUTTON",
        "ログクリア(&L)",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_BTN_LOG_CLEAR,
    );
    create_child(
        hwnd,
        hinst,
        "BUTTON",
        "ログ保存(&S)...",
        WS_TABSTOP | BS_PUSHBUTTON,
        0,
        ID_BTN_LOG_SAVE,
    );
    let h_log = create_child(
        hwnd,
        hinst,
        "EDIT",
        "",
        WS_TABSTOP
            | WS_BORDER
            | WS_VSCROLL
            | WS_HSCROLL
            | ES_MULTILINE
            | ES_AUTOVSCROLL
            | ES_AUTOHSCROLL
            | ES_READONLY,
        WS_EX_CLIENTEDGE,
        ID_EDIT_LOG,
    );
    unsafe {
        // Win32複数行EDITの既定32KB上限を約2GBへ拡張 (0指定は64KBになるため明示値を使用)
        SendMessageW(h_log, EM_SETLIMITTEXT, 0x7FFF_FFFE, 0);
        SetWindowSubclass(h_log, super::log_edit_subclass_proc, LOG_SUBCLASS_ID, 0);
    }
}
