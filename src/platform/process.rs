use crate::domain::commands::CommandStep;
use crate::domain::decode::StreamLineDecoder;
use crate::platform::win32::{
    ES_CONTINUOUS, ES_SYSTEM_REQUIRED, GetSystemDirectoryW, HANDLE, HWND, INFINITE, PostMessageW,
    SetThreadExecutionState, WM_APP, WaitForSingleObject, decode_cp932,
};
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::os::windows::io::AsRawHandle;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::JoinHandle;

pub const WM_APP_WORKER_EVENT: u32 = WM_APP + 1;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const MAX_QUEUE_LEN: usize = 5000;

#[derive(Debug, Clone)]
pub enum WorkerEvent {
    LogLine(String),
    Finished {
        cancelled: bool,
        last_exit_code: Option<i32>,
    },
    Error(String),
}

pub fn resolve_system32_dir() -> Result<PathBuf, String> {
    let mut buf = [0u16; 260];
    let len = unsafe { GetSystemDirectoryW(buf.as_mut_ptr(), buf.len() as u32) };
    if len > 0 && (len as usize) < buf.len() {
        let sys_dir = String::from_utf16_lossy(&buf[..len as usize]);
        Ok(PathBuf::from(sys_dir))
    } else {
        Err("System32 ディレクトリのパス取得に失敗しました。".to_string())
    }
}

pub struct WorkerHandle {
    child: Arc<Mutex<Option<Child>>>,
    cancelled: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>,
}

impl WorkerHandle {
    pub fn request_stop(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        if let Ok(mut guard) = self.child.lock()
            && let Some(ref mut ch) = *guard
        {
            let _ = ch.kill();
        }
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        self.request_stop();
        // UIスレッドをブロックしないよう join は待機せず切り離す
        self.join_handle.take();
    }
}

fn push_event(queue: &Arc<Mutex<VecDeque<WorkerEvent>>>, hwnd_raw: usize, ev: WorkerEvent) {
    if let Ok(mut q) = queue.lock() {
        if q.len() >= MAX_QUEUE_LEN {
            q.pop_front();
        }
        q.push_back(ev);
    }
    unsafe {
        PostMessageW(hwnd_raw as HWND, WM_APP_WORKER_EVENT, 0, 0);
    }
}

fn spawn_pipe_reader<R: Read + Send + 'static>(
    mut reader: R,
    queue: Arc<Mutex<VecDeque<WorkerEvent>>>,
    hwnd_raw: usize,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        let mut decoder = StreamLineDecoder::new(decode_cp932);
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    for line in decoder.push_bytes(&buf[..n]) {
                        push_event(&queue, hwnd_raw, WorkerEvent::LogLine(line));
                    }
                }
                Err(_) => break,
            }
        }
        if let Some(last) = decoder.finish() {
            push_event(&queue, hwnd_raw, WorkerEvent::LogLine(last));
        }
    })
}

/// 指定されたコマンドステップ列をバックグラウンドスレッドで順次実行する。
pub fn spawn_command_sequence(
    steps: Vec<CommandStep>,
    hwnd: HWND,
    queue: Arc<Mutex<VecDeque<WorkerEvent>>>,
) -> WorkerHandle {
    let hwnd_raw = hwnd as usize;
    let child_arc: Arc<Mutex<Option<Child>>> = Arc::new(Mutex::new(None));
    let cancelled = Arc::new(AtomicBool::new(false));

    let child_ref = Arc::clone(&child_arc);
    let cancel_ref = Arc::clone(&cancelled);

    let join_handle = std::thread::spawn(move || {
        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS | ES_SYSTEM_REQUIRED);
        }

        let sys32 = match resolve_system32_dir() {
            Ok(dir) => dir,
            Err(e) => {
                push_event(&queue, hwnd_raw, WorkerEvent::Error(e));
                unsafe {
                    SetThreadExecutionState(ES_CONTINUOUS);
                }
                return;
            }
        };

        let mut last_exit_code: Option<i32> = None;
        let mut failed = false;

        for step in steps {
            if cancel_ref.load(Ordering::SeqCst) {
                break;
            }

            push_event(
                &queue,
                hwnd_raw,
                WorkerEvent::LogLine(format!("[実行] {}", step.display_label)),
            );

            let exe_path = sys32.join(step.exe_name);
            let mut cmd = Command::new(&exe_path);
            cmd.args(&step.args);
            cmd.creation_flags(CREATE_NO_WINDOW);
            if step.stdin_response.is_some() {
                cmd.stdin(Stdio::piped());
            } else {
                cmd.stdin(Stdio::null());
            }
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let mut spawned = match cmd.spawn() {
                Ok(ch) => ch,
                Err(e) => {
                    push_event(
                        &queue,
                        hwnd_raw,
                        WorkerEvent::Error(format!(
                            "プロセス起動失敗 ({}): {e}",
                            exe_path.display()
                        )),
                    );
                    failed = true;
                    break;
                }
            };

            if let Some(resp) = step.stdin_response
                && let Some(mut stdin) = spawned.stdin.take()
            {
                let _ = stdin.write_all(resp.as_bytes());
                let _ = stdin.flush();
            }

            let stdout_handle = spawned
                .stdout
                .take()
                .map(|r| spawn_pipe_reader(r, Arc::clone(&queue), hwnd_raw));
            let stderr_handle = spawned
                .stderr
                .take()
                .map(|r| spawn_pipe_reader(r, Arc::clone(&queue), hwnd_raw));

            let raw_handle = spawned.as_raw_handle() as HANDLE;
            if let Ok(mut guard) = child_ref.lock() {
                if cancel_ref.load(Ordering::SeqCst) {
                    let _ = spawned.kill();
                }
                *guard = Some(spawned);
            }

            unsafe {
                WaitForSingleObject(raw_handle, INFINITE);
            }

            let status = child_ref
                .lock()
                .ok()
                .and_then(|mut g| g.take())
                .and_then(|mut ch| ch.wait().ok());
            if let Some(h) = stdout_handle {
                let _ = h.join();
            }
            if let Some(h) = stderr_handle {
                let _ = h.join();
            }

            last_exit_code = status.and_then(|s| s.code());
            if let Some(code) = last_exit_code {
                push_event(
                    &queue,
                    hwnd_raw,
                    WorkerEvent::LogLine(format!("[終了コード] {code}")),
                );
            }
        }

        unsafe {
            SetThreadExecutionState(ES_CONTINUOUS);
        }

        if !failed {
            push_event(
                &queue,
                hwnd_raw,
                WorkerEvent::Finished {
                    cancelled: cancel_ref.load(Ordering::SeqCst),
                    last_exit_code,
                },
            );
        }
    });

    WorkerHandle {
        child: child_arc,
        cancelled,
        join_handle: Some(join_handle),
    }
}
