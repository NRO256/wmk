use crate::domain::commands::{MaintenanceAction, trim_log_buffer};
use crate::platform::process::{WorkerEvent, WorkerHandle};
use crate::platform::win32::{DeleteObject, HFONT, HWND};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use std::time::Instant;

pub const MAX_LOG_BUFFER_BYTES: usize = 512 * 1024;

pub struct AppState {
    pub hfont_ui: HFONT,
    pub hfont_mono: HFONT,
    pub last_focused_ctrl: HWND,
    pub worker: Option<WorkerHandle>,
    pub event_queue: Arc<Mutex<VecDeque<WorkerEvent>>>,
    pub notify_pending: Arc<AtomicBool>,
    pub log_buffer: String,
    pub current_dpi: u32,
    pub running_action: Option<MaintenanceAction>,
    pub started_at: Option<Instant>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            hfont_ui: std::ptr::null_mut(),
            hfont_mono: std::ptr::null_mut(),
            last_focused_ctrl: std::ptr::null_mut(),
            worker: None,
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
            notify_pending: Arc::new(AtomicBool::new(false)),
            log_buffer: String::new(),
            current_dpi: 96,
            running_action: None,
            started_at: None,
        }
    }

    pub fn is_running(&self) -> bool {
        self.worker.is_some()
    }

    pub fn append_log(&mut self, line: &str) {
        self.log_buffer.push_str(line);
        self.log_buffer.push_str("\r\n");
        trim_log_buffer(&mut self.log_buffer, MAX_LOG_BUFFER_BYTES);
    }

    pub fn replace_fonts(&mut self, hfont_ui: HFONT, hfont_mono: HFONT) -> (HFONT, HFONT) {
        let old_ui = std::mem::replace(&mut self.hfont_ui, hfont_ui);
        let old_mono = std::mem::replace(&mut self.hfont_mono, hfont_mono);
        (old_ui, old_mono)
    }

    pub fn release_fonts(&mut self) {
        unsafe {
            if !self.hfont_ui.is_null() {
                DeleteObject(self.hfont_ui);
                self.hfont_ui = std::ptr::null_mut();
            }
            if !self.hfont_mono.is_null() {
                DeleteObject(self.hfont_mono);
                self.hfont_mono = std::ptr::null_mut();
            }
        }
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        self.worker.take();
        self.release_fonts();
    }
}
