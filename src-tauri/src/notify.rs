//! Windows 原生 Toast — 主窗口隐藏时推送系统通知

use crate::settings;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;

/// 主窗口不可见且用户启用通知时，发送系统 Toast
pub fn send_if_background(app: &AppHandle, title: &str, body: &str) {
    if !settings::get().native_notifications {
        return;
    }

    let visible = app
        .get_webview_window("main")
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    if visible {
        return;
    }

    let _ = app
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show();
}
