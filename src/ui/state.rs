use crate::platform::process::{WorkerEvent, WorkerHandle};
use crate::platform::win32::{DeleteObject, HFONT, HWND};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub hfont_ui: HFONT,
    pub hfont_mono: HFONT,
    pub last_focused_ctrl: HWND,
    pub worker: Option<WorkerHandle>,
    pub event_queue: Arc<Mutex<VecDeque<WorkerEvent>>>,
    pub log_buffer: String,
    pub current_dpi: u32,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            hfont_ui: std::ptr::null_mut(),
            hfont_mono: std::ptr::null_mut(),
            last_focused_ctrl: std::ptr::null_mut(),
            worker: None,
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
            log_buffer: String::new(),
            current_dpi: 96,
        }
    }

    pub fn is_running(&self) -> bool {
        self.worker.is_some()
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
