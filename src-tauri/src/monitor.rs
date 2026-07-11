//! Task 1：硬件与蓝牙断连实时监控 — 捕获 WM_DEVICECHANGE 并通过 Tauri Event 推送

use crate::events::DeviceEvent;
use crate::notify;
use crate::settings;
use crate::tray::{self, TrayLevel};
use crate::utils::device_name;
use crate::utils::device_path::{is_transient_disconnect, parse_device_path, DeviceCategory};
use crate::utils::guid::{DEVINTERFACE_BLUETOOTH, DEVINTERFACE_USB_DEVICE};
use crate::utils::logging;
use chrono::Local;
use std::collections::HashMap;
use std::mem::size_of;
use std::sync::{Mutex, OnceLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, RegisterDeviceNotificationW, TranslateMessage, UnregisterClassW,
    WM_DEVICECHANGE, WM_DESTROY, WNDCLASSW, CS_HREDRAW, CS_VREDRAW, DBT_DEVICEARRIVAL,
    DBT_DEVICEREMOVECOMPLETE, DBT_DEVTYP_DEVICEINTERFACE, DEV_BROADCAST_DEVICEINTERFACE_W,
    DEV_BROADCAST_HDR, DEVICE_NOTIFY_WINDOW_HANDLE, HWND_MESSAGE, WINDOW_EX_STYLE,
    WINDOW_STYLE,
};

const CLASS_NAME: PCWSTR = windows::core::w!("ZeroTickDeviceMonitor");

static TRACKER: OnceLock<Mutex<DisconnectTracker>> = OnceLock::new();
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

struct DisconnectTracker {
    pending: HashMap<String, Instant>,
}

impl DisconnectTracker {
    fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }

    fn record_disconnect(&mut self, path: &str) {
        self.pending.insert(path.to_string(), Instant::now());
    }

    fn record_arrival(&mut self, path: &str) -> Option<std::time::Duration> {
        self.pending.remove(path).map(|t| t.elapsed())
    }
}

pub fn spawn(app: AppHandle) -> windows::core::Result<JoinHandle<()>> {
    let _ = TRACKER.set(Mutex::new(DisconnectTracker::new()));
    let _ = APP_HANDLE.set(app);

    thread::Builder::new()
        .name("zerotick-device-monitor".into())
        .spawn(run_message_loop)
        .map_err(|e| {
            windows::core::Error::new(
                windows::core::HRESULT::from_win32(windows::Win32::Foundation::ERROR_GEN_FAILURE.0),
                format!("无法创建设备监控线程: {e}"),
            )
        })
}

fn emit_device_event(payload: DeviceEvent) {
    if let Some(app) = APP_HANDLE.get() {
        if let Err(e) = app.emit("device-event", &payload) {
            logging::error(format!("emit device-event 失败: {e}"));
        }
    }
}

fn push_event(
    event_type: &str,
    category: DeviceCategory,
    vid_pid: Option<String>,
    device_path: String,
    disconnect_ms: Option<u64>,
    tray_level: TrayLevel,
    tray_reason_id: &str,
) {
    let category_code = category.as_str().to_string();
    let friendly_name = device_name::resolve(&device_path, vid_pid.as_deref());
    let message = build_message(event_type, &category_code, &friendly_name, &vid_pid, disconnect_ms);

    if let Some(app) = APP_HANDLE.get() {
        tray::set_level(app, tray_level, tray_reason_id);
    }

    let event = DeviceEvent {
        timestamp: Local::now().to_rfc3339(),
        event_type: event_type.to_string(),
        category: category_code,
        vid_pid,
        device_path,
        disconnect_ms,
        message,
        friendly_name,
    };

    emit_device_event(event.clone());
    crate::history::append(&event);

    if let Some(app) = APP_HANDLE.get() {
        let locale = settings::get().locale;
        let (title, body) = match event_type {
            "transient_reconnect" => (
                crate::i18n::notify_transient_title(&locale),
                crate::i18n::format_device_notify(&locale, event_type, &event),
            ),
            "remove" => (
                crate::i18n::notify_disconnect_title(&locale),
                crate::i18n::format_device_notify(&locale, event_type, &event),
            ),
            _ => return,
        };
        notify::send_if_background(app, &title, &body);
    }
}

fn build_message(
    event_type: &str,
    category: &str,
    friendly_name: &Option<String>,
    vid_pid: &Option<String>,
    disconnect_ms: Option<u64>,
) -> String {
    let label = friendly_name
        .as_deref()
        .or(vid_pid.as_deref())
        .unwrap_or("unknown");

    match event_type {
        "transient_reconnect" => {
            format!(
                "[transient] [{category}] {label} — {}ms",
                disconnect_ms.unwrap_or(0)
            )
        }
        "arrival" if disconnect_ms.is_some() => {
            format!(
                "[{category}] {label} — reconnect {}ms",
                disconnect_ms.unwrap_or(0)
            )
        }
        "arrival" => format!("[{category}] {label} — arrival"),
        "remove" => format!("[{category}] {label} — remove"),
        _ => format!("[{category}] {label}"),
    }
}

fn run_message_loop() {
    if let Err(e) = run_message_loop_inner() {
        logging::error(format!("设备监控线程异常退出: {e}"));
    }
}

fn run_message_loop_inner() -> windows::core::Result<()> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;
        let wc = WNDCLASSW {
            lpfnWndProc: Some(device_wnd_proc),
            hInstance: hinstance.into(),
            lpszClassName: CLASS_NAME,
            style: CS_HREDRAW | CS_VREDRAW,
            ..Default::default()
        };
        let _ = RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            CLASS_NAME,
            windows::core::w!("ZeroTick Device Monitor"),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(hinstance.into()),
            None,
        )?;

        register_device_notification(hwnd, DEVINTERFACE_USB_DEVICE)?;
        register_device_notification(hwnd, DEVINTERFACE_BLUETOOTH)?;
        logging::info("硬件断连监控已启动（USB + 蓝牙 GUID 已注册）");

        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        UnregisterClassW(CLASS_NAME, Some(hinstance.into()))?;
    }
    Ok(())
}

unsafe fn register_device_notification(
    hwnd: HWND,
    class_guid: windows::core::GUID,
) -> windows::core::Result<()> {
    let filter = DEV_BROADCAST_DEVICEINTERFACE_W {
        dbcc_size: size_of::<DEV_BROADCAST_DEVICEINTERFACE_W>() as u32,
        dbcc_devicetype: DBT_DEVTYP_DEVICEINTERFACE.0,
        dbcc_classguid: class_guid,
        ..Default::default()
    };
    let _ = RegisterDeviceNotificationW(
        HANDLE(hwnd.0),
        std::ptr::from_ref(&filter).cast(),
        DEVICE_NOTIFY_WINDOW_HANDLE,
    )?;
    Ok(())
}

unsafe extern "system" fn device_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DEVICECHANGE => {
            handle_device_change(wparam, lparam);
            LRESULT(0)
        }
        WM_DESTROY => {
            let _ = PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn handle_device_change(wparam: WPARAM, lparam: LPARAM) {
    let event = wparam.0 as u32;
    if event != DBT_DEVICEARRIVAL && event != DBT_DEVICEREMOVECOMPLETE {
        return;
    }
    if lparam.0 == 0 {
        return;
    }

    let hdr = &*(lparam.0 as *const DEV_BROADCAST_HDR);
    if hdr.dbch_devicetype != DBT_DEVTYP_DEVICEINTERFACE {
        return;
    }

    let iface = &*(lparam.0 as *const DEV_BROADCAST_DEVICEINTERFACE_W);
    let path = read_device_interface_path(iface);
    if path.is_empty() {
        return;
    }

    let (category, vid_pid) = parse_device_path(&path);
    let tracker = match TRACKER.get() {
        Some(t) => t,
        None => return,
    };

    match event {
        DBT_DEVICEARRIVAL => {
            let mut guard = match tracker.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            if let Some(elapsed) = guard.record_arrival(&path) {
                let ms = elapsed.as_millis() as u64;
                let threshold =
                    Duration::from_millis(settings::get().transient_threshold_ms);
                if is_transient_disconnect(elapsed, threshold) {
                    logging::critical(format!("[瞬断] {path} — {ms}ms"));
                    push_event(
                        "transient_reconnect",
                        category,
                        vid_pid,
                        path,
                        Some(ms),
                        TrayLevel::Critical,
                        "transient_hw",
                    );
                } else {
                    logging::warn(format!("Reconnect {path} — {ms}ms"));
                    push_event(
                        "arrival",
                        category,
                        vid_pid,
                        path,
                        Some(ms),
                        TrayLevel::Warning,
                        "device_reconnect",
                    );
                }
            } else {
                logging::info(format!("Arrival {path}"));
                push_event(
                    "arrival",
                    category,
                    vid_pid,
                    path,
                    None,
                    TrayLevel::Normal,
                    "device_arrival",
                );
            }
        }
        DBT_DEVICEREMOVECOMPLETE => {
            let mut guard = match tracker.lock() {
                Ok(g) => g,
                Err(_) => return,
            };
            guard.record_disconnect(&path);
            logging::warn(format!("Remove {path}"));
            push_event(
                "remove",
                category,
                vid_pid,
                path,
                None,
                TrayLevel::Warning,
                "device_remove",
            );
        }
        _ => {}
    }
}

unsafe fn read_device_interface_path(iface: &DEV_BROADCAST_DEVICEINTERFACE_W) -> String {
    use windows::core::PCWSTR;
    PCWSTR(iface.dbcc_name.as_ptr())
        .to_string()
        .unwrap_or_default()
}
