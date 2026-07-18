//! Task 2：蓝牙驱动与状态异常诊断

use crate::events::BluetoothStatusEvent;
use crate::events::{BluetoothDeviceEntry, BluetoothIssue};
use crate::notify;
use crate::services::{self, BLUETOOTH};
use crate::settings;
use crate::tray::{self, TrayLevel};
use crate::utils::{device_name, logging, process::CommandExt, wmi_runner};
use chrono::Local;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};
use tokio::time;
use windows::Devices::Bluetooth::GenericAttributeProfile::{
    GattCharacteristicUuids, GattCommunicationStatus, GattServiceUuids,
};
use windows::Devices::Bluetooth::{
    BluetoothCacheMode, BluetoothConnectionStatus, BluetoothDevice, BluetoothLEDevice,
};
use windows::Storage::Streams::DataReader;
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    CM_Get_DevNode_Status, SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo,
    SetupDiGetClassDevsW, SetupDiGetDeviceInstanceIdW, CM_DEVNODE_STATUS_FLAGS, CM_PROB,
    DIGCF_PRESENT, GUID_DEVCLASS_BLUETOOTH, HDEVINFO, SP_DEVINFO_DATA,
};
use windows::Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED};
use wmi::WMIConnection;

static LAST_HEALTHY: OnceLock<Mutex<Option<bool>>> = OnceLock::new();
static DIAGNOSE_CACHE: OnceLock<Mutex<BluetoothDiagnoseCache>> = OnceLock::new();
static BATTERY_REFRESH_RUNNING: AtomicBool = AtomicBool::new(false);
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

    let mut report = diagnose_health()?;
    mark_live_state_pending(&mut report);
    cache.finished_at = Some(Instant::now());
    cache.report = Some(report.clone());
    Ok(report)
}

/// Overview scans only need adapter, driver, and service health. Accessory battery
/// reads can take tens of seconds for disconnected devices, so keep them exclusive
/// to the full Bluetooth panel diagnostic.
pub fn diagnose_health() -> Result<BluetoothReport, String> {
    let started = Instant::now();
    let pnp_started = Instant::now();
    let pnp_report = diagnose_pnp_native().or_else(|native_error| {
        wmi_runner::run(diagnose_pnp_inner)
            .map_err(|wmi_error| format!("native SetupAPI={native_error}; WMI={wmi_error}"))
    });
    let pnp_ms = pnp_started.elapsed().as_millis();
    let mut service_ms = 0_u128;
    let primary = pnp_report.and_then(|mut report| {
        let service_started = Instant::now();
        let service_report =
            services::diagnose_group_native(BLUETOOTH).or_else(|native_error| {
                logging::info(format!(
                    "Bluetooth native service query unavailable; using WMI fallback: {native_error}"
                ));
                services::diagnose_group(BLUETOOTH)
            })?;
        service_ms = service_started.elapsed().as_millis();
        if let Some(service) = service_report.services.first() {
            report.bthserv_state = service.state.clone();
            report.bthserv_start_mode = service.start_mode.clone();
        }
        apply_bthserv_issues(&mut report);
        Ok(report)
    });
    let result = match primary {
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
    };
    let total_ms = started.elapsed().as_millis();
    if total_ms >= 500 {
        logging::info(format!(
            "performance bluetooth_health pnp_ms={pnp_ms} service_ms={service_ms} total_ms={total_ms}"
        ));
    }
    result
}

struct DeviceInfoSet(HDEVINFO);

impl Drop for DeviceInfoSet {
    fn drop(&mut self) {
        unsafe {
            let _ = SetupDiDestroyDeviceInfoList(self.0);
        }
    }
}

fn diagnose_pnp_native() -> Result<BluetoothReport, String> {
    let device_info = DeviceInfoSet(unsafe {
        SetupDiGetClassDevsW(
            Some(&GUID_DEVCLASS_BLUETOOTH),
            windows::core::PCWSTR::null(),
            None,
            DIGCF_PRESENT,
        )
        .map_err(|error| format!("open Bluetooth device set failed: {error}"))?
    });
    let mut report = BluetoothReport::default();
    let mut index = 0_u32;
    loop {
        let mut device = SP_DEVINFO_DATA {
            cbSize: std::mem::size_of::<SP_DEVINFO_DATA>() as u32,
            ..Default::default()
        };
        if unsafe { SetupDiEnumDeviceInfo(device_info.0, index, &mut device) }.is_err() {
            break;
        }
        index += 1;
        let mut instance_buffer = [0_u16; 512];
        if unsafe {
            SetupDiGetDeviceInstanceIdW(device_info.0, &device, Some(&mut instance_buffer), None)
        }
        .is_err()
        {
            continue;
        }
        let instance_length = instance_buffer
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(instance_buffer.len());
        let instance_id = String::from_utf16_lossy(&instance_buffer[..instance_length]);
        let name = device_name::resolve_instance_id(&instance_id)
            .unwrap_or_else(|| "Bluetooth device".into());
        let mut status = CM_DEVNODE_STATUS_FLAGS(0);
        let mut problem = CM_PROB(0);
        let problem_code =
            (unsafe { CM_Get_DevNode_Status(&mut status, &mut problem, device.DevInst, 0) }
                == windows::Win32::Devices::DeviceAndDriverInstallation::CR_SUCCESS)
                .then_some(problem.0);

        if is_bluetooth_adapter_id(&instance_id) {
            report.adapter_devices.push(name.clone());
        }
        match problem_code {
            Some(code) if code != 0 => {
                report.error_devices.push(name.clone());
                report.issues.push(BluetoothIssue {
                    id: "driver_error".into(),
                    name: Some(name.clone()),
                    state: None,
                    code: Some(code),
                });
            }
            None => report.issues.push(BluetoothIssue {
                id: "driver_status_unknown".into(),
                name: Some(name.clone()),
                state: None,
                code: None,
            }),
            _ => {}
        }
        if is_bluetooth_peripheral_id(&instance_id) {
            report.devices.push(BluetoothDeviceEntry {
                name,
                instance_id,
                status: match problem_code {
                    Some(0) => "OK".into(),
                    Some(code) => format!("Error {code}"),
                    None => "Unknown".into(),
                },
                connected: None,
                battery_percent: None,
                battery_state: None,
            });
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

fn diagnose_pnp_inner(wmi: &WMIConnection) -> Result<BluetoothReport, wmi::WMIError> {
    let mut report = BluetoothReport::default();

    let pnp_query = "SELECT Name, PNPClass, ConfigManagerErrorCode, DeviceID FROM Win32_PnPEntity WHERE PNPClass='Bluetooth' AND Present=TRUE";
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
        match dev.error_code {
            Some(code) if code != 0 => {
                let name = dev.name.clone().unwrap_or_else(|| "Unknown".into());
                report.error_devices.push(name.clone());
                report.issues.push(BluetoothIssue {
                    id: "driver_error".into(),
                    name: Some(name),
                    state: None,
                    code: Some(code),
                });
            }
            None => report.issues.push(BluetoothIssue {
                id: "driver_status_unknown".into(),
                name: dev.name.clone(),
                state: None,
                code: None,
            }),
            _ => {}
        }
        if let (Some(name), Some(device_id)) = (&dev.name, &dev.device_id) {
            if is_bluetooth_peripheral_id(device_id) {
                report.devices.push(BluetoothDeviceEntry {
                    name: name.clone(),
                    instance_id: device_id.clone(),
                    status: match dev.error_code {
                        Some(0) => "OK".into(),
                        Some(code) => format!("Error {code}"),
                        None => "Unknown".into(),
                    },
                    connected: None,
                    battery_percent: None,
                    battery_state: None,
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
      problem = if ($null -eq $_.Problem) { $null } else { [uint32]$_.Problem }
    }
  })
  driver_issues = @($allDevices | Where-Object { $null -eq $_.Problem -or [uint32]$_.Problem -ne 0 } | ForEach-Object {
    [pscustomobject]@{
      name = $_.FriendlyName
      problem = if ($null -eq $_.Problem) { $null } else { [uint32]$_.Problem }
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
    if let Some(items) = val.get("driver_issues").and_then(|value| value.as_array()) {
        for item in items {
            let code = item.get("problem").and_then(|value| value.as_u64());
            let name = item
                .get("name")
                .and_then(|value| value.as_str())
                .map(str::to_string);
            if let Some(code) = code.filter(|code| *code != 0) {
                if let Some(name) = name.as_ref() {
                    report.error_devices.push(name.clone());
                }
                report.issues.push(BluetoothIssue {
                    id: "driver_error".into(),
                    name,
                    state: None,
                    code: Some(code.min(u32::MAX as u64) as u32),
                });
            } else if code.is_none() {
                report.issues.push(BluetoothIssue {
                    id: "driver_status_unknown".into(),
                    name,
                    state: None,
                    code: None,
                });
            }
        }
    }
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
                connected: None,
                battery_percent: None,
                battery_state: None,
            })
        })
        .collect())
}

fn mark_live_state_pending(report: &mut BluetoothReport) {
    for device in &mut report.devices {
        if bluetooth_address(&device.instance_id).is_some() {
            device.connected = None;
            device.battery_percent = None;
            device.battery_state = Some("refreshing".into());
        }
    }
}

struct BatteryRefreshGuard(&'static AtomicBool);

impl Drop for BatteryRefreshGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

pub fn refresh_gatt_battery_levels() -> Result<Option<BluetoothReport>, String> {
    if BATTERY_REFRESH_RUNNING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Ok(None);
    }
    let _guard = BatteryRefreshGuard(&BATTERY_REFRESH_RUNNING);
    let mut report = diagnose()?;
    let targets = report
        .devices
        .iter()
        .filter_map(|device| {
            bluetooth_address(&device.instance_id)
                .map(|address| (address, (device.name.clone(), device.instance_id.clone())))
        })
        .collect::<HashMap<_, _>>();
    let live_results = std::thread::scope(|scope| {
        let tasks = targets
            .into_iter()
            .map(|(address, (name, instance_id))| {
                scope.spawn(move || {
                    let result = read_live_bluetooth_state(&instance_id, &address);
                    (address, name, result)
                })
            })
            .collect::<Vec<_>>();
        tasks
            .into_iter()
            .filter_map(|task| task.join().ok())
            .map(|(address, name, result)| (address, (name, result)))
            .collect::<HashMap<_, _>>()
    });
    for device in &mut report.devices {
        let Some(address) = bluetooth_address(&device.instance_id) else {
            continue;
        };
        let Some((name, result)) = live_results.get(&address) else {
            device.battery_state = Some("unavailable".into());
            continue;
        };
        match result {
            Ok(state) => {
                device.connected = Some(state.connected);
                match &state.battery {
                    Ok(Some(percent)) => {
                        device.battery_percent = Some(*percent);
                        device.battery_state = Some("live".into());
                    }
                    Ok(None) => device.battery_state = None,
                    Err(error) => {
                        device.battery_state = Some("unavailable".into());
                        logging::info(format!(
                            "Bluetooth live battery unavailable for {}: {error}",
                            name
                        ));
                    }
                }
            }
            Err(error) => {
                device.battery_state = Some("unavailable".into());
                logging::info(format!(
                    "Bluetooth live state unavailable for {}: {error}",
                    name
                ));
            }
        }
    }
    Ok(Some(report))
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

struct LiveBluetoothState {
    connected: bool,
    battery: Result<Option<u8>, String>,
}

fn read_live_bluetooth_state(
    instance_id: &str,
    address: &str,
) -> Result<LiveBluetoothState, String> {
    let address_value = u64::from_str_radix(address, 16)
        .map_err(|error| format!("invalid Bluetooth address: {error}"))?;
    let _apartment = WinRtApartment::initialize();
    if !instance_id.to_ascii_uppercase().starts_with("BTHLE\\") {
        let device = BluetoothDevice::FromBluetoothAddressAsync(address_value)
            .map_err(|error| format!("open classic device request failed: {error}"))?
            .join()
            .map_err(|error| format!("open classic device failed: {error}"))?;
        let connected = device
            .ConnectionStatus()
            .map_err(|error| format!("classic connection status failed: {error}"))?
            == BluetoothConnectionStatus::Connected;
        return Ok(LiveBluetoothState {
            connected,
            battery: Ok(None),
        });
    }
    let device = BluetoothLEDevice::FromBluetoothAddressAsync(address_value)
        .map_err(|error| format!("open device request failed: {error}"))?
        .join()
        .map_err(|error| format!("open device failed: {error}"))?;
    let connected = device
        .ConnectionStatus()
        .map_err(|error| format!("connection status failed: {error}"))?
        == BluetoothConnectionStatus::Connected;
    if !connected {
        return Ok(LiveBluetoothState {
            connected: false,
            battery: Ok(None),
        });
    }
    let battery = read_gatt_battery_level(&device);
    Ok(LiveBluetoothState { connected, battery })
}

fn read_gatt_battery_level(device: &BluetoothLEDevice) -> Result<Option<u8>, String> {
    let service_uuid = GattServiceUuids::Battery()
        .map_err(|error| format!("battery service UUID unavailable: {error}"))?;
    let services_result = device
        .GetGattServicesForUuidWithCacheModeAsync(service_uuid, BluetoothCacheMode::Uncached)
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
                BluetoothCacheMode::Uncached,
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
                .ReadValueWithCacheModeAsync(BluetoothCacheMode::Uncached)
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

fn validate_bluetooth_peripheral_id(instance_id: &str) -> Result<(), String> {
    if instance_id.trim() != instance_id
        || instance_id.is_empty()
        || instance_id.len() > 512
        || instance_id
            .chars()
            .any(|character| matches!(character, '\r' | '\n' | '\0'))
        || !is_bluetooth_peripheral_id(instance_id)
    {
        return Err("Invalid Bluetooth peripheral instance identifier.".into());
    }
    Ok(())
}

/// Destructive device operations must not trust an instance id supplied by the
/// WebView. Re-read the present Bluetooth class immediately before acting and
/// refuse the operation if the target disappeared or belongs to another class.
fn validate_present_bluetooth_peripheral(instance_id: &str) -> Result<(), String> {
    validate_bluetooth_peripheral_id(instance_id)?;
    let report = diagnose_pnp_native().or_else(|native_error| {
        wmi_runner::run(diagnose_pnp_inner)
            .map_err(|wmi_error| format!("native SetupAPI={native_error}; WMI={wmi_error}"))
    })?;
    if report
        .devices
        .iter()
        .any(|device| device.instance_id.eq_ignore_ascii_case(instance_id))
    {
        Ok(())
    } else {
        Err(
            "The selected Bluetooth device is no longer present. Scan again before changing it."
                .into(),
        )
    }
}

pub fn remove_device(instance_id: &str) -> Result<(), String> {
    validate_present_bluetooth_peripheral(instance_id)?;
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
    validate_present_bluetooth_peripheral(instance_id)?;
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

    #[test]
    fn destructive_actions_accept_only_bluetooth_peripheral_ids() {
        assert!(validate_bluetooth_peripheral_id(
            r"BTHLE\DEV_D92825265D4F\D&B66BFB4&0&D92825265D4F"
        )
        .is_ok());
        assert!(validate_bluetooth_peripheral_id(r"USB\VID_0489&PID_E13A\1").is_err());
        assert!(validate_bluetooth_peripheral_id("BTHLE\\DEV_1234\nUSB\\OTHER").is_err());
        assert!(validate_bluetooth_peripheral_id(" BTHENUM\\DEV_1234").is_err());
    }

    #[test]
    fn pending_live_check_never_reuses_a_previous_connection_or_battery_value() {
        let mut report = BluetoothReport {
            devices: vec![BluetoothDeviceEntry {
                name: "Test device".into(),
                instance_id: r"BTHLE\DEV_D92825265D4F\D&B66BFB4&0&D92825265D4F".into(),
                status: "OK".into(),
                connected: Some(true),
                battery_percent: Some(80),
                battery_state: Some("live".into()),
            }],
            ..Default::default()
        };
        mark_live_state_pending(&mut report);
        let device = &report.devices[0];
        assert_eq!(device.connected, None);
        assert_eq!(device.battery_percent, None);
        assert_eq!(device.battery_state.as_deref(), Some("refreshing"));
    }
}
