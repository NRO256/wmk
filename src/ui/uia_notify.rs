use crate::platform::win32::{
    GetProcAddress, HWND, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW, S_OK, SysAllocString,
    SysFreeString,
};
use std::ffi::c_void;
use std::sync::OnceLock;

const NOTIFICATION_KIND_ACTION_COMPLETED: i32 = 4;
const NOTIFICATION_PROCESSING_MOST_RECENT: i32 = 3;
const MAX_ANNOUNCE_CHARS: usize = 300;

type UiaClientsAreListeningFn = unsafe extern "system" fn() -> i32;
type UiaHostProviderFromHwndFn =
    unsafe extern "system" fn(hwnd: HWND, pp_provider: *mut *mut c_void) -> i32;
type UiaRaiseNotificationEventFn = unsafe extern "system" fn(
    provider: *mut c_void,
    notification_kind: i32,
    notification_processing: i32,
    display_string: *const u16,
    activity_id: *const u16,
) -> i32;

struct UiaApi {
    clients_are_listening: UiaClientsAreListeningFn,
    host_provider_from_hwnd: UiaHostProviderFromHwndFn,
    raise_notification_event: UiaRaiseNotificationEventFn,
}

static UIA_API: OnceLock<Option<UiaApi>> = OnceLock::new();

fn load_uia_api() -> Option<&'static UiaApi> {
    UIA_API
        .get_or_init(|| {
            let dll_w: Vec<u16> = "UIAutomationCore.dll\0".encode_utf16().collect();
            let hmod = unsafe {
                LoadLibraryExW(
                    dll_w.as_ptr(),
                    std::ptr::null_mut(),
                    LOAD_LIBRARY_SEARCH_SYSTEM32,
                )
            };
            if hmod.is_null() {
                return None;
            }
            unsafe {
                let p_listening = GetProcAddress(hmod, c"UiaClientsAreListening".as_ptr());
                let p_host = GetProcAddress(hmod, c"UiaHostProviderFromHwnd".as_ptr());
                let p_raise = GetProcAddress(hmod, c"UiaRaiseNotificationEvent".as_ptr());
                if p_listening.is_null() || p_host.is_null() || p_raise.is_null() {
                    return None;
                }
                Some(UiaApi {
                    clients_are_listening: std::mem::transmute::<
                        *mut c_void,
                        UiaClientsAreListeningFn,
                    >(p_listening),
                    host_provider_from_hwnd: std::mem::transmute::<
                        *mut c_void,
                        UiaHostProviderFromHwndFn,
                    >(p_host),
                    raise_notification_event: std::mem::transmute::<
                        *mut c_void,
                        UiaRaiseNotificationEventFn,
                    >(p_raise),
                })
            }
        })
        .as_ref()
}

struct ProviderGuard(*mut c_void);

impl Drop for ProviderGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                let vtbl = *(self.0 as *const *const *const c_void);
                let release: unsafe extern "system" fn(*mut c_void) -> u32 =
                    std::mem::transmute(*vtbl.add(2));
                release(self.0);
            }
        }
    }
}

struct Bstr(*const u16);

impl Bstr {
    fn new(text: &str) -> Option<Self> {
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let ptr = unsafe { SysAllocString(wide.as_ptr()) };
        if ptr.is_null() { None } else { Some(Self(ptr)) }
    }
}

impl Drop for Bstr {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                SysFreeString(self.0);
            }
        }
    }
}

fn format_announcement_text(text: &str) -> String {
    let normalized: String = text
        .chars()
        .map(|ch| match ch {
            '\r' | '\n' | '\t' => ' ',
            c if c.is_control() => ' ',
            c => c,
        })
        .take(MAX_ANNOUNCE_CHARS)
        .collect();
    normalized.trim().to_string()
}

pub(crate) fn announce_uia_notification(hwnd: HWND, text: &str) {
    if hwnd.is_null() {
        return;
    }
    let clean = format_announcement_text(text);
    if clean.is_empty() {
        return;
    }
    let Some(api) = load_uia_api() else {
        return;
    };
    unsafe {
        if (api.clients_are_listening)() == 0 {
            return;
        }
        let mut provider: *mut c_void = std::ptr::null_mut();
        if (api.host_provider_from_hwnd)(hwnd, &mut provider) != S_OK || provider.is_null() {
            return;
        }
        let _guard = ProviderGuard(provider);
        let Some(display) = Bstr::new(&clean) else {
            return;
        };
        let Some(activity_id) = Bstr::new("WMK.StatusNotification") else {
            return;
        };
        let _ = (api.raise_notification_event)(
            provider,
            NOTIFICATION_KIND_ACTION_COMPLETED,
            NOTIFICATION_PROCESSING_MOST_RECENT,
            display.0,
            activity_id.0,
        );
    }
}
