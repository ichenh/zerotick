//! Windows 设备与驱动诊断：聚合常用硬件类别，并把设备管理器错误转成稳定的语义标识。

use crate::utils::{device_name, elevated, process::CommandExt, wmi_runner};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;
use wmi::WMIConnection;

#[derive(Debug, Deserialize)]
struct PnpEntity {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "PNPClass")]
    class_name: Option<String>,
    #[serde(rename = "DeviceID")]
    device_id: Option<String>,
    #[serde(rename = "ConfigManagerErrorCode")]
    error_code: Option<u32>,
    #[serde(rename = "Status")]
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PnpSignedDriver {
    #[serde(rename = "DeviceID")]
    device_id: Option<String>,
    #[serde(rename = "DriverProviderName")]
    provider: Option<String>,
    #[serde(rename = "DriverVersion")]
    version: Option<String>,
    #[serde(rename = "DriverDate")]
    date: Option<String>,
    #[serde(rename = "InfName")]
    inf_name: Option<String>,
    #[serde(rename = "IsSigned")]
    is_signed: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DriverEvidence {
    pub provider: Option<String>,
    pub version: Option<String>,
    pub date: Option<String>,
    pub inf_name: Option<String>,
    pub is_signed: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct DeviceClassSummary {
    pub id: String,
    pub count: usize,
}

#[derive(Debug, Serialize)]
pub struct DeviceProblem {
    pub name: String,
    pub class_id: String,
    pub device_id: String,
    pub error_code: u32,
    pub reason_id: String,
    pub status: Option<String>,
    pub driver: Option<DriverEvidence>,
    pub available_actions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DevicesDiagReport {
    pub classes: Vec<DeviceClassSummary>,
    pub problems: Vec<DeviceProblem>,
    pub network_missing: bool,
    pub display_missing: bool,
}

#[derive(Debug, Serialize)]
pub struct DeviceRescanResult {
    pub success: bool,
    pub needs_admin: bool,
    pub details: String,
}

#[derive(Debug, Serialize)]
pub struct DeviceRepairResult {
    pub action_id: String,
    pub command_succeeded: bool,
    pub verified: bool,
    pub needs_admin: bool,
    pub reboot_required: bool,
    pub before_code: u32,
    pub after_code: Option<u32>,
    pub device_present: bool,
    pub details: String,
}

pub fn diagnose() -> Result<DevicesDiagReport, String> {
    wmi_runner::run(diagnose_inner)
}

fn diagnose_inner(wmi: &WMIConnection) -> Result<DevicesDiagReport, wmi::WMIError> {
    // 只枚举当前产品关心的常用硬件和当前异常项，避免扫描整个 PnP 树。
    let query = "SELECT Name, PNPClass, DeviceID, ConfigManagerErrorCode, Status \
                 FROM Win32_PnPEntity \
                 WHERE Present=TRUE AND (ConfigManagerErrorCode <> 0 \
                    OR PNPClass='Net' OR PNPClass='Display' OR PNPClass='Camera' \
                    OR PNPClass='Bluetooth' OR PNPClass='USB')";
    let entities: Vec<PnpEntity> = wmi.raw_query(query)?;
    let driver_by_device = load_problem_driver_evidence(wmi, &entities);

    let class_ids = ["network", "display", "camera", "bluetooth", "usb"];
    let mut counts = [0usize; 5];
    let mut problems = Vec::new();

    for entity in entities {
        let class_id = normalize_class(entity.class_name.as_deref());
        if let Some(index) = class_ids.iter().position(|id| *id == class_id) {
            counts[index] += 1;
        }
        let code = entity.error_code.unwrap_or(0);
        // 45 表示历史设备当前未连接；Present 过滤后通常不会出现，仍保留保护。
        if code != 0 && code != 45 {
            let device_id = entity.device_id.unwrap_or_default();
            let raw_name = entity.name.unwrap_or_else(|| "Unknown device".into());
            let name = device_name::resolve_instance_id(&device_id)
                .unwrap_or_else(|| clarify_usb_device_name(&raw_name));
            let driver = driver_by_device
                .get(&device_id.to_ascii_uppercase())
                .cloned();
            problems.push(DeviceProblem {
                name,
                class_id: class_id.into(),
                device_id,
                error_code: code,
                reason_id: error_reason(code).into(),
                status: entity.status,
                available_actions: available_actions(code, driver.as_ref()),
                driver,
            });
        }
    }

    let classes = class_ids
        .iter()
        .zip(counts)
        .map(|(id, count)| DeviceClassSummary {
            id: (*id).into(),
            count,
        })
        .collect::<Vec<_>>();

    Ok(DevicesDiagReport {
        network_missing: counts[0] == 0,
        display_missing: counts[1] == 0,
        classes,
        problems,
    })
}

fn load_problem_driver_evidence(
    wmi: &WMIConnection,
    entities: &[PnpEntity],
) -> HashMap<String, DriverEvidence> {
    let mut device_ids = entities
        .iter()
        .filter(|entity| {
            let code = entity.error_code.unwrap_or(0);
            code != 0 && code != 45
        })
        .filter_map(|entity| entity.device_id.as_deref())
        .map(str::to_string)
        .collect::<Vec<_>>();
    device_ids.sort_unstable_by_key(|id| id.to_ascii_uppercase());
    device_ids.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    if device_ids.is_empty() {
        return HashMap::new();
    }

    let mut evidence = HashMap::new();
    for ids in device_ids.chunks(16) {
        let conditions = ids
            .iter()
            .map(|id| format!("DeviceID='{}'", id.replace('\'', "''")))
            .collect::<Vec<_>>()
            .join(" OR ");
        let query = format!(
            "SELECT DeviceID, DriverProviderName, DriverVersion, DriverDate, InfName, IsSigned \
             FROM Win32_PnPSignedDriver WHERE {conditions}"
        );
        let drivers: Vec<PnpSignedDriver> = match wmi.raw_query(&query) {
            Ok(drivers) => drivers,
            Err(error) => {
                crate::utils::logging::error(format!(
                    "targeted driver metadata query failed; keeping device error evidence: {error}"
                ));
                continue;
            }
        };
        evidence.extend(drivers.into_iter().filter_map(|driver| {
            let id = driver.device_id.as_ref()?.to_ascii_uppercase();
            Some((
                id,
                DriverEvidence {
                    provider: driver.provider,
                    version: driver.version,
                    date: driver.date,
                    inf_name: driver.inf_name,
                    is_signed: driver.is_signed,
                },
            ))
        }));
    }
    evidence
}

fn available_actions(code: u32, driver: Option<&DriverEvidence>) -> Vec<String> {
    let mut actions = Vec::new();
    if code == 22 {
        actions.push("enable".into());
    } else if code != 28 {
        actions.push("restart".into());
    }
    if code != 28 && driver.and_then(|item| item.inf_name.as_deref()).is_some() {
        actions.push("reinstall_store".into());
    }
    actions.push("install_inf".into());
    actions
}

fn clarify_usb_device_name(name: &str) -> String {
    // Hardware ids and the raw Windows class name remain available in advanced
    // mode. Ordinary mode must not present a generic driver-node label as if it
    // were the product name.
    match name.trim().to_ascii_lowercase().as_str() {
        "usb composite device" | "usb device" | "unknown usb device" => "USB device".into(),
        _ => name.to_string(),
    }
}

fn normalize_class(class_name: Option<&str>) -> &'static str {
    match class_name.unwrap_or_default().to_ascii_lowercase().as_str() {
        "net" => "network",
        "display" => "display",
        "camera" | "image" => "camera",
        "bluetooth" => "bluetooth",
        "usb" => "usb",
        _ => "other",
    }
}

fn error_reason(code: u32) -> &'static str {
    match code {
        10 => "cannot_start",
        22 => "disabled",
        28 => "driver_missing",
        31 => "driver_load_failed",
        43 => "device_reported_problem",
        _ => "other",
    }
}

pub fn rescan() -> Result<DeviceRescanResult, String> {
    let output = Command::new("pnputil.exe")
        .hide_window()
        .args(["/scan-devices"])
        .output()
        .map_err(|error| format!("无法启动 Windows 设备扫描: {error}"))?;
    let details = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stderr.is_empty() {
            stdout
        } else {
            stderr
        }
    };
    Ok(DeviceRescanResult {
        success: output.status.success(),
        needs_admin: !output.status.success() && !elevated::is_elevated(),
        details,
    })
}

pub fn repair_device(
    device_id: &str,
    action_id: &str,
    inf_path: Option<&str>,
) -> Result<DeviceRepairResult, String> {
    validate_device_id(device_id)?;
    if !elevated::is_elevated() {
        let current_code = current_problem_code(device_id)?;
        return Ok(DeviceRepairResult {
            action_id: action_id.into(),
            command_succeeded: false,
            verified: false,
            needs_admin: true,
            reboot_required: false,
            before_code: current_code.unwrap_or(0),
            after_code: current_code,
            device_present: current_code.is_some(),
            details: "Administrator privileges are required for device driver changes.".into(),
        });
    }

    let before_code = current_problem_code(device_id)?.ok_or_else(|| {
        "The selected device is no longer present. Scan again before repairing.".to_string()
    })?;
    if before_code == 0 {
        return Err("The selected device no longer reports an error. No change was made.".into());
    }
    let args = match action_id {
        "enable" if before_code == 22 => vec!["/enable-device".into(), device_id.into()],
        "restart" if before_code != 22 && before_code != 28 => {
            vec!["/restart-device".into(), device_id.into()]
        }
        "reinstall_store" if before_code != 28 && has_stored_driver(device_id)? => {
            vec!["/remove-device".into(), device_id.into()]
        }
        "install_inf" => {
            let path = inf_path.ok_or_else(|| "No driver INF file was selected.".to_string())?;
            validate_inf_path(path)?;
            vec!["/add-driver".into(), path.into(), "/install".into()]
        }
        _ => return Err("This repair action does not apply to the device's current state.".into()),
    };

    let output = run_pnputil(&args)?;
    let mut command_succeeded = command_completed(&output);
    let mut reboot_required = output.status.code() == Some(3010);
    let mut details = command_details(&output);
    if command_succeeded && action_id == "reinstall_store" {
        let scan = run_pnputil(&["/scan-devices".into()])?;
        command_succeeded = command_completed(&scan);
        reboot_required |= scan.status.code() == Some(3010);
        if !command_completed(&scan) {
            details.push_str("\nHardware rescan failed:\n");
            details.push_str(&command_details(&scan));
        }
    }

    let mut after_code = current_problem_code(device_id)?;
    if command_succeeded {
        for _ in 0..5 {
            if after_code == Some(0) {
                break;
            }
            thread::sleep(Duration::from_millis(500));
            after_code = current_problem_code(device_id)?;
        }
    }
    let device_present = after_code.is_some();
    let verified = command_succeeded && after_code == Some(0);
    Ok(DeviceRepairResult {
        action_id: action_id.into(),
        command_succeeded,
        verified,
        needs_admin: false,
        reboot_required,
        before_code,
        after_code,
        device_present,
        details,
    })
}

fn validate_device_id(device_id: &str) -> Result<(), String> {
    if device_id.trim().is_empty()
        || device_id.len() > 512
        || device_id
            .chars()
            .any(|character| matches!(character, '\r' | '\n' | '\0'))
    {
        return Err("Invalid device instance identifier.".into());
    }
    Ok(())
}

fn validate_inf_path(path: &str) -> Result<(), String> {
    let path = Path::new(path);
    if !path.is_file()
        || !path
            .extension()
            .is_some_and(|extension| extension.to_string_lossy().eq_ignore_ascii_case("inf"))
    {
        return Err("Select an existing .inf driver package file.".into());
    }
    Ok(())
}

fn run_pnputil(args: &[String]) -> Result<std::process::Output, String> {
    Command::new("pnputil.exe")
        .hide_window()
        .args(args)
        .output()
        .map_err(|error| format!("Unable to start Windows driver management: {error}"))
}

fn command_completed(output: &std::process::Output) -> bool {
    output.status.success() || output.status.code() == Some(3010)
}

fn command_details(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        stdout
    } else if stdout.is_empty() {
        stderr
    } else {
        format!("{stdout}\n{stderr}")
    }
}

fn current_problem_code(device_id: &str) -> Result<Option<u32>, String> {
    let wanted = device_id.to_ascii_uppercase();
    wmi_runner::run(move |wmi| {
        let entities: Vec<PnpEntity> = wmi.raw_query(
            "SELECT Name, PNPClass, DeviceID, ConfigManagerErrorCode, Status FROM Win32_PnPEntity WHERE Present=TRUE",
        )?;
        Ok(entities.into_iter().find_map(|entity| {
            (entity.device_id?.to_ascii_uppercase() == wanted)
                .then_some(entity.error_code.unwrap_or(0))
        }))
    })
}

fn has_stored_driver(device_id: &str) -> Result<bool, String> {
    let escaped = device_id.replace('\'', "''");
    wmi_runner::run(move |wmi| {
        let query = format!(
            "SELECT DeviceID, DriverProviderName, DriverVersion, DriverDate, InfName, IsSigned \
             FROM Win32_PnPSignedDriver WHERE DeviceID='{escaped}'"
        );
        let drivers: Vec<PnpSignedDriver> = wmi.raw_query(&query)?;
        Ok(drivers
            .into_iter()
            .any(|driver| driver.inf_name.is_some_and(|name| !name.trim().is_empty())))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_common_device_manager_codes() {
        assert_eq!(error_reason(10), "cannot_start");
        assert_eq!(error_reason(28), "driver_missing");
        assert_eq!(error_reason(43), "device_reported_problem");
        assert_eq!(error_reason(999), "other");
    }

    #[test]
    fn does_not_expose_hardware_ids_in_the_display_name() {
        assert_eq!(
            clarify_usb_device_name("USB Composite Device"),
            "USB device"
        );
    }

    #[test]
    fn offers_only_actions_that_apply_to_the_current_error() {
        assert_eq!(available_actions(22, None), vec!["enable", "install_inf"]);
        assert_eq!(available_actions(28, None), vec!["install_inf"]);
        let stored = DriverEvidence {
            provider: None,
            version: None,
            date: None,
            inf_name: Some("oem42.inf".into()),
            is_signed: Some(true),
        };
        assert_eq!(
            available_actions(10, Some(&stored)),
            vec!["restart", "reinstall_store", "install_inf"]
        );
    }

    #[test]
    fn rejects_device_ids_that_could_change_command_boundaries() {
        assert!(validate_device_id("").is_err());
        assert!(validate_device_id("PCI\\VALID_DEVICE\\1").is_ok());
        assert!(validate_device_id("PCI\\DEVICE\n/scan-devices").is_err());
    }
}
