//! Task 2：蓝牙驱动与状态异常诊断

use crate::events::BluetoothIssue;
use crate::events::BluetoothStatusEvent;
use crate::notify;
use crate::settings;
use crate::tray::{self, TrayLevel};
use crate::utils::{logging, wmi_runner};
use chrono::Local;
use serde::Deserialize;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::time;
use wmi::WMIConnection;

static LAST_HEALTHY: OnceLock<Mutex<Option<bool>>> = OnceLock::new();

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PnPDevice {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "PNPClass")]
    pnp_class: Option<String>,
    #[serde(rename = "ConfigManagerErrorCode")]
    error_code: Option<u32>,
    #[serde(rename = "Status")]
    status: Option<String>,
    #[serde(rename = "DeviceID")]
    device_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Win32Service {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "State")]
    state: Option<String>,
    #[serde(rename = "StartMode")]
    start_mode: Option<String>,
    #[serde(rename = "Status")]
    status: Option<String>,
}

#[derive(Debug, Default)]
pub struct BluetoothReport {
    pub radio_devices: Vec<String>,
    pub error_devices: Vec<String>,
    pub bthserv_state: Option<String>,
    pub bthserv_start_mode: Option<String>,
    pub issues: Vec<BluetoothIssue>,
}

impl BluetoothReport {
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }
}

pub fn diagnose() -> Result<BluetoothReport, String> {
    wmi_runner::run(diagnose_inner)
}

fn diagnose_inner(wmi: &WMIConnection) -> Result<BluetoothReport, wmi::WMIError> {
    let mut report = BluetoothReport::default();
    let pnp_query = "SELECT Name, PNPClass, ConfigManagerErrorCode, Status, DeviceID \
                     FROM Win32_PnPEntity \
                     WHERE PNPClass='Bluetooth' \
                        OR Name LIKE '%Bluetooth%' \
                        OR DeviceID LIKE '%BTH%'";
    let devices: Vec<PnPDevice> = wmi.raw_query(pnp_query)?;
    for dev in &devices {
        if let Some(name) = &dev.name {
            report.radio_devices.push(name.clone());
        }
        if let Some(code) = dev.error_code {
            if code != 0 {
                let name = dev.name.clone().unwrap_or_else(|| "Unknown".into());
                report.error_devices.push(name.clone());
                report.issues.push(BluetoothIssue {
                    id: "driver_error".into(),
                    name: Some(name),
                    state: None,
                    code: Some(code),
                });
            }
        }
    }
    if report.radio_devices.is_empty() {
        report.issues.push(BluetoothIssue {
            id: "no_radio".into(),
            name: None,
            state: None,
            code: None,
        });
    }
    let svc_query =
        "SELECT Name, State, StartMode, Status FROM Win32_Service WHERE Name='bthserv'";
    let services: Vec<Win32Service> = wmi.raw_query(svc_query)?;
    if let Some(svc) = services.first() {
        report.bthserv_state = svc.state.clone();
        report.bthserv_start_mode = svc.start_mode.clone();
        match svc.state.as_deref() {
            Some("Running") => {}
            Some(state) => report.issues.push(BluetoothIssue {
                id: "bthserv_not_running".into(),
                name: None,
                state: Some(state.to_string()),
                code: None,
            }),
            None => report.issues.push(BluetoothIssue {
                id: "bthserv_status_unknown".into(),
                name: None,
                state: None,
                code: None,
            }),
        }
        if svc.start_mode.as_deref() == Some("Disabled") {
            report.issues.push(BluetoothIssue {
                id: "bthserv_disabled".into(),
                name: None,
                state: None,
                code: None,
            });
        }
    } else {
        report.issues.push(BluetoothIssue {
            id: "bthserv_missing".into(),
            name: None,
            state: None,
            code: None,
        });
    }
    Ok(report)
}

/// WMI 轮询监控，状态变更时才 emit `bluetooth-status`
pub async fn run_monitor(app: AppHandle) {
    loop {
        run_cycle(&app).await;
        let secs = settings::get().bluetooth_poll_secs;
        time::sleep(Duration::from_secs(secs)).await;
    }
}

async fn run_cycle(app: &AppHandle) {
    match diagnose() {
        Ok(report) => {
            emit_report(&report);
            emit_status_event(app, &report);
        }
        Err(e) => logging::error(format!("Bluetooth WMI diagnose failed: {e}")),
    }
}

fn health_changed(healthy: bool) -> bool {
    let cell = LAST_HEALTHY.get_or_init(|| Mutex::new(None));
    let Ok(mut last) = cell.lock() else {
        return true;
    };
    let changed = *last != Some(healthy);
    if changed {
        *last = Some(healthy);
    }
    changed
}

fn emit_status_event(app: &AppHandle, report: &BluetoothReport) {
    let healthy = !report.has_issues();
    if !health_changed(healthy) {
        return;
    }

    let event = BluetoothStatusEvent {
        timestamp: Local::now().to_rfc3339(),
        healthy,
        bthserv_state: report.bthserv_state.clone(),
        issues: report.issues.clone(),
        radio_count: report.radio_devices.len(),
    };
    if let Err(e) = app.emit("bluetooth-status", &event) {
        logging::error(format!("emit bluetooth-status failed: {e}"));
    }
    if !event.healthy {
        tray::set_level(app, TrayLevel::Critical, "bluetooth_issue");
        let locale = settings::get().locale;
        let detail = event
            .issues
            .first()
            .map(|i| crate::i18n::format_bluetooth_issue(&locale, i))
            .unwrap_or_else(|| crate::i18n::tray_reason(&locale, "bluetooth_issue"));
        notify::send_if_background(
            app,
            &crate::i18n::notify_bluetooth_title(&locale),
            &detail,
        );
    }
}

fn emit_report(report: &BluetoothReport) {
    if report.has_issues() {
        logging::warn("Bluetooth diagnose: issues found");
        for issue in &report.issues {
            logging::warn(format!("  • {} {:?}", issue.id, issue.name));
        }
    }
}
