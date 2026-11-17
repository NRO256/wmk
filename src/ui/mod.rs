pub mod dialogs;
pub mod state;

use crate::domain::commands::{MaintenanceAction, build_command_steps};
use crate::domain::source_detect::detect_portable_image_in_dir;
use crate::platform::drive::list_available_drives;
use crate::platform::io::{get_exe_directory, save_text_utf8};
use crate::platform::process::{WM_APP_WORKER_EVENT, WorkerEvent, spawn_command_sequence};
use crate::platform::win32::{
    BN_CLICKED, BS_PUSHBUTTON, CB_ADDSTRING, CB_GETCURSEL, CB_SETCURSEL, CBS_DROPDOWNLIST,
    CBS_HASSTRINGS, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, COLOR_BTNFACE, CW_USEDEFAULT,
    CreateFontW, CreateWindowExW, DEFAULT_CHARSET, DEFAULT_PITCH, DLGC_WANTALLKEYS, DLGC_WANTTAB,
    DWORD_PTR, DefSubclassProc, DefWindowProcW, DestroyWindow, DispatchMessageW, EM_GETSEL,
    EM_REPLACESEL, EM_SETLIMITTEXT, EM_SETSEL, ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_LEFT,
    ES_MULTILINE, ES_READONLY, EnableWindow, FALSE, FIXED_PITCH, FW_NORMAL, GetClientRect,
    GetDlgItem, GetDpiForWindow, GetFocus, GetKeyState, GetMessageW, GetModuleHandleW, GetParent,
    GetWindowTextLengthW, HBRUSH, HFONT, HINSTANCE, HMENU, HWND, ICC_STANDARD_CLASSES, IDC_ARROW,
    IDI_APPLICATION, INITCOMMONCONTROLSEX, InitCommonControlsEx, IsDialogMessageW, IsWindowEnabled,
    LPARAM, LRESULT, LoadCursorW, LoadIconW, MSG, MoveWindow, OUT_DEFAULT_PRECIS, PostQuitMessage,
    RECT, RegisterClassExW, RemoveWindowSubclass, SW_SHOW, SendMessageW, SetFocus,
    SetWindowSubclass, ShowWindow, TRUE, TranslateMessage, UINT, UINT_PTR, UpdateWindow,
    VK_CONTROL, VK_ESCAPE, WM_CHAR, WM_CLOSE, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_DPICHANGED,
    WM_GETDLGCODE, WM_KEYDOWN, WM_NCDESTROY, WM_SETFONT, WM_SIZE, WNDCLASSEXW, WPARAM, WS_BORDER,
    WS_CHILD, WS_EX_CLIENTEDGE, WS_HSCROLL, WS_OVERLAPPEDWINDOW, WS_TABSTOP, WS_VISIBLE,
    WS_VSCROLL, get_window_text, make_int_resource_w, set_window_text, with_wide,
};
use crate::ui::dialogs::{
    browse_folder, confirm_yes_no, open_image_file, save_log_file, show_error, show_info,
};
use crate::ui::state::AppState;
use std::cell::RefCell;

// 共通・上部バー
const ID_LBL_STATUS: usize = 101;
const ID_BTN_CANCEL: usize = 102;

// メンテナンス操作
const ID_BTN_DISM_ONLINE: usize = 201;
const ID_BTN_SFC: usize = 202;
const ID_BTN_NET_ALL: usize = 203;
const ID_BTN_NTP_RESYNC: usize = 204;
const ID_BTN_SYSINFO: usize = 205;

const ID_LBL_CHKDSK_DRIVE: usize = 210;
const ID_CMB_CHKDSK_DRIVE: usize = 211;
const ID_BTN_CHKDSK: usize = 212;

const ID_LBL_DISM_SOURCE: usize = 220;
const ID_EDIT_DISM_SOURCE: usize = 221;
const ID_BTN_DISM_BROWSE_DIR: usize = 222;
const ID_BTN_DISM_BROWSE_IMG: usize = 223;
const ID_LBL_DISM_INDEX: usize = 224;
const ID_CMB_DISM_INDEX: usize = 225;
const ID_BTN_DISM_SOURCE: usize = 226;

// ログ領域
const ID_LBL_LOG: usize = 301;
const ID_BTN_LOG_CLEAR: usize = 302;
const ID_BTN_LOG_SAVE: usize = 303;
const ID_EDIT_LOG: usize = 304;

const LOG_SUBCLASS_ID: UINT_PTR = 1;

const ACTION_BUTTON_IDS: &[usize] = &[
    ID_BTN_DISM_ONLINE,
    ID_BTN_SFC,
    ID_BTN_NET_ALL,
    ID_BTN_NTP_RESYNC,
    ID_BTN_SYSINFO,
    ID_BTN_CHKDSK,
    ID_BTN_DISM_BROWSE_DIR,
    ID_BTN_DISM_BROWSE_IMG,
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

thread_local! {
    static APP_STATE: RefCell<AppState> = RefCell::new(AppState::new());
}

fn with_state<R>(f: impl FnOnce(&AppState) -> R) -> Option<R> {
    APP_STATE.with(|st| st.try_borrow().ok().map(|s| f(&s)))
}

fn with_state_mut<R>(f: impl FnOnce(&mut AppState) -> R) -> Option<R> {
    APP_STATE.with(|st| st.try_borrow_mut().ok().map(|mut s| f(&mut s)))
}

fn ctrl(hwnd: HWND, id: usize) -> HWND {
    unsafe { GetDlgItem(hwnd, id as i32) }
}

fn scale(px: i32, dpi: u32) -> i32 {
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

fn create_fonts_for_dpi(dpi: u32) -> (HFONT, HFONT) {
    let height = -scale(14, dpi);
    with_wide("Yu Gothic UI", |ui_face| {
        with_wide("Consolas", |mono_face| unsafe {
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

fn apply_fonts(hwnd: HWND, hfont_ui: HFONT, hfont_mono: HFONT) {
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

fn layout_controls(hwnd: HWND, dpi: u32) {
    let mut rc = RECT::default();
    unsafe {
        GetClientRect(hwnd, &raw mut rc);
    }
    let w = (rc.right - rc.left).max(scale(700, dpi));
    let h = (rc.bottom - rc.top).max(scale(520, dpi));

    let pad = scale(12, dpi);
    let gap = scale(8, dpi);
    let row_h = scale(30, dpi);
    let lbl_h = scale(20, dpi);
    let inner_w = w - pad * 2;
    let mut y = pad;

    // 1. 状態表示 + 実行中断
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
    y += row_h + gap + scale(4, dpi);

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

unsafe extern "system" fn log_edit_subclass_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
    u_id_subclass: UINT_PTR,
    _dw_ref_data: DWORD_PTR,
) -> LRESULT {
    match msg {
        WM_GETDLGCODE => {
            // 複数行EDITは既定で DLGC_WANTALLKEYS / DLGC_WANTTAB を返すため、
            // これを除外して IsDialogMessageW の Tab / Shift+Tab 移動を有効化する。
            let code = unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) };
            code & !(DLGC_WANTALLKEYS | DLGC_WANTTAB)
        }
        WM_KEYDOWN => {
            let ctrl_down = unsafe { (GetKeyState(VK_CONTROL) as u16) & 0x8000 } != 0;
            if ctrl_down && wparam == b'A' as usize {
                unsafe {
                    SendMessageW(hwnd, EM_SETSEL, 0, -1);
                }
                return 0;
            }
            if wparam == VK_ESCAPE {
                let parent = unsafe { GetParent(hwnd) };
                if !parent.is_null() {
                    let h_first = ctrl(parent, ID_BTN_DISM_ONLINE);
                    let target = if !h_first.is_null() && unsafe { IsWindowEnabled(h_first) } != 0 {
                        h_first
                    } else {
                        ctrl(parent, ID_BTN_CANCEL)
                    };
                    if !target.is_null() && unsafe { IsWindowEnabled(target) } != 0 {
                        unsafe {
                            SetFocus(target);
                        }
                        return 0;
                    }
                }
            }
            unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
        }
        WM_CHAR => {
            // Ctrl+A の制御文字 (0x01) による警告音を抑制する
            if wparam == 0x01 {
                return 0;
            }
            unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
        }
        WM_NCDESTROY => unsafe {
            RemoveWindowSubclass(hwnd, log_edit_subclass_proc, u_id_subclass);
            DefSubclassProc(hwnd, msg, wparam, lparam)
        },
        _ => unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) },
    }
}

fn append_log_line(hwnd: HWND, line: &str) {
    with_state_mut(|s| {
        s.log_buffer.push_str(line);
        s.log_buffer.push_str("\r\n");
    });

    let h_log = ctrl(hwnd, ID_EDIT_LOG);
    if h_log.is_null() {
        return;
    }
    let with_nl = format!("{line}\r\n");
    with_wide(&with_nl, |wide_ptr| unsafe {
        let focused = GetFocus() == h_log;
        let mut sel_start: u32 = 0;
        let mut sel_end: u32 = 0;
        if focused {
            SendMessageW(
                h_log,
                EM_GETSEL,
                (&raw mut sel_start) as WPARAM,
                (&raw mut sel_end) as LPARAM,
            );
        }
        let len = GetWindowTextLengthW(h_log) as WPARAM;
        SendMessageW(h_log, EM_SETSEL, len, len as LPARAM);
        SendMessageW(h_log, EM_REPLACESEL, 0, wide_ptr as LPARAM);
        if focused {
            SendMessageW(h_log, EM_SETSEL, sel_start as WPARAM, sel_end as LPARAM);
        }
    });
}

fn set_running_ui_state(hwnd: HWND, running: bool, status_text: &str) {
    let h_status = ctrl(hwnd, ID_LBL_STATUS);
    if !h_status.is_null() {
        set_window_text(h_status, status_text);
    }

    let h_cancel = ctrl(hwnd, ID_BTN_CANCEL);
    unsafe {
        if running {
            if !h_cancel.is_null() {
                EnableWindow(h_cancel, TRUE);
                let focused = GetFocus();
                let on_action_btn = ACTION_BUTTON_IDS
                    .iter()
                    .any(|&id| ctrl(hwnd, id) == focused);
                if on_action_btn {
                    with_state_mut(|s| s.last_focused_ctrl = focused);
                    SetFocus(h_cancel);
                }
            }
            for &id in ACTION_BUTTON_IDS {
                let h = ctrl(hwnd, id);
                if !h.is_null() {
                    EnableWindow(h, FALSE);
                }
            }
        } else {
            for &id in ACTION_BUTTON_IDS {
                let h = ctrl(hwnd, id);
                if !h.is_null() {
                    EnableWindow(h, TRUE);
                }
            }
            if !h_cancel.is_null() {
                if GetFocus() == h_cancel {
                    let restore_target = with_state(|s| s.last_focused_ctrl)
                        .filter(|&h| !h.is_null() && IsWindowEnabled(h) != 0)
                        .unwrap_or_else(|| ctrl(hwnd, ID_BTN_DISM_ONLINE));
                    if !restore_target.is_null() {
                        SetFocus(restore_target);
                    }
                }
                EnableWindow(h_cancel, FALSE);
            }
        }
    }
}

fn start_action(hwnd: HWND, action: MaintenanceAction) {
    let already_running = with_state(|s| s.is_running()).unwrap_or(true);
    if already_running {
        show_info(
            hwnd,
            "実行中",
            "別の処理が実行中です。完了または中断をお待ちください。",
        );
        return;
    }

    let steps = match build_command_steps(&action) {
        Ok(s) => s,
        Err(err) => {
            show_error(hwnd, "入力エラー", &err);
            return;
        }
    };

    if let Some(msg) = action.confirmation_message()
        && !confirm_yes_no(hwnd, "実行確認", &msg)
    {
        return;
    }

    append_log_line(hwnd, &format!("=== {} 開始 ===", action.title()));
    set_running_ui_state(hwnd, true, &format!("状態: 実行中 ({})", action.title()));

    let Some(queue) = with_state(|s| s.event_queue.clone()) else {
        return;
    };
    let handle = spawn_command_sequence(steps, hwnd, queue);

    with_state_mut(|s| s.worker = Some(handle));
}

fn process_worker_events(hwnd: HWND) {
    let Some(queue) = with_state(|s| s.event_queue.clone()) else {
        return;
    };

    let events: Vec<WorkerEvent> = if let Ok(mut q) = queue.lock() {
        q.drain(..).collect()
    } else {
        Vec::new()
    };

    for ev in events {
        match ev {
            WorkerEvent::LogLine(line) => {
                append_log_line(hwnd, &line);
            }
            WorkerEvent::Finished {
                cancelled,
                last_exit_code,
            } => {
                with_state_mut(|s| {
                    s.worker = None;
                });
                let msg = if cancelled {
                    "=== ユーザーにより中断されました ===".to_string()
                } else if let Some(code) = last_exit_code {
                    format!("=== 完了 (終了コード: {code}) ===")
                } else {
                    "=== 完了 ===".to_string()
                };
                append_log_line(hwnd, &msg);
                set_running_ui_state(hwnd, false, "状態: 待機中");
            }
            WorkerEvent::Error(err) => {
                with_state_mut(|s| {
                    s.worker = None;
                });
                append_log_line(hwnd, &format!("[エラー] {err}"));
                set_running_ui_state(hwnd, false, "状態: エラー終了");
                show_error(hwnd, "実行エラー", &err);
            }
        }
    }
}

fn get_selected_combo_text(hwnd: HWND, id: usize, fallback: &str) -> String {
    let h = ctrl(hwnd, id);
    if h.is_null() {
        return fallback.to_string();
    }
    let idx = unsafe { SendMessageW(h, CB_GETCURSEL, 0, 0) };
    if idx < 0 {
        return fallback.to_string();
    }
    let text = get_window_text(h);
    if text.is_empty() {
        fallback.to_string()
    } else {
        text
    }
}

fn handle_command(hwnd: HWND, id: usize, code: u16) {
    if code != BN_CLICKED {
        return;
    }

    match id {
        ID_BTN_CANCEL => {
            with_state(|s| {
                if let Some(ref w) = s.worker {
                    w.request_stop();
                }
            });
            append_log_line(hwnd, "[情報] 中断リクエストを送信しました...");
        }
        ID_BTN_LOG_CLEAR => {
            with_state_mut(|s| {
                s.log_buffer.clear();
            });
            let h_log = ctrl(hwnd, ID_EDIT_LOG);
            if !h_log.is_null() {
                set_window_text(h_log, "");
            }
        }
        ID_BTN_LOG_SAVE => {
            let content = with_state(|s| s.log_buffer.clone()).unwrap_or_default();
            if let Some(path) = save_log_file(hwnd) {
                match save_text_utf8(&path, &content) {
                    Ok(()) => show_info(hwnd, "保存完了", "実行ログを UTF-8 で保存しました。"),
                    Err(e) => show_error(hwnd, "保存エラー", &e),
                }
            }
        }
        ID_BTN_DISM_ONLINE => start_action(hwnd, MaintenanceAction::DismRestoreHealth),
        ID_BTN_SFC => start_action(hwnd, MaintenanceAction::SfcScannow),
        ID_BTN_NET_ALL => start_action(hwnd, MaintenanceAction::NetFullRepair),
        ID_BTN_NTP_RESYNC => start_action(hwnd, MaintenanceAction::NtpResync),
        ID_BTN_SYSINFO => start_action(hwnd, MaintenanceAction::SystemInfo),
        ID_BTN_CHKDSK => {
            let drive = get_selected_combo_text(hwnd, ID_CMB_CHKDSK_DRIVE, "C:");
            start_action(hwnd, MaintenanceAction::ChkdskRepair { drive });
        }
        ID_BTN_DISM_BROWSE_DIR => {
            if let Some(path) = browse_folder(hwnd, "DISM 修復ソースフォルダを選択") {
                set_window_text(ctrl(hwnd, ID_EDIT_DISM_SOURCE), &path.to_string_lossy());
            }
        }
        ID_BTN_DISM_BROWSE_IMG => {
            if let Some(path) = open_image_file(hwnd) {
                set_window_text(ctrl(hwnd, ID_EDIT_DISM_SOURCE), &path.to_string_lossy());
            }
        }
        ID_BTN_DISM_SOURCE => {
            let src = get_window_text(ctrl(hwnd, ID_EDIT_DISM_SOURCE));
            let idx_str = get_selected_combo_text(hwnd, ID_CMB_DISM_INDEX, "1");
            let idx: u32 = idx_str.trim().parse().unwrap_or(1);
            start_action(
                hwnd,
                MaintenanceAction::DismRestoreHealthWithSource {
                    source: src,
                    index: idx,
                },
            );
        }
        _ => {}
    }
}

fn create_controls(hwnd: HWND, hinst: HINSTANCE) {
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
        SetWindowSubclass(h_log, log_edit_subclass_proc, LOG_SUBCLASS_ID, 0);
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let res = std::panic::catch_unwind(|| match msg {
        WM_CREATE => {
            let hinst = unsafe { GetModuleHandleW(std::ptr::null()) };
            create_controls(hwnd, hinst);

            let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
            let (hfont_ui, hfont_mono) = create_fonts_for_dpi(dpi);

            with_state_mut(|s| {
                s.current_dpi = dpi;
                s.hfont_ui = hfont_ui;
                s.hfont_mono = hfont_mono;
            });

            apply_fonts(hwnd, hfont_ui, hfont_mono);
            set_running_ui_state(hwnd, false, "状態: 待機中");
            layout_controls(hwnd, dpi);

            if let Some(exe_dir) = get_exe_directory()
                && let Some(img_path) = detect_portable_image_in_dir(&exe_dir)
            {
                let path_str = img_path.to_string_lossy().into_owned();
                set_window_text(ctrl(hwnd, ID_EDIT_DISM_SOURCE), &path_str);
                append_log_line(
                    hwnd,
                    &format!("[情報] ポータブル修復イメージを検出しました: {path_str}"),
                );
            }
            0
        }
        WM_SIZE => {
            let dpi = with_state(|s| s.current_dpi).unwrap_or(96).max(96);
            layout_controls(hwnd, dpi);
            0
        }
        WM_DPICHANGED => {
            let new_dpi = (wparam & 0xFFFF) as u32;
            let dpi = new_dpi.max(96);
            let (hfont_ui, hfont_mono) = create_fonts_for_dpi(dpi);
            with_state_mut(|s| {
                s.current_dpi = dpi;
                s.release_fonts();
                s.hfont_ui = hfont_ui;
                s.hfont_mono = hfont_mono;
            });
            apply_fonts(hwnd, hfont_ui, hfont_mono);
            if lparam != 0 {
                let rect = unsafe { &*(lparam as *const RECT) };
                unsafe {
                    MoveWindow(
                        hwnd,
                        rect.left,
                        rect.top,
                        rect.right - rect.left,
                        rect.bottom - rect.top,
                        TRUE,
                    );
                }
            }
            layout_controls(hwnd, dpi);
            0
        }
        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            let code = ((wparam >> 16) & 0xFFFF) as u16;
            handle_command(hwnd, id, code);
            0
        }
        WM_APP_WORKER_EVENT => {
            process_worker_events(hwnd);
            0
        }
        WM_CLOSE => {
            let is_running = with_state(|s| s.is_running()).unwrap_or(false);
            if is_running
                && !confirm_yes_no(
                    hwnd,
                    "終了確認",
                    "現在コマンドを実行中です。中断して終了してもよろしいですか？",
                )
            {
                return 0;
            }
            unsafe {
                DestroyWindow(hwnd);
            }
            0
        }
        WM_DESTROY => {
            with_state_mut(|s| {
                s.worker.take();
                s.release_fonts();
            });
            unsafe {
                PostQuitMessage(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    });

    res.unwrap_or_else(|_| unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) })
}

pub fn run_app() -> i32 {
    unsafe {
        let icc = INITCOMMONCONTROLSEX {
            dwSize: std::mem::size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_STANDARD_CLASSES,
        };
        InitCommonControlsEx(&raw const icc);

        let hinst = GetModuleHandleW(std::ptr::null());
        let hwnd = with_wide("WMKMainWindowClass", |class_ptr| {
            with_wide(
                concat!("Windows Maintenance Kit (WMK) v", env!("CARGO_PKG_VERSION")),
                |title_ptr| {
                    let wc = WNDCLASSEXW {
                        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                        lpfnWndProc: Some(wnd_proc),
                        hInstance: hinst,
                        hIcon: LoadIconW(
                            std::ptr::null_mut(),
                            make_int_resource_w(IDI_APPLICATION),
                        ),
                        hCursor: LoadCursorW(std::ptr::null_mut(), make_int_resource_w(IDC_ARROW)),
                        hbrBackground: (COLOR_BTNFACE + 1) as HBRUSH,
                        lpszClassName: class_ptr,
                        ..Default::default()
                    };

                    if RegisterClassExW(&raw const wc) == 0 {
                        return std::ptr::null_mut();
                    }

                    CreateWindowExW(
                        0,
                        class_ptr,
                        title_ptr,
                        WS_OVERLAPPEDWINDOW,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        820,
                        620,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        hinst,
                        std::ptr::null(),
                    )
                },
            )
        });
        if hwnd.is_null() {
            return 1;
        }

        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
        let h_first = ctrl(hwnd, ID_BTN_DISM_ONLINE);
        if !h_first.is_null() {
            SetFocus(h_first);
        }

        let mut msg = MSG::default();
        while GetMessageW(&raw mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            if IsDialogMessageW(hwnd, &raw const msg) == 0 {
                TranslateMessage(&raw const msg);
                DispatchMessageW(&raw const msg);
            }
        }
        msg.wParam as i32
    }
}
