//! Task 4：自动化一键修复

use crate::utils::logging;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::Path;
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
};
use windows::Win32::System::Services::{
    CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx, StartServiceW,
    SC_MANAGER_CONNECT, SC_STATUS_PROCESS_INFO, SERVICE_CONTINUE_PENDING, SERVICE_PAUSED,
    SERVICE_PAUSE_PENDING, SERVICE_QUERY_STATUS, SERVICE_RUNNING, SERVICE_START,
    SERVICE_START_PENDING, SERVICE_STATUS_CURRENT_STATE, SERVICE_STATUS_PROCESS, SERVICE_STOPPED,
    SERVICE_STOP_PENDING,
};

use crate::services;

#[derive(Debug, Default)]
pub struct RepairReport {
    pub services_restarted: Vec<String>,
    pub services_healthy: Vec<String>,
    pub service_errors: Vec<String>,
    pub usb_power_configs: Vec<UsbPowerConfig>,
    pub power_scan_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsbPowerConfig {
    pub device_id: String,
    pub instance_group: String,
    pub interface_count: usize,
}

pub fn run_auto_repair() -> windows::core::Result<RepairReport> {
    let mut report = RepairReport::default();
    logging::info("── 开始自动修复 ──");
    let started = std::time::Instant::now();
    let (service_result, power_result) = std::thread::scope(|scope| {
        let service_tasks = services::repair_target_groups()
            .into_iter()
            .map(|names| {
                scope.spawn(move || {
                    let label = names.join(",");
                    let group_started = std::time::Instant::now();
                    let result = repair_services(&names);
                    logging::info(format!(
                        "performance repair_group={label} run_ms={}",
                        group_started.elapsed().as_millis()
                    ));
                    result
                })
            })
            .collect::<Vec<_>>();
        let power_task =
            scope.spawn(|| scan_usb_power_management().map_err(|error| format!("{error}")));

        let mut combined = ServiceRepairResult::default();
        for (index, task) in service_tasks.into_iter().enumerate() {
            match task.join() {
                Ok(result) => combined.merge(result),
                Err(_) => combined
                    .errors
                    .push(format!("repair group {index} terminated unexpectedly")),
            }
        }
        let power_result = power_task
            .join()
            .unwrap_or_else(|_| Err("USB power scan terminated unexpectedly".into()));
        (combined, power_result)
    });
    report.services_restarted = service_result.repaired;
    report.services_healthy = service_result.healthy;
    report.service_errors = service_result.errors;
    match power_result {
        Ok(configs) => {
            report.usb_power_configs = configs;
        }
        Err(e) => {
            report.power_scan_error = Some(e);
        }
    }
    logging::info(format!(
        "performance repair_total_ms={}",
        started.elapsed().as_millis()
    ));
    services::invalidate_diagnostic_cache();
    crate::bluetooth::invalidate_diagnostic_cache();
    logging::info("── 自动修复完成 ──");
    Ok(report)
}

/// 重启指定服务列表，返回 (成功, 失败消息)
pub fn restart_services(names: &[&str]) -> (Vec<String>, Vec<String>) {
    let result = repair_services(names);
    (result.repaired, result.errors)
}

#[derive(Debug, Default)]
struct ServiceRepairResult {
    repaired: Vec<String>,
    healthy: Vec<String>,
    errors: Vec<String>,
}

impl ServiceRepairResult {
    fn merge(&mut self, mut other: Self) {
        self.repaired.append(&mut other.repaired);
        self.healthy.append(&mut other.healthy);
        self.errors.append(&mut other.errors);
    }
}

fn repair_services(names: &[&str]) -> ServiceRepairResult {
    let mut result = ServiceRepairResult::default();
    for name in names {
        match ensure_service_running(name) {
            Ok(false) => result.healthy.push((*name).to_string()),
            Ok(true) => {
                logging::info(format!("服务 {name} 已恢复运行"));
                result.repaired.push((*name).to_string());
            }
            Err(error) => {
                logging::error(format!("服务 {name} 修复失败: {error}"));
                result.errors.push(format!("{name}: {error}"));
            }
        }
    }
    result
}

/// 判断服务错误是否由权限不足引起
pub fn errors_need_elevation(errors: &[String]) -> bool {
    errors.iter().any(|e| {
        let lower = e.to_lowercase();
        lower.contains("access is denied")
            || lower.contains("拒绝访问")
            || lower.contains("error 5")
            || lower.contains("0x80070005")
    })
}

/// 生成面向用户的修复摘要键（供前端 i18n）
pub fn build_summary_meta(
    success: bool,
    needs_admin: bool,
    report: &RepairReport,
) -> (String, Option<usize>) {
    if success && report.usb_power_configs.is_empty() {
        return ("ok_clean".into(), None);
    }
    if success && !report.usb_power_configs.is_empty() {
        return (
            "ok_power_configs".into(),
            Some(report.usb_power_configs.len()),
        );
    }
    if needs_admin {
        return ("needs_admin".into(), None);
    }
    if report.service_errors.is_empty() {
        return ("usb_scan_error".into(), None);
    }
    ("service_errors".into(), Some(report.service_errors.len()))
}

fn ensure_service_running(service_name: &str) -> windows::core::Result<bool> {
    unsafe {
        let scm = OpenSCManagerW(None, None, SC_MANAGER_CONNECT)?;
        let wide_name: Vec<u16> = service_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let query_service =
            match OpenServiceW(scm, PCWSTR(wide_name.as_ptr()), SERVICE_QUERY_STATUS) {
                Ok(service) => service,
                Err(error) => {
                    let _ = CloseServiceHandle(scm);
                    return Err(error);
                }
            };
        let status_result = (|| {
            let status = query_service_status(query_service)?;
            if status.dwCurrentState == SERVICE_RUNNING {
                return Ok(Some(false));
            }
            if matches!(
                status.dwCurrentState,
                SERVICE_START_PENDING | SERVICE_CONTINUE_PENDING
            ) {
                wait_for_service_state(query_service, SERVICE_RUNNING)?;
                return Ok(Some(true));
            }
            if matches!(
                status.dwCurrentState,
                SERVICE_STOP_PENDING | SERVICE_PAUSE_PENDING
            ) {
                wait_for_service_state(query_service, SERVICE_STOPPED)?;
            } else if status.dwCurrentState == SERVICE_PAUSED {
                return Err(windows::core::Error::new(
                    windows::core::HRESULT::from_win32(
                        windows::Win32::Foundation::ERROR_INVALID_STATE.0,
                    ),
                    "服务处于暂停状态，未执行破坏性重启",
                ));
            }
            Ok(None)
        })();
        let _ = CloseServiceHandle(query_service);
        match status_result {
            Ok(Some(done)) => {
                let _ = CloseServiceHandle(scm);
                return Ok(done);
            }
            Err(error) => {
                let _ = CloseServiceHandle(scm);
                return Err(error);
            }
            Ok(None) => {}
        }

        let service = match OpenServiceW(
            scm,
            PCWSTR(wide_name.as_ptr()),
            SERVICE_START | SERVICE_QUERY_STATUS,
        ) {
            Ok(service) => service,
            Err(error) => {
                let _ = CloseServiceHandle(scm);
                return Err(error);
            }
        };
        let result = (|| {
            if query_service_status(service)?.dwCurrentState == SERVICE_RUNNING {
                return Ok(true);
            }
            StartServiceW(service, None)?;
            wait_for_service_state(service, SERVICE_RUNNING)?;
            Ok(true)
        })();
        let _ = CloseServiceHandle(service);
        let _ = CloseServiceHandle(scm);
        result
    }
}

unsafe fn query_service_status(
    service: windows::Win32::System::Services::SC_HANDLE,
) -> windows::core::Result<SERVICE_STATUS_PROCESS> {
    let mut status = SERVICE_STATUS_PROCESS::default();
    let mut bytes_needed = 0u32;
    let buf = std::slice::from_raw_parts_mut(
        (&mut status as *mut SERVICE_STATUS_PROCESS).cast::<u8>(),
        std::mem::size_of::<SERVICE_STATUS_PROCESS>(),
    );
    QueryServiceStatusEx(
        service,
        SC_STATUS_PROCESS_INFO,
        Some(buf),
        &mut bytes_needed,
    )?;
    Ok(status)
}

unsafe fn wait_for_service_state(
    service: windows::Win32::System::Services::SC_HANDLE,
    desired: SERVICE_STATUS_CURRENT_STATE,
) -> windows::core::Result<()> {
    let absolute_deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    let mut last_checkpoint = 0;
    let mut last_progress = std::time::Instant::now();
    loop {
        let status = query_service_status(service)?;
        if status.dwCurrentState == desired {
            return Ok(());
        }
        if status.dwCheckPoint > last_checkpoint {
            last_checkpoint = status.dwCheckPoint;
            last_progress = std::time::Instant::now();
        }
        let wait_hint_ms = if status.dwWaitHint == 0 {
            10_000
        } else {
            status.dwWaitHint.clamp(1_000, 30_000)
        };
        let wait_hint = std::time::Duration::from_millis(wait_hint_ms as u64);
        if std::time::Instant::now() >= absolute_deadline || last_progress.elapsed() > wait_hint {
            return Err(windows::core::Error::new(
                windows::core::HRESULT::from_win32(windows::Win32::Foundation::ERROR_TIMEOUT.0),
                format!("服务未在预期时间内进入状态 {}", desired.0),
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(
            (status.dwWaitHint / 10).clamp(100, 1_000) as u64,
        ));
    }
}

fn scan_usb_power_management() -> windows::core::Result<Vec<UsbPowerConfig>> {
    let mut results = Vec::new();
    unsafe {
        let mut usb_key = HKEY::default();
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            windows::core::w!(r"SYSTEM\CurrentControlSet\Enum\USB"),
            None,
            KEY_READ,
            &mut usb_key,
        )
        .ok()?;
        scan_registry_tree(usb_key, Path::new("HKLM\\Enum\\USB"), &mut results)?;
        let _ = RegCloseKey(usb_key);
    }
    Ok(group_usb_power_nodes(results))
}

unsafe fn scan_registry_tree(
    key: HKEY,
    path_prefix: &Path,
    results: &mut Vec<(String, String)>,
) -> windows::core::Result<()> {
    let mut index = 0u32;
    loop {
        let mut name_buf = [0u16; 256];
        let mut name_len = name_buf.len() as u32;
        let mut class_buf = [0u16; 256];
        let mut class_len = class_buf.len() as u32;
        let result = RegEnumKeyExW(
            key,
            index,
            Some(windows::core::PWSTR(name_buf.as_mut_ptr())),
            &mut name_len,
            None,
            Some(windows::core::PWSTR(class_buf.as_mut_ptr())),
            Some(&mut class_len),
            None,
        );
        if result.is_err() {
            break;
        }
        index += 1;
        let sub_name = String::from_utf16_lossy(&name_buf[..name_len as usize]);
        let mut sub_key = HKEY::default();
        if RegOpenKeyExW(key, PCWSTR(name_buf.as_ptr()), None, KEY_READ, &mut sub_key).is_err() {
            continue;
        }
        if let Some(val) = read_dword_value(sub_key, "Device Parameters") {
            if val == 1 {
                let hardware_id = path_prefix
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("USB")
                    .to_string();
                results.push((hardware_id, sub_name.clone()));
            }
        }
        let sub_path = path_prefix.join(&sub_name);
        let _ = scan_registry_tree(sub_key, &sub_path, results);
        let _ = RegCloseKey(sub_key);
    }
    Ok(())
}

fn group_usb_power_nodes(nodes: Vec<(String, String)>) -> Vec<UsbPowerConfig> {
    let mut grouped: BTreeMap<(String, String), usize> = BTreeMap::new();
    for (hardware_id, instance_id) in nodes {
        let device_id = hardware_id
            .split("&MI_")
            .next()
            .unwrap_or(&hardware_id)
            .to_string();
        let mut parts: Vec<&str> = instance_id.split('&').collect();
        if parts.len() > 3 {
            parts.pop();
        }
        let instance_group = parts.join("&");
        *grouped.entry((device_id, instance_group)).or_default() += 1;
    }
    grouped
        .into_iter()
        .map(
            |((device_id, instance_group), interface_count)| UsbPowerConfig {
                device_id,
                instance_group,
                interface_count,
            },
        )
        .collect()
}

unsafe fn read_dword_value(key: HKEY, sub_path: &str) -> Option<u32> {
    let wide_sub: Vec<u16> = sub_path.encode_utf16().chain(std::iter::once(0)).collect();
    let mut sub_key = HKEY::default();
    if RegOpenKeyExW(key, PCWSTR(wide_sub.as_ptr()), None, KEY_READ, &mut sub_key).is_err() {
        return None;
    }
    let value_name = windows::core::w!("EnhancedPowerManagementEnabled");
    let mut data: u32 = 0;
    let mut data_size = std::mem::size_of::<u32>() as u32;
    let mut value_type = windows::Win32::System::Registry::REG_VALUE_TYPE(0);
    let result = RegQueryValueExW(
        sub_key,
        value_name,
        None,
        Some(&mut value_type),
        Some((&mut data as *mut u32).cast::<u8>()),
        Some(&mut data_size),
    );
    let _ = RegCloseKey(sub_key);
    if result.is_ok() {
        Some(data)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::group_usb_power_nodes;

    #[test]
    fn groups_composite_usb_interfaces_by_device_instance() {
        let grouped = group_usb_power_nodes(vec![
            ("VID_046D&PID_C548&MI_00".into(), "9&ABC&0&0000".into()),
            ("VID_046D&PID_C548&MI_01".into(), "9&ABC&0&0001".into()),
            ("VID_046D&PID_C548&MI_02".into(), "9&ABC&0&0002".into()),
        ]);
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].device_id, "VID_046D&PID_C548");
        assert_eq!(grouped[0].instance_group, "9&ABC&0");
        assert_eq!(grouped[0].interface_count, 3);
    }
}
