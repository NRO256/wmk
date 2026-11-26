pub mod dialogs;
pub mod layout;
pub mod state;

use crate::domain::commands::{MaintenanceAction, build_command_steps};
use crate::domain::source_detect::detect_portable_image_in_dir;
use crate::platform::io::{get_exe_directory, save_text_utf8};
use crate::platform::process::{WM_APP_WORKER_EVENT, WorkerEvent, spawn_command_sequence};
use crate::platform::win32::{
    AdjustWindowRectExForDpi, BN_CLICKED, CB_GETCURSEL, COLOR_BTNFACE, CW_USEDEFAULT,
    CreateWindowExW, DLGC_WANTALLKEYS, DLGC_WANTTAB, DWORD_PTR, DefSubclassProc, DefWindowProcW,
    DeleteObject, DestroyWindow, DispatchMessageW, EM_GETFIRSTVISIBLELINE, EM_GETSEL,
    EM_LINEFROMCHAR, EM_LINEINDEX, EM_LINESCROLL, EM_REPLACESEL, EM_SETSEL, EnableWindow, FALSE,
    GetDpiForWindow, GetFocus, GetKeyState, GetMessageW, GetModuleHandleW, GetParent,
    GetWindowTextLengthW, HBRUSH, HWND, ICC_STANDARD_CLASSES, IDC_ARROW, IDI_APPLICATION,
    INITCOMMONCONTROLSEX, InitCommonControlsEx, InvalidateRect, IsDialogMessageW, IsWindowEnabled,
    LPARAM, LRESULT, LoadCursorW, LoadIconW, MINMAXINFO, MSG, MoveWindow, PostQuitMessage, RECT,
    RegisterClassExW, RemoveWindowSubclass, SW_SHOW, SendMessageW, SetFocus, ShowWindow, TRUE,
    TranslateMessage, UINT, UINT_PTR, UpdateWindow, VK_CONTROL, VK_ESCAPE, WM_CHAR, WM_CLOSE,
    WM_COMMAND, WM_CREATE, WM_DESTROY, WM_DPICHANGED, WM_GETDLGCODE, WM_GETMINMAXINFO, WM_KEYDOWN,
    WM_NCDESTROY, WM_SETREDRAW, WM_SIZE, WNDCLASSEXW, WPARAM, WS_OVERLAPPEDWINDOW, get_window_text,
    make_int_resource_w, set_window_text, with_wide,
};
use crate::ui::dialogs::{
    browse_folder, confirm_yes_no, open_image_file, save_log_file, show_error, show_info,
};
use crate::ui::layout::{
    ACTION_BUTTON_IDS, ID_BTN_CANCEL, ID_BTN_CHKDSK, ID_BTN_DISM_BROWSE_DIR,
    ID_BTN_DISM_BROWSE_IMG, ID_BTN_DISM_ONLINE, ID_BTN_DISM_SOURCE, ID_BTN_LOG_CLEAR,
    ID_BTN_LOG_SAVE, ID_BTN_NET_ALL, ID_BTN_NTP_RESYNC, ID_BTN_SFC, ID_BTN_SYSINFO,
    ID_CMB_CHKDSK_DRIVE, ID_CMB_DISM_INDEX, ID_EDIT_DISM_SOURCE, ID_EDIT_LOG, ID_LBL_STATUS,
    MIN_CLIENT_HEIGHT, MIN_CLIENT_WIDTH, apply_fonts, create_controls, create_fonts_for_dpi, ctrl,
    layout_controls, scale,
};
use crate::ui::state::AppState;
use std::cell::RefCell;
use std::sync::atomic::Ordering;

const MAX_LOG_EDIT_CHARS: usize = 512 * 1024;

thread_local! {
    static APP_STATE: RefCell<AppState> = RefCell::new(AppState::new());
}

fn with_state<R>(f: impl FnOnce(&AppState) -> R) -> Option<R> {
    APP_STATE.with(|st| st.try_borrow().ok().map(|s| f(&s)))
}

fn with_state_mut<R>(f: impl FnOnce(&mut AppState) -> R) -> Option<R> {
    APP_STATE.with(|st| st.try_borrow_mut().ok().map(|mut s| f(&mut s)))
}

pub(super) unsafe extern "system" fn log_edit_subclass_proc(
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
        s.append_log(line);
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
        let mut first_visible: isize = 0;
        if focused {
            SendMessageW(
                h_log,
                EM_GETSEL,
                (&raw mut sel_start) as WPARAM,
                (&raw mut sel_end) as LPARAM,
            );
            first_visible = SendMessageW(h_log, EM_GETFIRSTVISIBLELINE, 0, 0);
            SendMessageW(h_log, WM_SETREDRAW, FALSE as WPARAM, 0);
        }

        let len = GetWindowTextLengthW(h_log) as WPARAM;
        SendMessageW(h_log, EM_SETSEL, len, len as LPARAM);
        SendMessageW(h_log, EM_REPLACESEL, 0, wide_ptr as LPARAM);

        let mut removed_lines: isize = 0;
        let total_len = GetWindowTextLengthW(h_log).max(0) as usize;
        if total_len > MAX_LOG_EDIT_CHARS {
            let excess = total_len - MAX_LOG_EDIT_CHARS;
            let next_line = SendMessageW(h_log, EM_LINEFROMCHAR, excess, 0) + 1;
            let line_idx = SendMessageW(h_log, EM_LINEINDEX, next_line as WPARAM, 0);
            let cut_idx = if line_idx > 0 {
                removed_lines = next_line;
                line_idx as usize
            } else {
                excess
            };
            if cut_idx > 0 {
                SendMessageW(h_log, EM_SETSEL, 0, cut_idx as LPARAM);
                with_wide("", |empty_ptr| {
                    SendMessageW(h_log, EM_REPLACESEL, 0, empty_ptr as LPARAM);
                });
                let cut_u32 = u32::try_from(cut_idx).unwrap_or(u32::MAX);
                sel_start = sel_start.saturating_sub(cut_u32);
                sel_end = sel_end.saturating_sub(cut_u32);
            }
        }

        if focused {
            SendMessageW(h_log, EM_SETSEL, sel_start as WPARAM, sel_end as LPARAM);
            let current_first = SendMessageW(h_log, EM_GETFIRSTVISIBLELINE, 0, 0);
            let target_first = (first_visible - removed_lines).max(0);
            let delta = target_first - current_first;
            if delta != 0 {
                SendMessageW(h_log, EM_LINESCROLL, 0, delta as LPARAM);
            }
            SendMessageW(h_log, WM_SETREDRAW, TRUE as WPARAM, 0);
            InvalidateRect(h_log, std::ptr::null(), FALSE);
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

    let Some((queue, notify_pending)) =
        with_state(|s| (s.event_queue.clone(), s.notify_pending.clone()))
    else {
        return;
    };
    let handle = spawn_command_sequence(steps, hwnd, queue, notify_pending);

    with_state_mut(|s| s.worker = Some(handle));
}

fn process_worker_events(hwnd: HWND) {
    let Some((queue, notify_pending)) =
        with_state(|s| (s.event_queue.clone(), s.notify_pending.clone()))
    else {
        return;
    };

    let events: Vec<WorkerEvent> = if let Ok(mut q) = queue.lock() {
        notify_pending.store(false, Ordering::Release);
        q.drain(..).collect()
    } else {
        notify_pending.store(false, Ordering::Release);
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
        WM_GETMINMAXINFO => {
            if lparam != 0 {
                let dpi = with_state(|s| s.current_dpi).unwrap_or(96).max(96);
                let mut rc = RECT {
                    left: 0,
                    top: 0,
                    right: scale(MIN_CLIENT_WIDTH, dpi),
                    bottom: scale(MIN_CLIENT_HEIGHT, dpi),
                };
                let (min_w, min_h) = unsafe {
                    if AdjustWindowRectExForDpi(&raw mut rc, WS_OVERLAPPEDWINDOW, FALSE, 0, dpi)
                        != 0
                    {
                        (rc.right - rc.left, rc.bottom - rc.top)
                    } else {
                        (scale(MIN_CLIENT_WIDTH, dpi), scale(MIN_CLIENT_HEIGHT, dpi))
                    }
                };
                let mmi = unsafe { &mut *(lparam as *mut MINMAXINFO) };
                mmi.ptMinTrackSize.x = min_w;
                mmi.ptMinTrackSize.y = min_h;
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
            let old_fonts = with_state_mut(|s| {
                s.current_dpi = dpi;
                s.replace_fonts(hfont_ui, hfont_mono)
            });
            apply_fonts(hwnd, hfont_ui, hfont_mono);
            if let Some((old_ui, old_mono)) = old_fonts {
                unsafe {
                    if !old_ui.is_null() {
                        DeleteObject(old_ui);
                    }
                    if !old_mono.is_null() {
                        DeleteObject(old_mono);
                    }
                }
            }
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
