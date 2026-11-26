use crate::platform::process::{WorkerEvent, WorkerHandle};
use crate::platform::win32::{DeleteObject, HFONT, HWND};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, atomic::AtomicBool};

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
        }
    }

    pub fn is_running(&self) -> bool {
        self.worker.is_some()
    }

    pub fn append_log(&mut self, line: &str) {
        self.log_buffer.push_str(line);
        self.log_buffer.push_str("\r\n");
        if self.log_buffer.len() > MAX_LOG_BUFFER_BYTES {
            let excess = self.log_buffer.len() - MAX_LOG_BUFFER_BYTES;
            let mut cut_pos = excess;
            while cut_pos < self.log_buffer.len() && !self.log_buffer.is_char_boundary(cut_pos) {
                cut_pos += 1;
            }
            if let Some(rel_nl) = self.log_buffer[cut_pos..].find('\n') {
                cut_pos += rel_nl + 1;
            }
            self.log_buffer.drain(..cut_pos);
        }
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
