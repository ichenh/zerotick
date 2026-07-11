//! 后台诊断引擎编排 — 启动 monitor / bluetooth / bsod

use crate::{bluetooth, bsod, monitor};
use tauri::AppHandle;

/// 启动全部后台诊断引擎（Win32 消息泵 + WMI 轮询 + BSOD 启动扫描）
pub fn start(app: &AppHandle) -> Result<(), String> {
    monitor::spawn(app.clone()).map_err(|e| format!("硬件监控启动失败: {e}"))?;

    let app_bluetooth = app.clone();
    tauri::async_runtime::spawn(async move {
        bluetooth::run_monitor(app_bluetooth).await;
    });

    let app_bsod = app.clone();
    tauri::async_runtime::spawn(async move {
        let handle = app_bsod.clone();
        let _ = tokio::task::spawn_blocking(move || bsod::startup_scan_emit(&handle)).await;
    });

    crate::utils::logging::info("诊断引擎已全部启动（monitor + bluetooth + bsod）");
    Ok(())
}
