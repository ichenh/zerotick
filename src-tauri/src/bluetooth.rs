//! Task 2：蓝牙驱动与状态异常诊断

use crate::events::BluetoothStatusEvent;
use crate::events::{BluetoothDeviceEntry, BluetoothIssue};
use crate::notify;
use crate::services::{self, BLUETOOTH};
use crate::settings;
use crate::tray::{self, TrayLevel};
use crate::utils::{logging, process::CommandExt, wmi_runner};
use chrono::Local;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::time;
use windows::Devices::Bluetooth::GenericAttributeProfile::{
    GattCharacteristicUuids, GattCommunicationStatus, GattServiceUuids,
};
use windows::Devices::Bluetooth::{BluetoothCacheMode, BluetoothLEDevice};
use windows::Storage::Streams::DataReader;
use windows::Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED};
use wmi::WMIConnection;

static LAST_HEALTHY: OnceLock<Mutex<Option<bool>>> = OnceLock::new();
static DIAGNOSE_CACHE: OnceLock<Mutex<BluetoothDiagnoseCache>> = OnceLock::new();
const DIAGNOSE_CACHE_TTL: Duration = Duration::from_secs(2);

#[derive(Default)]
struct BluetoothDiagnoseCache {
    finished_at: Option<Instant>,
    report: Option<BluetoothReport>,
}

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

#[derive(Debug, Default, Clone)]
pub struct BluetoothReport {
    pub adapter_devices: Vec<String>,
    pub error_devices: Vec<String>,
    pub bthserv_state: Option<String>,
    pub bthserv_start_mode: Option<String>,
    pub issues: Vec<BluetoothIssue>,
    pub devices: Vec<BluetoothDeviceEntry>,
}

impl BluetoothReport {
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }
}

pub fn diagnose() -> Result<BluetoothReport, String> {
    let cache = DIAGNOSE_CACHE.get_or_init(|| Mutex::new(BluetoothDiagnoseCache::default()));
    let mut cache = cache
        .lock()
        .map_err(|_| "Bluetooth diagnostic cache lock failed".to_string())?;
    if let (Some(finished_at), Some(report)) = (cache.finished_at, cache.report.as_ref()) {
        if finished_at.elapsed() <= DIAGNOSE_CACHE_TTL {
            return Ok(report.clone());
        }
    }

    let primary = wmi_runner::run(diagnose_pnp_inner).and_then(|mut report| {
        let service_report = services::diagnose_group(BLUETOOTH)?;
        if let Some(service) = service_report.services.first() {
            report.bthserv_state = service.state.clone();
            report.bthserv_start_mode = service.start_mode.clone();
        }
        apply_bthserv_issues(&mut report);
        Ok(report)
    });
    let mut report = match primary {
        Ok(report) => Ok(report),
        Err(e) => match diagnose_powershell() {
            Ok(report) => {
                logging::info(format!(
                    "Bluetooth WMI 兼容路径不可用，PowerShell 后备诊断成功: {e}"
                ));
                Ok(report)
            }
            Err(fallback_error) => {
                logging::warn(format!(
                    "Bluetooth 诊断失败: WMI={e}; PowerShell={fallback_error}"
                ));
                Err(fallback_error)
            }
        },
    }?;
    if let Err(error) = attach_battery_levels(&mut report) {
        logging::info(format!("Bluetooth battery levels are unavailable: {error}"));
    }
    cache.finished_at = Some(Instant::now());
    cache.report = Some(report.clone());
    Ok(report)
}

fn diagnose_pnp_inner(wmi: &WMIConnection) -> Result<BluetoothReport, wmi::WMIError> {
    let mut report = BluetoothReport::default();

    let pnp_query = "SELECT Name, PNPClass, ConfigManagerErrorCode, DeviceID FROM Win32_PnPEntity WHERE PNPClass='Bluetooth'";
    // 查询失败必须向上传递，让 diagnose() 进入 PowerShell 备用路径；
    // 不能把权限错误吞成空列表，否则会误报“没有蓝牙适配器”。
    let devices: Vec<PnPDevice> = wmi.raw_query(pnp_query).map_err(|error| {
        logging::info(format!("Bluetooth WMI 失败阶段=PnP 设备查询: {error}"));
        error
    })?;

    for dev in &devices {
        if let (Some(name), Some(device_id)) = (&dev.name, &dev.device_id) {
            if is_bluetooth_adapter_id(device_id) {
                report.adapter_devices.push(name.clone());
            }
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
        if let (Some(name), Some(device_id)) = (&dev.name, &dev.device_id) {
            if is_bluetooth_peripheral_id(device_id) {
                let code = dev.error_code.unwrap_or(0);
                report.devices.push(BluetoothDeviceEntry {
                    name: name.clone(),
                    instance_id: device_id.clone(),
                    status: if code == 0 {
                        "OK".to_string()
                    } else {
                        format!("Error {code}")
                    },
                    connected: code == 0,
                    battery_percent: None,
                });
            }
        }
    }
    if report.adapter_devices.is_empty() {
        report.issues.push(BluetoothIssue {
            id: "no_radio".into(),
            name: None,
            state: None,
            code: None,
        });
    }
    Ok(report)
}

pub fn invalidate_diagnostic_cache() {
    if let Some(cache) = DIAGNOSE_CACHE.get() {
        if let Ok(mut cache) = cache.lock() {
            cache.finished_at = None;
            cache.report = None;
        }
    }
}

fn diagnose_powershell() -> Result<BluetoothReport, String> {
    let mut report = BluetoothReport::default();
    let script = r#"
$allDevices = @(Get-PnpDevice -Class Bluetooth -PresentOnly -ErrorAction Stop)
$adapters = @($allDevices | Where-Object { ([string]$_.InstanceId) -match '^(USB|PCI|ACPI|ROOT)\\' })
$svc = Get-Service -Name bthserv -ErrorAction Stop
[pscustomobject]@{
  radios = @($adapters | ForEach-Object { $_.FriendlyName } | Where-Object { $_ })
  bthserv_state = if ($svc) { [string]$svc.Status } else { $null }
  bthserv_start_mode = if ($svc) { [string]$svc.StartType } else { $null }
  devices = @($allDevices | ForEach-Object {
    $name = $_.FriendlyName
    if (-not $name) { return }
    $id = [string]$_.InstanceId
    if ($id -notmatch '^(BTHLE|BTHENUM)\\DEV_') { return }
    [pscustomobject]@{
      name = $name
      instance_id = $_.InstanceId
      status = [string]$_.Status
      connected = ($_.Status -eq 'OK')
    }
  })
}
"#;
    let val = crate::utils::powershell::run_json(script)?;
    if let Some(arr) = val.get("radios").and_then(|v| v.as_array()) {
        for item in arr {
            if let Some(name) = item.as_str() {
                report.adapter_devices.push(name.to_string());
            }
        }
    }
    if report.adapter_devices.is_empty() {
        report.issues.push(BluetoothIssue {
            id: "no_radio".into(),
            name: None,
            state: None,
            code: None,
        });
    }
    report.bthserv_state = val
        .get("bthserv_state")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    report.bthserv_start_mode = val
        .get("bthserv_start_mode")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    apply_bthserv_issues(&mut report);
    report.devices = val
        .get("devices")
        .map(|v| parse_device_entries(v.clone()))
        .transpose()?
        .unwrap_or_default();
    Ok(report)
}

fn parse_device_entries(val: serde_json::Value) -> Result<Vec<BluetoothDeviceEntry>, String> {
    let arr = match val {
        serde_json::Value::Array(a) => a,
        serde_json::Value::Object(_) => vec![val],
        serde_json::Value::Null => vec![],
        _ => return Err("蓝牙设备列表格式异常".into()),
    };
    Ok(arr
        .into_iter()
        .filter_map(|item| {
            Some(BluetoothDeviceEntry {
                name: item.get("name")?.as_str()?.to_string(),
                instance_id: item.get("instance_id")?.as_str()?.to_string(),
                status: item
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                connected: item
                    .get("connected")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                battery_percent: None,
            })
        })
        .collect())
}

fn attach_battery_levels(report: &mut BluetoothReport) -> Result<(), String> {
    let mut query_errors = Vec::new();
    let levels = match cached_battery_levels() {
        Ok(levels) => levels,
        Err(error) => {
            query_errors.push(format!("Windows property cache: {error}"));
            HashMap::new()
        }
    };
    let mut gatt_levels = HashMap::<String, Option<u8>>::new();
    for device in &mut report.devices {
        let Some(address) = bluetooth_address(&device.instance_id) else {
            continue;
        };
        if let Some(percent) = levels.get(&address).copied() {
            device.battery_percent = Some(percent);
            continue;
        }

        let percent = if let Some(percent) = gatt_levels.get(&address) {
            *percent
        } else {
            let result = read_gatt_battery_level(&address);
            if let Err(error) = &result {
                query_errors.push(format!("{} ({address}): {error}", device.name));
            }
            let percent = result.unwrap_or(None);
            gatt_levels.insert(address.clone(), percent);
            percent
        };
        device.battery_percent = percent;
    }

    if report
        .devices
        .iter()
        .any(|device| device.battery_percent.is_some())
        || query_errors.is_empty()
    {
        Ok(())
    } else {
        Err(query_errors.join("; "))
    }
}

fn cached_battery_levels() -> Result<HashMap<String, u8>, String> {
    let script = r#"
$levels = @{}
function Add-BatteryLevel($instanceId, $value) {
  if (-not $instanceId -or $null -eq $value) { return }
  $number = 0
  if (-not [int]::TryParse(([string]$value), [ref]$number)) { return }
  if ($number -lt 0 -or $number -gt 100) { return }
  if ([string]$instanceId -match '(?i)DEV_([0-9A-F]{12})') {
    $levels[$Matches[1].ToUpperInvariant()] = $number
  }
}

# Windows exposes BAS/HFP battery data as a device property when the accessory
# and its driver support reporting it. Battery-service child nodes are folded
# into the parent device below by their shared Bluetooth address.
Get-PnpDevice -PresentOnly -ErrorAction SilentlyContinue | ForEach-Object {
  $battery = Get-PnpDeviceProperty -InstanceId $_.InstanceId -KeyName 'DEVPKEY_Device_BatteryLevel' -ErrorAction SilentlyContinue
  if ($battery) { Add-BatteryLevel $_.InstanceId $battery.Data }
}

# Some classic Bluetooth drivers cache the HFP battery indicator here instead
# of publishing DEVPKEY_Device_BatteryLevel.
$cacheRoot = 'HKLM:\SYSTEM\CurrentControlSet\Services\BTHPORT\Parameters\Devices'
if (Test-Path -LiteralPath $cacheRoot) {
  Get-ChildItem -LiteralPath $cacheRoot -ErrorAction SilentlyContinue | ForEach-Object {
    $properties = Get-ItemProperty -LiteralPath $_.PSPath -ErrorAction SilentlyContinue
    $battery = if ($properties) { $properties.PSObject.Properties['BatteryLevel'] } else { $null }
    if ($battery) { Add-BatteryLevel "DEV_$($_.PSChildName)" $battery.Value }
  }
}

@($levels.GetEnumerator() | ForEach-Object {
  [pscustomobject]@{ address = $_.Key; battery_percent = $_.Value }
})
"#;
    let value = crate::utils::powershell::run_json(script)?;
    let items = match value {
        serde_json::Value::Array(items) => items,
        serde_json::Value::Object(_) => vec![value],
        serde_json::Value::Null => Vec::new(),
        _ => return Err("Bluetooth battery result has an invalid format".into()),
    };
    Ok(items
        .into_iter()
        .filter_map(|item| {
            let address = item.get("address")?.as_str()?.to_ascii_uppercase();
            let percent = item.get("battery_percent")?.as_u64()?;
            (percent <= 100).then_some((address, percent as u8))
        })
        .collect())
}

struct WinRtApartment(bool);

impl WinRtApartment {
    fn initialize() -> Self {
        Self(unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.is_ok())
    }
}

impl Drop for WinRtApartment {
    fn drop(&mut self) {
        if self.0 {
            unsafe { RoUninitialize() };
        }
    }
}

fn read_gatt_battery_level(address: &str) -> Result<Option<u8>, String> {
    let address_value = u64::from_str_radix(address, 16)
        .map_err(|error| format!("invalid Bluetooth address: {error}"))?;
    let _apartment = WinRtApartment::initialize();
    let device = BluetoothLEDevice::FromBluetoothAddressAsync(address_value)
        .map_err(|error| format!("open device request failed: {error}"))?
        .join()
        .map_err(|error| format!("open device failed: {error}"))?;
    let service_uuid = GattServiceUuids::Battery()
        .map_err(|error| format!("battery service UUID unavailable: {error}"))?;
    let services_result = device
        .GetGattServicesForUuidWithCacheModeAsync(service_uuid, BluetoothCacheMode::Cached)
        .map_err(|error| format!("battery service request failed: {error}"))?
        .join()
        .map_err(|error| format!("battery service query failed: {error}"))?;
    if services_result
        .Status()
        .map_err(|error| format!("battery service status failed: {error}"))?
        != GattCommunicationStatus::Success
    {
        return Ok(None);
    }

    let characteristic_uuid = GattCharacteristicUuids::BatteryLevel()
        .map_err(|error| format!("battery characteristic UUID unavailable: {error}"))?;
    let services = services_result
        .Services()
        .map_err(|error| format!("battery services unavailable: {error}"))?;
    for index in 0..services
        .Size()
        .map_err(|error| format!("battery service count unavailable: {error}"))?
    {
        let service = services
            .GetAt(index)
            .map_err(|error| format!("battery service unavailable: {error}"))?;
        let characteristics_result = service
            .GetCharacteristicsForUuidWithCacheModeAsync(
                characteristic_uuid,
                BluetoothCacheMode::Cached,
            )
            .map_err(|error| format!("battery characteristic request failed: {error}"))?
            .join()
            .map_err(|error| format!("battery characteristic query failed: {error}"))?;
        if characteristics_result
            .Status()
            .map_err(|error| format!("battery characteristic status failed: {error}"))?
            != GattCommunicationStatus::Success
        {
            continue;
        }
        let characteristics = characteristics_result
            .Characteristics()
            .map_err(|error| format!("battery characteristics unavailable: {error}"))?;
        for characteristic_index in 0..characteristics
            .Size()
            .map_err(|error| format!("battery characteristic count unavailable: {error}"))?
        {
            let characteristic = characteristics
                .GetAt(characteristic_index)
                .map_err(|error| format!("battery characteristic unavailable: {error}"))?;
            let read_result = characteristic
                .ReadValueWithCacheModeAsync(BluetoothCacheMode::Cached)
                .map_err(|error| format!("battery read request failed: {error}"))?
                .join()
                .map_err(|error| format!("battery read failed: {error}"))?;
            if read_result
                .Status()
                .map_err(|error| format!("battery read status failed: {error}"))?
                != GattCommunicationStatus::Success
            {
                continue;
            }
            let buffer = read_result
                .Value()
                .map_err(|error| format!("battery value unavailable: {error}"))?;
            if buffer
                .Length()
                .map_err(|error| format!("battery value length unavailable: {error}"))?
                == 0
            {
                continue;
            }
            let reader = DataReader::FromBuffer(&buffer)
                .map_err(|error| format!("battery value reader failed: {error}"))?;
            let percent = reader
                .ReadByte()
                .map_err(|error| format!("battery value read failed: {error}"))?;
            if percent <= 100 {
                return Ok(Some(percent));
            }
        }
    }
    Ok(None)
}

fn bluetooth_address(instance_id: &str) -> Option<String> {
    let upper = instance_id.to_ascii_uppercase();
    let tail = upper.split("DEV_").nth(1)?;
    let address = tail.get(..12)?;
    address
        .chars()
        .all(|character| character.is_ascii_hexdigit())
        .then(|| address.to_string())
}

fn apply_bthserv_issues(report: &mut BluetoothReport) {
    let has_service = report.bthserv_state.is_some() || report.bthserv_start_mode.is_some();
    if !has_service {
        report.issues.push(BluetoothIssue {
            id: "bthserv_missing".into(),
            name: None,
            state: None,
            code: None,
        });
        return;
    }
    match report.bthserv_state.as_deref() {
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
    if report.bthserv_start_mode.as_deref() == Some("Disabled") {
        report.issues.push(BluetoothIssue {
            id: "bthserv_disabled".into(),
            name: None,
            state: None,
            code: None,
        });
    }
}

fn is_bluetooth_adapter_id(device_id: &str) -> bool {
    let upper = device_id.to_ascii_uppercase();
    ["USB\\", "PCI\\", "ACPI\\", "ROOT\\"]
        .iter()
        .any(|prefix| upper.starts_with(prefix))
}

fn is_bluetooth_peripheral_id(device_id: &str) -> bool {
    let upper = device_id.to_ascii_uppercase();
    upper.starts_with("BTHLE\\DEV_") || upper.starts_with("BTHENUM\\DEV_")
}

pub fn remove_device(instance_id: &str) -> Result<(), String> {
    let output = std::process::Command::new("pnputil")
        .hide_window()
        .args(["/remove-device", instance_id, "/force"])
        .output()
        .map_err(|e| format!("pnputil 失败: {e}"))?;
    if output.status.success() {
        invalidate_diagnostic_cache();
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Err(if stderr.is_empty() { stdout } else { stderr })
    }
}

pub fn reconnect_device(instance_id: &str) -> Result<(), String> {
    let id = instance_id.replace('\'', "''");
    let script = format!(
        "$ErrorActionPreference='Stop'; Disable-PnpDevice -InstanceId '{id}' -Confirm:$false -ErrorAction Stop; Start-Sleep -Milliseconds 500; Enable-PnpDevice -InstanceId '{id}' -Confirm:$false -ErrorAction Stop"
    );
    crate::utils::powershell::run_void(&script)?;
    invalidate_diagnostic_cache();
    Ok(())
}

pub fn repair_service() -> (Vec<String>, Vec<String>) {
    let result = services::repair_group(BLUETOOTH);
    invalidate_diagnostic_cache();
    result
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
    match tokio::task::spawn_blocking(diagnose).await {
        Ok(Ok(report)) => {
            emit_report(&report);
            emit_status_event(app, &report);
        }
        Ok(Err(e)) => logging::error(format!("Bluetooth diagnose failed: {e}")),
        Err(e) => logging::error(format!("Bluetooth diagnose task failed: {e}")),
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
        adapter_count: report.adapter_devices.len(),
        adapters: report.adapter_devices.clone(),
        devices: report.devices.clone(),
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
        notify::send_if_background(app, &crate::i18n::notify_bluetooth_title(&locale), &detail);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinguishes_adapter_from_bluetooth_children() {
        assert!(is_bluetooth_adapter_id(
            r"USB\VID_0489&PID_E13A&MI_00\B&26E6BFF2&0&0000"
        ));
        assert!(!is_bluetooth_adapter_id(
            r"BTHLE\DEV_D92825265D4F\D&B66BFB4&0&D92825265D4F"
        ));
    }

    #[test]
    fn keeps_only_real_peripherals() {
        assert!(is_bluetooth_peripheral_id(
            r"BTHLE\DEV_D92825265D4F\D&B66BFB4&0&D92825265D4F"
        ));
        assert!(is_bluetooth_peripheral_id(
            r"BTHENUM\DEV_605556B971C0\D&304C4053&0&BLUETOOTHDEVICE_605556B971C0"
        ));
        assert!(!is_bluetooth_peripheral_id(
            r"BTHLEDEVICE\{0000180F-0000-1000-8000-00805F9B34FB}_DEV_X"
        ));
        assert!(!is_bluetooth_peripheral_id(r"BTH\MS_BTHLE\C&1BA46DC9&2&3"));
    }

    #[test]
    fn extracts_bluetooth_address_for_battery_matching() {
        assert_eq!(
            bluetooth_address(r"BTHLE\DEV_D92825265D4F\D&B66BFB4&0&D92825265D4F"),
            Some("D92825265D4F".into())
        );
        assert_eq!(bluetooth_address(r"USB\VID_0489&PID_E13A"), None);
    }
}
