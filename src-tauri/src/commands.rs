//! Tauri Command 处理器 — 供前端主动调用

use crate::audio::{self, AudioDiagReport};
use crate::autostart;
use crate::bluetooth;
use crate::bsod;
use crate::devices::{self, DeviceRescanResult, DevicesDiagReport};
use crate::events::{BluetoothStatusEvent, BsodAlertEvent, DeviceEvent};
use crate::history;
use crate::i18n;
use crate::network::{self, NetworkDiagReport, SpeedTestResult};
use crate::notify;
use crate::ports::{self, PortScanReport, ReleaseReport};
use crate::repair;
use crate::settings::{self, AppSettings};
use crate::tray::{self, TrayLevel};
use crate::usb_storage::{self, LockingProcess, UsbDiagReport, UsbDrive};
use crate::utils;
use chrono::Local;
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::{Mutex as AsyncMutex, Semaphore};

async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|error| format!("后台任务异常终止: {error}"))?
}

const DIAGNOSTIC_CONCURRENCY: usize = 4;
const FULL_SCAN_CACHE_TTL: Duration = Duration::from_secs(2);

static DIAGNOSTIC_PERMITS: OnceLock<Arc<Semaphore>> = OnceLock::new();
static FULL_SCAN_CACHE: OnceLock<AsyncMutex<FullScanCache>> = OnceLock::new();
static FULL_SCAN_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Default)]
struct FullScanCache {
    finished_at: Option<Instant>,
    payload: Option<Value>,
}

fn diagnostic_permits() -> &'static Arc<Semaphore> {
    DIAGNOSTIC_PERMITS.get_or_init(|| Arc::new(Semaphore::new(DIAGNOSTIC_CONCURRENCY)))
}

fn full_scan_cache() -> &'static AsyncMutex<FullScanCache> {
    FULL_SCAN_CACHE.get_or_init(|| AsyncMutex::new(FullScanCache::default()))
}

async fn invalidate_full_scan_cache() {
    let mut cache = full_scan_cache().lock().await;
    cache.finished_at = None;
    cache.payload = None;
}

async fn run_diagnostic_blocking<T, F>(name: &'static str, task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    let queued_at = Instant::now();
    let _permit = diagnostic_permits()
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| "Diagnostic scheduler is unavailable".to_string())?;
    let queue_ms = queued_at.elapsed().as_millis();
    run_blocking(move || {
        // Keep the permit inside the blocking job. If an async caller times out,
        // the underlying Windows query still counts against the concurrency cap
        // until it actually exits.
        let _permit = _permit;
        let started = Instant::now();
        let result = task();
        utils::logging::info(format!(
            "performance diagnostic={name} queue_ms={queue_ms} run_ms={}",
            started.elapsed().as_millis()
        ));
        result
    })
    .await
}

#[derive(Debug, Serialize)]
struct FullScanItem {
    duration_ms: u64,
    result: Option<Value>,
    error: Option<String>,
}

async fn full_scan_item<T, F>(name: &'static str, task: F) -> FullScanItem
where
    T: Serialize + Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    let started = Instant::now();
    let timeout = Duration::from_secs(settings::get().full_scan_timeout_secs);
    let outcome = match tokio::time::timeout(timeout, run_diagnostic_blocking(name, task)).await {
        Ok(outcome) => outcome,
        Err(_) => Err(format!(
            "{name} scan timed out after {} seconds",
            timeout.as_secs()
        )),
    };
    let duration_ms = started.elapsed().as_millis().min(u64::MAX as u128) as u64;
    match outcome {
        Ok(value) => match serde_json::to_value(value) {
            Ok(result) => FullScanItem {
                duration_ms,
                result: Some(result),
                error: None,
            },
            Err(error) => FullScanItem {
                duration_ms,
                result: None,
                error: Some(format!("Serialize {name} result failed: {error}")),
            },
        },
        Err(error) => FullScanItem {
            duration_ms,
            result: None,
            error: Some(error),
        },
    }
}

#[tauri::command]
pub async fn full_scan(
    app: AppHandle,
    force: Option<bool>,
    request_id: Option<u64>,
) -> Result<Value, String> {
    // Holding this lock across the scan intentionally coalesces concurrent full-scan
    // requests. A waiter receives the freshly cached result instead of starting the
    // same expensive Windows queries again.
    let mut cache = full_scan_cache().lock().await;
    if !force.unwrap_or(false) {
        if let (Some(finished_at), Some(payload)) = (cache.finished_at, cache.payload.as_ref()) {
            if finished_at.elapsed() <= FULL_SCAN_CACHE_TTL {
                let mut cached = payload.clone();
                if let Some(object) = cached.as_object_mut() {
                    object.insert("cached".into(), Value::Bool(true));
                }
                return Ok(cached);
            }
        }
    }

    let scan_id = FULL_SCAN_ID.fetch_add(1, Ordering::Relaxed) + 1;
    let started = Instant::now();
    let mut tasks = tokio::task::JoinSet::new();
    tasks.spawn(async {
        (
            "network",
            full_scan_item("network", network::diagnose).await,
        )
    });
    tasks.spawn(async { ("audio", full_scan_item("audio", audio::diagnose).await) });
    tasks.spawn(async { ("usb", full_scan_item("usb", usb_storage::diagnose).await) });
    tasks.spawn(async {
        (
            "bluetooth",
            full_scan_item("bluetooth", || {
                let report = bluetooth::diagnose()?;
                Ok(BluetoothStatusEvent {
                    timestamp: Local::now().to_rfc3339(),
                    healthy: !report.has_issues(),
                    bthserv_state: report.bthserv_state,
                    issues: report.issues,
                    adapter_count: report.adapter_devices.len(),
                    adapters: report.adapter_devices,
                    devices: report.devices,
                })
            })
            .await,
        )
    });
    tasks.spawn(async {
        (
            "devices",
            full_scan_item("devices", devices::diagnose).await,
        )
    });

    let mut items = serde_json::Map::new();
    while let Some(outcome) = tasks.join_next().await {
        match outcome {
            Ok((name, item)) => {
                let item_value = serde_json::to_value(&item)
                    .map_err(|error| format!("Serialize {name} result failed: {error}"))?;
                if let Err(error) = app.emit(
                    "full-scan-progress",
                    json!({
                        "scan_id": scan_id,
                        "request_id": request_id,
                        "id": name,
                        "item": &item_value,
                    }),
                ) {
                    utils::logging::warn(format!(
                        "emit full-scan-progress failed for {name}: {error}"
                    ));
                }
                items.insert(name.into(), item_value);
            }
            Err(error) => {
                utils::logging::error(format!("full scan task terminated unexpectedly: {error}"));
            }
        }
    }

    for name in ["network", "audio", "usb", "bluetooth", "devices"] {
        items.entry(name).or_insert_with(|| {
            json!(FullScanItem {
                duration_ms: 0,
                result: None,
                error: Some(format!("{name} scan terminated unexpectedly")),
            })
        });
    }
    let total_ms = started.elapsed().as_millis().min(u64::MAX as u128) as u64;
    let payload = json!({
        "scan_id": scan_id,
        "cached": false,
        "total_ms": total_ms,
        "items": items,
    });
    utils::logging::info(format!(
        "performance full_scan id={scan_id} total_ms={total_ms}"
    ));
    cache.finished_at = Some(Instant::now());
    cache.payload = Some(payload.clone());
    Ok(payload)
}

#[derive(Debug, Serialize)]
pub struct RepairResult {
    pub success: bool,
    pub needs_admin: bool,
    pub services_restarted: Vec<String>,
    pub services_healthy: Vec<String>,
    pub service_errors: Vec<String>,
    pub usb_power_configs: Vec<repair::UsbPowerConfig>,
    pub power_scan_error: Option<String>,
    pub summary_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_count: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct ScopedRepairResult {
    pub services_restarted: Vec<String>,
    pub service_errors: Vec<String>,
    pub needs_admin: bool,
}

fn scoped_repair(restarted: Vec<String>, errors: Vec<String>) -> ScopedRepairResult {
    let elevated = utils::elevated::is_elevated();
    ScopedRepairResult {
        needs_admin: !elevated && repair::errors_need_elevation(&errors),
        services_restarted: restarted,
        service_errors: errors,
    }
}

#[tauri::command]
pub async fn check_bluetooth() -> Result<BluetoothStatusEvent, String> {
    run_diagnostic_blocking("bluetooth", || {
        let report = bluetooth::diagnose()?;
        Ok(BluetoothStatusEvent {
            timestamp: Local::now().to_rfc3339(),
            healthy: !report.has_issues(),
            bthserv_state: report.bthserv_state,
            issues: report.issues,
            adapter_count: report.adapter_devices.len(),
            adapters: report.adapter_devices.clone(),
            devices: report.devices,
        })
    })
    .await
}

#[tauri::command]
pub async fn bluetooth_remove_device(instance_id: String) -> Result<(), String> {
    run_blocking(move || bluetooth::remove_device(&instance_id)).await
}

#[tauri::command]
pub async fn bluetooth_reconnect_device(instance_id: String) -> Result<(), String> {
    run_blocking(move || bluetooth::reconnect_device(&instance_id)).await
}

#[tauri::command]
pub async fn repair_bluetooth() -> Result<ScopedRepairResult, String> {
    let result = run_blocking(|| {
        let (ok, err) = bluetooth::repair_service();
        Ok(scoped_repair(ok, err))
    })
    .await?;
    invalidate_full_scan_cache().await;
    Ok(result)
}

#[tauri::command]
pub async fn diagnose_network() -> Result<NetworkDiagReport, String> {
    run_diagnostic_blocking("network", network::diagnose).await
}

#[tauri::command]
pub async fn network_speed_test() -> Result<SpeedTestResult, String> {
    run_blocking(network::speed_test).await
}

#[tauri::command]
pub async fn network_flush_dns() -> Result<(), String> {
    run_blocking(network::flush_dns).await
}

#[tauri::command]
pub async fn repair_network() -> Result<ScopedRepairResult, String> {
    let result = run_blocking(|| {
        let (ok, err) = network::repair();
        Ok(scoped_repair(ok, err))
    })
    .await?;
    invalidate_full_scan_cache().await;
    Ok(result)
}

#[tauri::command]
pub async fn diagnose_audio() -> Result<AudioDiagReport, String> {
    run_diagnostic_blocking("audio", audio::diagnose).await
}

#[tauri::command]
pub async fn set_default_audio_device(device_id: String, kind: String) -> Result<(), String> {
    run_blocking(move || audio::set_default_device(&device_id, &kind)).await
}

#[tauri::command]
pub async fn set_audio_mode(device_id: String, kind: String, mode: String) -> Result<(), String> {
    run_blocking(move || audio::set_device_mode(&device_id, &kind, &mode)).await
}

#[tauri::command]
pub async fn set_audio_volume(device_id: String, percent: u8) -> Result<(), String> {
    run_blocking(move || audio::set_endpoint_volume(&device_id, percent)).await
}

#[tauri::command]
pub async fn set_audio_mute(device_id: String, muted: bool) -> Result<(), String> {
    run_blocking(move || audio::set_endpoint_mute(&device_id, muted)).await
}

#[tauri::command]
pub async fn repair_audio() -> Result<ScopedRepairResult, String> {
    let result = run_blocking(|| {
        let (ok, err) = audio::repair();
        Ok(scoped_repair(ok, err))
    })
    .await?;
    invalidate_full_scan_cache().await;
    Ok(result)
}

#[tauri::command]
pub async fn diagnose_usb() -> Result<UsbDiagReport, String> {
    run_diagnostic_blocking("usb", usb_storage::diagnose).await
}

#[tauri::command]
pub async fn usb_list_drives() -> Result<Vec<UsbDrive>, String> {
    run_blocking(usb_storage::list_drives).await
}

#[tauri::command]
pub async fn usb_locking_processes(drive_letter: String) -> Result<Vec<LockingProcess>, String> {
    run_blocking(move || usb_storage::find_locking_processes(&drive_letter)).await
}

#[tauri::command]
pub async fn usb_close_process(pid: u32) -> Result<usb_storage::UsbCloseProcessResult, String> {
    run_blocking(move || usb_storage::request_close_process(pid)).await
}

#[tauri::command]
pub async fn usb_open_volume(drive_letter: String) -> Result<(), String> {
    run_blocking(move || usb_storage::open_volume(&drive_letter)).await
}

#[tauri::command]
pub async fn usb_eject(drive_letter: String) -> Result<usb_storage::UsbEjectResult, String> {
    run_blocking(move || usb_storage::eject_drive(&drive_letter)).await
}

#[tauri::command]
pub async fn usb_format_volume(
    drive_letter: String,
    filesystem: String,
    label: String,
    full: bool,
) -> Result<(), String> {
    run_blocking(move || usb_storage::format_volume(&drive_letter, &filesystem, &label, full)).await
}

#[tauri::command]
pub async fn repair_usb() -> Result<ScopedRepairResult, String> {
    let result = run_blocking(|| {
        let (ok, err) = usb_storage::repair();
        Ok(scoped_repair(ok, err))
    })
    .await?;
    invalidate_full_scan_cache().await;
    Ok(result)
}

#[tauri::command]
pub async fn diagnose_devices() -> Result<DevicesDiagReport, String> {
    run_diagnostic_blocking("devices", devices::diagnose).await
}

#[tauri::command]
pub async fn rescan_devices() -> Result<DeviceRescanResult, String> {
    run_blocking(devices::rescan).await
}

#[tauri::command]
pub async fn scan_bsod() -> Result<Option<BsodAlertEvent>, String> {
    run_diagnostic_blocking("bsod", || {
        let report = bsod::analyze_latest_dump().map_err(|e| format!("BSOD scan failed: {e}"))?;
        Ok(report.map(|r| bsod::report_to_event(&r)))
    })
    .await
}

#[tauri::command]
pub async fn apply_bsod_repairs(fix_ids: Vec<String>) -> Result<Vec<String>, String> {
    run_blocking(move || bsod::apply_repairs(fix_ids)).await
}

#[tauri::command]
pub async fn run_repair(app: AppHandle) -> Result<RepairResult, String> {
    let elevated = utils::elevated::is_elevated();
    let report =
        run_blocking(|| repair::run_auto_repair().map_err(|e| format!("Repair failed: {e}")))
            .await?;
    invalidate_full_scan_cache().await;

    let success = report.service_errors.is_empty() && report.power_scan_error.is_none();
    let needs_admin = !elevated && repair::errors_need_elevation(&report.service_errors);
    let (summary_id, summary_count) = repair::build_summary_meta(success, needs_admin, &report);
    let locale = settings::get().locale;
    let summary_text = i18n::repair_summary(&locale, &summary_id, summary_count);

    let result = RepairResult {
        success,
        needs_admin,
        services_restarted: report.services_restarted.clone(),
        services_healthy: report.services_healthy,
        service_errors: report.service_errors.clone(),
        usb_power_configs: report.usb_power_configs,
        power_scan_error: report.power_scan_error.clone(),
        summary_id: summary_id.clone(),
        summary_count,
    };

    notify::send_if_background(&app, &i18n::notify_repair_title(&locale), &summary_text);

    if success {
        tray::set_level(&app, TrayLevel::Normal, "repair_done");
    } else if needs_admin {
        tray::set_level(&app, TrayLevel::Warning, "repair_admin");
    } else {
        tray::set_level(&app, TrayLevel::Warning, "repair_partial");
    }

    Ok(result)
}

#[tauri::command]
pub fn get_device_history() -> Vec<DeviceEvent> {
    history::list()
}

#[tauri::command]
pub fn clear_device_history() -> Result<(), String> {
    history::clear()
}

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
        .set_title(i18n::export_dialog_title(&locale))
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

#[tauri::command]
pub fn is_elevated() -> bool {
    utils::elevated::is_elevated()
}

#[tauri::command]
pub fn restart_elevated(app: AppHandle) -> Result<(), String> {
    utils::elevated::relaunch_as_admin()?;
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn get_settings() -> AppSettings {
    settings::get()
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: AppSettings) -> Result<AppSettings, String> {
    let previous = settings::get();
    let saved = settings::save(settings)?;
    if previous.launch_at_startup != saved.launch_at_startup {
        autostart::sync(&app, saved.launch_at_startup)?;
    }
    if previous.locale != saved.locale {
        crate::tray::refresh_locale(&app);
    }
    Ok(saved)
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub async fn scan_ports() -> Result<PortScanReport, String> {
    run_diagnostic_blocking("ports", ports::scan).await
}

#[tauri::command]
pub async fn release_port(pid: u32) -> Result<(), String> {
    run_blocking(move || ports::release_pid(pid)).await
}

#[tauri::command]
pub async fn release_connection(connection_key: String) -> Result<(), String> {
    run_blocking(move || ports::release_connection(&connection_key)).await
}

#[tauri::command]
pub async fn release_releasable_ports() -> Result<ReleaseReport, String> {
    run_blocking(ports::release_all_releasable).await
}
