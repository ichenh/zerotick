//! Tauri Command 处理器 — 供前端主动调用

use crate::autostart;
use crate::bluetooth;
use crate::bsod;
use crate::events::{BluetoothStatusEvent, BsodAlertEvent, DeviceEvent, RepairCompleteEvent};
use crate::history;
use crate::i18n;
use crate::notify;
use crate::ports::{self, PortScanReport, ReleaseReport};
use crate::repair;
use crate::settings::{self, AppSettings};
use crate::tray::{self, TrayLevel};
use crate::utils;
use chrono::Local;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::DialogExt;

#[derive(Debug, Serialize)]
pub struct RepairResult {
    pub success: bool,
    pub needs_admin: bool,
    pub services_restarted: Vec<String>,
    pub service_errors: Vec<String>,
    pub usb_power_warnings: Vec<String>,
    pub power_scan_error: Option<String>,
    pub summary_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_count: Option<usize>,
}

/// 主动检测蓝牙 / bthserv 状态
#[tauri::command]
pub fn check_bluetooth() -> Result<BluetoothStatusEvent, String> {
    let report = bluetooth::diagnose()?;
    Ok(BluetoothStatusEvent {
        timestamp: Local::now().to_rfc3339(),
        healthy: !report.has_issues(),
        bthserv_state: report.bthserv_state,
        issues: report.issues,
        radio_count: report.radio_devices.len(),
    })
}

/// 扫描最新 Minidump 与 BugCheck 事件
#[tauri::command]
pub fn scan_bsod() -> Result<Option<BsodAlertEvent>, String> {
    let report = bsod::analyze_latest_dump().map_err(|e| format!("BSOD scan failed: {e}"))?;
    Ok(report.map(|r| BsodAlertEvent {
        timestamp: Local::now().to_rfc3339(),
        is_recent: r.is_recent,
        dump_path: r.dump_path.display().to_string(),
        bugcheck_code: r.bugcheck_code,
        faulting_driver: r.faulting_driver,
        message: r.event_message,
    }))
}

/// 一键修复：重启 bthserv / Audiosrv + USB 电源管理扫描
#[tauri::command]
pub fn run_repair(app: AppHandle) -> Result<RepairResult, String> {
    let elevated = utils::elevated::is_elevated();
    let report = repair::run_auto_repair().map_err(|e| format!("Repair failed: {e}"))?;

    let success = report.service_errors.is_empty();
    let needs_admin = !elevated && repair::errors_need_elevation(&report.service_errors);
    let (summary_id, summary_count) = repair::build_summary_meta(success, needs_admin, &report);
    let locale = settings::get().locale;
    let summary_text = i18n::repair_summary(&locale, &summary_id, summary_count);

    let result = RepairResult {
        success,
        needs_admin,
        services_restarted: report.services_restarted.clone(),
        service_errors: report.service_errors.clone(),
        usb_power_warnings: report.usb_hubs_with_power_mgmt.clone(),
        power_scan_error: report.power_scan_error.clone(),
        summary_id: summary_id.clone(),
        summary_count,
    };

    let event = RepairCompleteEvent {
        timestamp: Local::now().to_rfc3339(),
        success,
        needs_admin,
        services_restarted: report.services_restarted,
        service_errors: report.service_errors,
        usb_power_warnings: report.usb_hubs_with_power_mgmt,
        power_scan_error: report.power_scan_error,
        summary_id: summary_id.clone(),
        summary_count,
    };

    let _ = app.emit("repair-complete", &event);
    notify::send_if_background(
        &app,
        &i18n::notify_repair_title(&locale),
        &summary_text,
    );

    if success {
        tray::set_level(&app, TrayLevel::Normal, "repair_done");
    } else if needs_admin {
        tray::set_level(&app, TrayLevel::Warning, "repair_admin");
    } else {
        tray::set_level(&app, TrayLevel::Warning, "repair_partial");
    }

    Ok(result)
}

/// 获取持久化的断连历史
#[tauri::command]
pub fn get_device_history() -> Vec<DeviceEvent> {
    history::list()
}

/// 清空断连历史
#[tauri::command]
pub fn clear_device_history() -> Result<(), String> {
    history::clear()
}

/// 导出断连历史（JSON / CSV），弹出保存对话框
#[tauri::command]
pub fn export_device_history(app: AppHandle, format: String) -> Result<String, String> {
    let locale = settings::get().locale;
    if history::list().is_empty() {
        return Err(i18n::export_error(&locale, "empty"));
    }

    let (content, ext, label) = match format.as_str() {
        "csv" => (history::export_csv()?, "csv", "CSV"),
        _ => (history::export_json()?, "json", "JSON"),
    };

    let stamp = Local::now().format("%Y%m%d_%H%M%S");
    let default_name = format!("zerotick_history_{stamp}.{ext}");

    let path = app
        .dialog()
        .file()
        .set_title(&i18n::export_dialog_title(&locale))
        .set_file_name(&default_name)
        .add_filter(label, &[ext])
        .blocking_save_file();

    let Some(file_path) = path else {
        return Err(i18n::export_error(&locale, "cancelled"));
    };

    let path_buf = file_path
        .into_path()
        .map_err(|e| format!("Invalid export path: {e}"))?;
    std::fs::write(&path_buf, &content).map_err(|e| format!("Write failed: {e}"))?;
    Ok(path_buf.to_string_lossy().into_owned())
}

/// 当前进程是否已提升（管理员）
#[tauri::command]
pub fn is_elevated() -> bool {
    utils::elevated::is_elevated()
}

/// 获取当前设置
#[tauri::command]
pub fn get_settings() -> AppSettings {
    settings::get()
}

/// 保存设置并同步开机自启
#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    let launch = settings.launch_at_startup;
    let saved = settings::save(settings)?;
    autostart::sync(&app, launch)?;
    crate::tray::refresh_locale(&app);
    Ok(saved)
}

/// 应用版本号
#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 扫描本地端口占用（含 Windows 保留段）
#[tauri::command]
pub fn scan_ports() -> Result<PortScanReport, String> {
    ports::scan()
}

/// 结束单个可释放进程
#[tauri::command]
pub fn release_port(pid: u32) -> Result<(), String> {
    ports::release_pid(pid)
}

/// 一键解除所有可释放占用
#[tauri::command]
pub fn release_releasable_ports() -> Result<ReleaseReport, String> {
    ports::release_all_releasable()
}
