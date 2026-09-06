//! Tauri Command 处理器 — 供前端主动调用

use crate::audio::{self, AudioDiagReport};
use crate::autostart;
use crate::bluetooth;
use crate::bsod;
use crate::devices::{self, DeviceRepairResult, DeviceRescanResult, DevicesDiagReport};
use crate::events::{BluetoothStatusEvent, BsodAlertEvent, DeviceEvent};
use crate::history;
use crate::i18n;
use crate::network::{self, NetworkDiagReport, SpeedTestResult};
use crate::notify;
use crate::ports::{self, PortScanReport, ReleaseReport};
use crate::repair;
use crate::settings::{self, AppSettings};
use crate::tray::{self, TrayLevel};
use crate::updates::{self, UpdateInfo};
use crate::usb_storage::{self, LockingProcess, UsbDiagReport};
use crate::utils;
use chrono::Local;
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::{Mutex as AsyncMutex, Notify, Semaphore};

async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|error| format!("后台任务异常终止: {error}"))?
}

// Hold the guard in the blocking job, including its verification. Cancelling
// the IPC waiter must not allow another repair to interrupt an active operation.
static SERVICE_REPAIR_LOCK: Mutex<()> = Mutex::new(());

async fn run_service_repair<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    run_blocking(move || {
        let _guard = match SERVICE_REPAIR_LOCK.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::WouldBlock) => return Err("service_repair_busy".into()),
            Err(std::sync::TryLockError::Poisoned(error)) => {
                // Only unit data is protected. A previous worker panic must not
                // permanently disable every later repair and audio control.
                utils::logging::warn(
                    "Previous service repair task panicked; allowing a fresh operation",
                );
                SERVICE_REPAIR_LOCK.clear_poison();
                error.into_inner()
            }
        };
        task()
    })
    .await
}

const DIAGNOSTIC_CONCURRENCY: usize = 4;
static DIAGNOSTIC_PERMITS: OnceLock<Arc<Semaphore>> = OnceLock::new();
static FULL_SCAN_STATE: OnceLock<AsyncMutex<Option<Arc<FullScanFlight>>>> = OnceLock::new();
static FULL_SCAN_ID: AtomicU64 = AtomicU64::new(0);

fn diagnostic_permits() -> &'static Arc<Semaphore> {
    DIAGNOSTIC_PERMITS.get_or_init(|| Arc::new(Semaphore::new(DIAGNOSTIC_CONCURRENCY)))
}

fn full_scan_state() -> &'static AsyncMutex<Option<Arc<FullScanFlight>>> {
    FULL_SCAN_STATE.get_or_init(|| AsyncMutex::new(None))
}

struct FullScanFlight {
    result: AsyncMutex<Option<Result<Value, String>>>,
    completed: Notify,
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
pub async fn full_scan(app: AppHandle, request_id: Option<u64>) -> Result<Value, String> {
    // Share only a scan that is currently collecting live evidence. The global slot is
    // cleared before waiters are notified, so a later invocation always starts a fresh scan.
    let flight = {
        let mut state = full_scan_state().lock().await;
        if let Some(flight) = state.as_ref() {
            Arc::clone(flight)
        } else {
            let flight = Arc::new(FullScanFlight {
                result: AsyncMutex::new(None),
                completed: Notify::new(),
            });
            *state = Some(Arc::clone(&flight));
            let task_flight = Arc::clone(&flight);
            tokio::spawn(async move {
                let result = tokio::spawn(execute_full_scan(app, request_id))
                    .await
                    .unwrap_or_else(|error| {
                        Err(format!("full scan task terminated unexpectedly: {error}"))
                    });
                let mut state = full_scan_state().lock().await;
                if state
                    .as_ref()
                    .is_some_and(|current| Arc::ptr_eq(current, &task_flight))
                {
                    *state = None;
                }
                *task_flight.result.lock().await = Some(result);
                drop(state);
                task_flight.completed.notify_waiters();
            });
            flight
        }
    };

    loop {
        let completed = flight.completed.notified();
        if let Some(result) = flight.result.lock().await.clone() {
            return result;
        }
        completed.await;
    }
}

async fn execute_full_scan(app: AppHandle, request_id: Option<u64>) -> Result<Value, String> {
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
                let report = bluetooth::diagnose_health()?;
                Ok(bluetooth_status_event(report))
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
        "total_ms": total_ms,
        "items": items,
    });
    utils::logging::info(format!(
        "performance full_scan id={scan_id} total_ms={total_ms}"
    ));
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

#[derive(Debug, Serialize)]
pub struct NetworkRepairResult {
    #[serde(flatten)]
    pub operation: ScopedRepairResult,
    pub dns_flush_attempted: bool,
    pub dns_cache_cleared: bool,
    pub before_test: Option<network::probe::ConnectionProbe>,
    pub after_test: Option<network::probe::ConnectionProbe>,
    pub before_error: Option<String>,
    pub verification_error: Option<String>,
    pub after: Option<NetworkDiagReport>,
    pub refresh_error: Option<String>,
    pub outcome: &'static str,
}

fn network_repair_outcome(
    before: Option<&network::probe::ConnectionProbe>,
    after: Option<&network::probe::ConnectionProbe>,
    operation: &ScopedRepairResult,
    changed: bool,
) -> &'static str {
    let Some(after) = after else {
        return "unverified";
    };
    if !operation.service_errors.is_empty() || operation.needs_admin {
        return "needs_attention";
    }
    if !network::probe::response_observed(after) {
        return if matches!(after.dns_status.as_str(), "timeout" | "busy") {
            "unverified"
        } else {
            "needs_attention"
        };
    }
    let observed_failure = before.is_some_and(|before| {
        !network::probe::response_observed(before)
            && !matches!(before.dns_status.as_str(), "timeout" | "busy")
    });
    if changed && observed_failure {
        "target_recovered"
    } else {
        "target_responded"
    }
}

fn perform_network_repair(
    target: &str,
    restore_services: bool,
) -> Result<NetworkRepairResult, String> {
    // Reject invalid/credential-bearing targets before changing any system state.
    let target = network::probe::validate_target(target)?.to_string();
    let (before_test, before_error) = match network::probe::run(&target) {
        Ok(report) => (Some(report), None),
        Err(error) => (None, Some(error)),
    };
    let (restored, mut errors) = if restore_services {
        network::repair()
    } else {
        (Vec::new(), Vec::new())
    };
    let dns_cache_cleared = match network::flush_dns() {
        Ok(()) => true,
        Err(error) => {
            errors.push(error);
            false
        }
    };
    let operation = scoped_repair(restored, errors);
    // Always keep the action record, even when the probe or panel refresh fails.
    let (after_test, verification_error) = match network::probe::run(&target) {
        Ok(report) => (Some(report), None),
        Err(error) => (None, Some(error)),
    };
    let outcome = network_repair_outcome(
        before_test.as_ref(),
        after_test.as_ref(),
        &operation,
        dns_cache_cleared || !operation.services_restarted.is_empty(),
    );
    let (after, refresh_error) = match network::diagnose_after_repair() {
        Ok(report) => (Some(report), None),
        Err(error) => (None, Some(error)),
    };
    Ok(NetworkRepairResult {
        operation,
        dns_flush_attempted: true,
        dns_cache_cleared,
        before_test,
        after_test,
        before_error,
        verification_error,
        after,
        refresh_error,
        outcome,
    })
}

#[derive(Debug, Serialize)]
pub struct AudioRepairResult {
    #[serde(flatten)]
    pub operation: ScopedRepairResult,
    pub before: Option<AudioDiagReport>,
    pub after: Option<AudioDiagReport>,
    pub before_error: Option<String>,
    pub verification_error: Option<String>,
    pub outcome: AudioRepairOutcome,
    pub findings: Vec<&'static str>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AudioRepairOutcome {
    Recovered,
    Ready,
    NeedsAttention,
    Unverified,
}

fn audio_findings(report: &AudioDiagReport) -> Vec<&'static str> {
    let mut findings = Vec::new();
    if !report.services.issues.is_empty()
        || !crate::services::AUDIO.iter().all(|required| {
            report.services.services.iter().any(|service| {
                service.name == required.name
                    && service.state.as_deref() == Some("Running")
                    && service
                        .start_mode
                        .as_deref()
                        .is_some_and(|mode| mode != "Disabled")
            })
        })
    {
        findings.push("service_issue");
    }
    if report.playback.is_empty() {
        findings.push("no_playback");
    } else if let Some(output) = report.playback.iter().find(|device| device.is_default) {
        if output.is_muted == Some(true) {
            findings.push("muted");
        }
        if output.volume_percent == Some(0) {
            findings.push("zero_volume");
        }
        if output.is_muted.is_none() || output.volume_percent.is_none() {
            findings.push("volume_unknown");
        }
    } else {
        findings.push("no_default_playback");
    }
    findings
}

fn audio_repair_outcome(
    before: Option<&AudioDiagReport>,
    after: Option<&AudioDiagReport>,
    operation: &ScopedRepairResult,
) -> (AudioRepairOutcome, Vec<&'static str>) {
    let Some(after) = after else {
        return (AudioRepairOutcome::Unverified, Vec::new());
    };
    let findings = audio_findings(after);
    if !operation.service_errors.is_empty() || operation.needs_admin || !findings.is_empty() {
        return (AudioRepairOutcome::NeedsAttention, findings);
    }
    let was_faulty = before.is_some_and(|report| {
        audio_findings(report)
            .iter()
            .any(|finding| *finding != "volume_unknown")
    });
    let outcome = if was_faulty && !operation.services_restarted.is_empty() {
        AudioRepairOutcome::Recovered
    } else {
        AudioRepairOutcome::Ready
    };
    (outcome, findings)
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
pub async fn check_bluetooth(app: AppHandle) -> Result<BluetoothStatusEvent, String> {
    let initial = run_diagnostic_blocking("bluetooth", || {
        let report = bluetooth::diagnose()?;
        Ok(bluetooth_status_event(report))
    })
    .await?;
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        match tokio::task::spawn_blocking(bluetooth::refresh_gatt_battery_levels).await {
            Ok(Ok(Some(report))) => {
                if let Err(error) =
                    app_handle.emit("bluetooth-battery-refresh", bluetooth_status_event(report))
                {
                    utils::logging::warn(format!("emit bluetooth-battery-refresh failed: {error}"));
                }
            }
            Ok(Ok(None)) => {}
            Ok(Err(error)) => {
                utils::logging::info(format!("Bluetooth live battery refresh skipped: {error}"));
            }
            Err(error) => utils::logging::warn(format!(
                "Bluetooth live battery refresh task terminated: {error}"
            )),
        }
    });
    Ok(initial)
}

fn bluetooth_status_event(report: bluetooth::BluetoothReport) -> BluetoothStatusEvent {
    BluetoothStatusEvent {
        timestamp: Local::now().to_rfc3339(),
        healthy: !report.has_issues(),
        bthserv_state: report.bthserv_state,
        issues: report.issues,
        adapter_count: report.adapter_devices.len(),
        adapters: report.adapter_devices,
        devices: report.devices,
    }
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
    let result = run_service_repair(|| {
        let (ok, err) = bluetooth::repair_service();
        Ok(scoped_repair(ok, err))
    })
    .await?;
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
pub async fn network_flush_dns(target: String) -> Result<NetworkRepairResult, String> {
    run_service_repair(move || perform_network_repair(&target, false)).await
}

#[tauri::command]
pub async fn repair_network(target: String) -> Result<NetworkRepairResult, String> {
    run_service_repair(move || perform_network_repair(&target, true)).await
}

#[tauri::command]
pub async fn network_test_connection(
    target: String,
) -> Result<network::probe::ConnectionProbe, String> {
    // Do not overlap the before/after test of an in-progress service repair.
    run_service_repair(move || network::probe::run(&target)).await
}

#[tauri::command]
pub async fn diagnose_audio() -> Result<AudioDiagReport, String> {
    run_diagnostic_blocking("audio", audio::diagnose).await
}

#[tauri::command]
pub async fn set_default_audio_device(device_id: String, kind: String) -> Result<(), String> {
    run_service_repair(move || audio::set_default_device(&device_id, &kind)).await
}

#[tauri::command]
pub async fn set_audio_mode(device_id: String, kind: String, mode: String) -> Result<(), String> {
    run_service_repair(move || audio::set_device_mode(&device_id, &kind, &mode)).await
}

#[tauri::command]
pub async fn set_audio_volume(device_id: String, percent: u8) -> Result<(), String> {
    run_service_repair(move || audio::set_endpoint_volume(&device_id, percent)).await
}

#[tauri::command]
pub async fn set_audio_mute(device_id: String, muted: bool) -> Result<(), String> {
    run_service_repair(move || audio::set_endpoint_mute(&device_id, muted)).await
}

#[tauri::command]
pub async fn repair_audio() -> Result<AudioRepairResult, String> {
    run_service_repair(|| {
        // A stopped audio service may prevent endpoint enumeration. Keep that
        // failure as evidence but still allow the explicitly requested recovery.
        let (before, before_error) = match audio::diagnose_live() {
            Ok(report) => (Some(report), None),
            Err(error) => (None, Some(error)),
        };
        let (ok, err) = audio::repair();
        let operation = scoped_repair(ok, err);
        // Verification must never use the registry fallback or a cached service
        // snapshot. Keep operation results even if live verification fails.
        let (after, verification_error) = match audio::diagnose_live() {
            Ok(report) => (Some(report), None),
            Err(error) => (None, Some(error)),
        };
        let (outcome, findings) = audio_repair_outcome(before.as_ref(), after.as_ref(), &operation);
        Ok(AudioRepairResult {
            operation,
            before,
            after,
            before_error,
            verification_error,
            outcome,
            findings,
        })
    })
    .await
}

#[tauri::command]
pub async fn diagnose_usb() -> Result<UsbDiagReport, String> {
    run_diagnostic_blocking("usb", usb_storage::diagnose).await
}

#[tauri::command]
pub async fn diagnose_usb_progressive(app: AppHandle) -> Result<UsbDiagReport, String> {
    let initial = run_diagnostic_blocking("usb_quick", usb_storage::diagnose_quick).await?;
    tauri::async_runtime::spawn(async move {
        match run_diagnostic_blocking("usb", usb_storage::diagnose).await {
            Ok(report) => {
                if let Err(error) = app.emit("usb-storage-refresh", report) {
                    utils::logging::warn(format!("emit usb-storage-refresh failed: {error}"));
                }
            }
            Err(error) => {
                utils::logging::warn(format!(
                    "USB complete background diagnostic failed: {error}"
                ));
                if let Err(emit_error) = app.emit("usb-storage-refresh-error", &error) {
                    utils::logging::warn(format!(
                        "emit usb-storage-refresh-error failed: {emit_error}"
                    ));
                }
            }
        }
    });
    Ok(initial)
}

#[tauri::command]
pub async fn usb_locking_processes(drive_letter: String) -> Result<Vec<LockingProcess>, String> {
    run_blocking(move || usb_storage::find_locking_processes(&drive_letter)).await
}

#[tauri::command]
pub async fn usb_close_process(
    pid: u32,
    drive_letter: String,
    expected_process_name: String,
) -> Result<usb_storage::UsbCloseProcessResult, String> {
    run_blocking(move || {
        usb_storage::request_close_process(pid, &drive_letter, &expected_process_name)
    })
    .await
}

#[tauri::command]
pub async fn usb_open_volume(drive_letter: String) -> Result<(), String> {
    run_blocking(move || usb_storage::open_volume(&drive_letter)).await
}

#[tauri::command]
pub async fn usb_eject(drive_letter: String) -> Result<usb_storage::UsbEjectResult, String> {
    let result = run_blocking(move || usb_storage::eject_drive(&drive_letter)).await?;
    usb_storage::invalidate_diagnostic_cache();
    Ok(result)
}

#[tauri::command]
pub async fn usb_format_volume(
    drive_letter: String,
    filesystem: String,
    label: String,
    full: bool,
) -> Result<(), String> {
    run_blocking(move || usb_storage::format_volume(&drive_letter, &filesystem, &label, full))
        .await?;
    usb_storage::invalidate_diagnostic_cache();
    Ok(())
}

#[tauri::command]
pub async fn repair_usb() -> Result<ScopedRepairResult, String> {
    let result = run_service_repair(|| {
        let (ok, err) = usb_storage::repair();
        Ok(scoped_repair(ok, err))
    })
    .await?;
    usb_storage::invalidate_diagnostic_cache();
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
pub async fn repair_device_driver(
    device_id: String,
    action_id: String,
    inf_path: Option<String>,
) -> Result<DeviceRepairResult, String> {
    let result =
        run_blocking(move || devices::repair_device(&device_id, &action_id, inf_path.as_deref()))
            .await?;
    Ok(result)
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
        run_service_repair(|| repair::run_auto_repair().map_err(|e| format!("Repair failed: {e}")))
            .await?;

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

    // A service operation is not a fresh device/network diagnosis. Preserve the
    // tray's diagnostic warning until the corresponding diagnostic clears it.
    if needs_admin {
        tray::set_level(&app, TrayLevel::Warning, "repair_admin");
    } else if !success {
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
    utils::elevated::relaunch_as_admin(false)?;
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
    let startup_mode_changed = previous.launch_at_startup != saved.launch_at_startup
        || previous.run_as_admin != saved.run_as_admin;
    if startup_mode_changed && !(saved.launch_at_startup && saved.run_as_admin && !is_elevated()) {
        autostart::sync(&app, saved.launch_at_startup, saved.run_as_admin)?;
    }
    if previous.locale != saved.locale {
        crate::tray::refresh_locale(&app);
    }
    Ok(saved)
}

#[tauri::command]
pub fn take_autostart_error() -> Option<String> {
    autostart::take_last_error()
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub async fn check_for_updates(force: bool) -> Result<UpdateInfo, String> {
    run_blocking(move || updates::check(force)).await
}

#[tauri::command]
pub fn open_project_url(app: AppHandle, url: String) -> Result<(), String> {
    updates::open_project_url(&app, &url)
}

#[tauri::command]
pub async fn scan_ports() -> Result<PortScanReport, String> {
    run_diagnostic_blocking("ports", ports::scan).await
}

#[tauri::command]
pub async fn release_port(
    pid: u32,
    expected_process_name: String,
    port: u16,
) -> Result<(), String> {
    run_blocking(move || ports::release_pid(pid, &expected_process_name, port)).await
}

#[tauri::command]
pub async fn release_releasable_ports() -> Result<ReleaseReport, String> {
    run_blocking(ports::release_all_releasable).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioDevice;
    use crate::services::ServicesReport;

    fn playback_report(muted: Option<bool>, volume: Option<u8>) -> AudioDiagReport {
        AudioDiagReport {
            services: ServicesReport {
                services: crate::services::AUDIO
                    .iter()
                    .map(|service| crate::events::ServiceEntry {
                        name: service.name.into(),
                        label_id: service.label_id.into(),
                        state: Some("Running".into()),
                        start_mode: Some("Auto".into()),
                        expected_stopped: false,
                    })
                    .collect(),
                issues: Vec::new(),
            },
            playback: vec![AudioDevice {
                id: "output".into(),
                name: "Speakers".into(),
                kind: "playback".into(),
                category: "speakers".into(),
                is_default: true,
                mode: "shared".into(),
                volume_percent: volume,
                is_muted: muted,
            }],
            capture: Vec::new(),
        }
    }

    fn completed_audio_operation() -> ScopedRepairResult {
        ScopedRepairResult {
            services_restarted: vec!["Audiosrv".into()],
            service_errors: Vec::new(),
            needs_admin: false,
        }
    }

    #[test]
    fn audio_capture_only_does_not_verify_playback_recovery() {
        let mut after = playback_report(Some(false), Some(50));
        let mut microphone = after.playback.remove(0);
        microphone.kind = "capture".into();
        after.capture.push(microphone);
        let (outcome, findings) =
            audio_repair_outcome(None, Some(&after), &completed_audio_operation());
        assert_eq!(outcome, AudioRepairOutcome::NeedsAttention);
        assert_eq!(findings, ["no_playback"]);
    }

    #[test]
    fn audio_running_service_does_not_hide_muted_silent_or_unknown_output() {
        for (muted, volume, expected) in [
            (Some(true), Some(40), "muted"),
            (Some(false), Some(0), "zero_volume"),
            (None, Some(40), "volume_unknown"),
            (Some(false), None, "volume_unknown"),
        ] {
            let after = playback_report(muted, volume);
            let (outcome, findings) =
                audio_repair_outcome(None, Some(&after), &completed_audio_operation());
            assert_eq!(outcome, AudioRepairOutcome::NeedsAttention);
            assert!(findings.contains(&expected));
        }
        let mut after = playback_report(Some(false), Some(40));
        after.playback[0].is_default = false;
        assert_eq!(audio_findings(&after), ["no_default_playback"]);
    }

    #[test]
    fn audio_recovered_requires_observed_fault_and_completed_action() {
        let before = playback_report(Some(true), Some(40));
        let after = playback_report(Some(false), Some(40));
        let mut operation = completed_audio_operation();
        assert_eq!(
            audio_repair_outcome(Some(&before), Some(&after), &operation).0,
            AudioRepairOutcome::Recovered
        );
        operation.services_restarted.clear();
        assert_eq!(
            audio_repair_outcome(Some(&before), Some(&after), &operation).0,
            AudioRepairOutcome::Ready
        );
        let unknown_before = playback_report(None, None);
        assert_eq!(
            audio_repair_outcome(
                Some(&unknown_before),
                Some(&after),
                &completed_audio_operation()
            )
            .0,
            AudioRepairOutcome::Ready
        );
    }

    #[test]
    fn audio_failed_verification_keeps_completed_operation_in_response() {
        let operation = completed_audio_operation();
        let (outcome, findings) = audio_repair_outcome(None, None, &operation);
        assert_eq!(outcome, AudioRepairOutcome::Unverified);
        let response = AudioRepairResult {
            operation,
            before: None,
            after: None,
            before_error: Some("Audio service unavailable".into()),
            verification_error: Some("Live endpoint enumeration failed".into()),
            outcome,
            findings,
        };
        let json = serde_json::to_value(response).unwrap();
        assert_eq!(json["services_restarted"][0], "Audiosrv");
        assert_eq!(json["outcome"], "unverified");
        assert!(json["verification_error"].is_string());
        assert!(json["after"].is_null());
    }

    #[test]
    fn audio_operation_error_is_not_cleared_by_healthy_output() {
        let after = playback_report(Some(false), Some(40));
        let mut operation = completed_audio_operation();
        operation
            .service_errors
            .push("Audiosrv: access denied".into());
        operation.needs_admin = true;
        assert_eq!(
            audio_repair_outcome(None, Some(&after), &operation).0,
            AudioRepairOutcome::NeedsAttention
        );
    }

    #[test]
    fn audio_missing_service_evidence_is_not_a_healthy_verification() {
        let mut after = playback_report(Some(false), Some(40));
        after.services.services.clear();
        assert_eq!(audio_findings(&after), ["service_issue"]);
    }

    fn network_probe(dns: &str, status: Option<u16>) -> network::probe::ConnectionProbe {
        network::probe::ConnectionProbe {
            target: "https://example.com/".into(),
            dns_status: dns.into(),
            addresses: Vec::new(),
            http_status: status,
            http_error: status.is_none().then(|| "connect".into()),
            elapsed_ms: 1,
        }
    }

    #[test]
    fn network_repair_distinguishes_target_response_from_recovery() {
        let operation = ScopedRepairResult {
            services_restarted: Vec::new(),
            service_errors: Vec::new(),
            needs_admin: false,
        };
        let failed = network_probe("failed", None);
        let responded = network_probe("resolved", Some(200));
        let denied = network_probe("resolved", Some(403));
        assert_eq!(
            network_repair_outcome(Some(&failed), Some(&responded), &operation, true),
            "target_recovered"
        );
        assert_eq!(
            network_repair_outcome(Some(&failed), Some(&responded), &operation, false),
            "target_responded"
        );
        assert_eq!(
            network_repair_outcome(Some(&denied), Some(&responded), &operation, true),
            "target_responded"
        );
        // A server refusing access still proves it responded; it is not no-network evidence.
        assert_eq!(
            network_repair_outcome(None, Some(&denied), &operation, true),
            "target_responded"
        );
    }

    #[test]
    fn network_repair_never_turns_missing_evidence_into_a_repaired_fault() {
        let mut operation = ScopedRepairResult {
            services_restarted: Vec::new(),
            service_errors: Vec::new(),
            needs_admin: false,
        };
        let responded = network_probe("resolved", Some(200));
        for dns in ["timeout", "busy"] {
            let unknown = network_probe(dns, None);
            assert_eq!(
                network_repair_outcome(Some(&unknown), Some(&responded), &operation, true),
                "target_responded"
            );
            assert_eq!(
                network_repair_outcome(None, Some(&unknown), &operation, true),
                "unverified"
            );
        }
        assert_eq!(
            network_repair_outcome(None, None, &operation, true),
            "unverified"
        );
        operation
            .service_errors
            .push("network_dns_flush:timeout".into());
        assert_eq!(
            network_repair_outcome(None, Some(&responded), &operation, true),
            "needs_attention"
        );
    }

    #[test]
    fn network_failed_verification_preserves_the_dns_action_record() {
        let result = NetworkRepairResult {
            operation: ScopedRepairResult {
                services_restarted: Vec::new(),
                service_errors: Vec::new(),
                needs_admin: false,
            },
            dns_flush_attempted: true,
            dns_cache_cleared: true,
            before_test: None,
            after_test: None,
            before_error: None,
            verification_error: Some("network_probe:worker_failed".into()),
            after: None,
            refresh_error: Some("network_scan:worker_failed".into()),
            outcome: "unverified",
        };
        let json = serde_json::to_value(result).unwrap();
        assert_eq!(json["dns_cache_cleared"], true);
        assert_eq!(json["outcome"], "unverified");
        assert!(json["after_test"].is_null());
        assert!(json["verification_error"].is_string());
        assert!(json["refresh_error"].is_string());
    }
}
