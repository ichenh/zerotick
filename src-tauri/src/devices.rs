//! Windows 设备与驱动诊断：聚合常用硬件类别，并把设备管理器错误转成稳定的语义标识。

use crate::utils::{elevated, process::CommandExt, wmi_runner};
use serde::{Deserialize, Serialize};
use std::process::Command;
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
            problems.push(DeviceProblem {
                name: entity.name.unwrap_or_else(|| "Unknown device".into()),
                class_id: class_id.into(),
                device_id: entity.device_id.unwrap_or_default(),
                error_code: code,
                reason_id: error_reason(code).into(),
                status: entity.status,
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
}
