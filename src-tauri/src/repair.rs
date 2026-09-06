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
    CloseServiceHandle, ControlService, OpenSCManagerW, OpenServiceW, QueryServiceConfigW,
    QueryServiceStatusEx, StartServiceW, QUERY_SERVICE_CONFIGW, SC_MANAGER_CONNECT,
    SC_STATUS_PROCESS_INFO, SERVICE_CONTINUE_PENDING, SERVICE_CONTROL_CONTINUE,
    SERVICE_CONTROL_STOP, SERVICE_DISABLED, SERVICE_PAUSED, SERVICE_PAUSE_CONTINUE,
    SERVICE_PAUSE_PENDING, SERVICE_QUERY_CONFIG, SERVICE_QUERY_STATUS, SERVICE_RUNNING,
    SERVICE_START, SERVICE_START_PENDING, SERVICE_STATUS, SERVICE_STATUS_CURRENT_STATE,
    SERVICE_STATUS_PROCESS, SERVICE_STOP, SERVICE_STOPPED, SERVICE_STOP_PENDING,
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

/// 恢复停止或暂停的服务，返回 (由本次操作恢复的服务, 失败消息)。
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

#[derive(Debug, PartialEq, Eq)]
enum ServiceRecoveryPlan {
    Healthy,
    WaitRunning,
    WaitStopped,
    WaitPaused,
    Start,
    Continue,
    Unsupported,
}

fn service_recovery_plan(state: SERVICE_STATUS_CURRENT_STATE) -> ServiceRecoveryPlan {
    match state {
        SERVICE_RUNNING => ServiceRecoveryPlan::Healthy,
        SERVICE_START_PENDING | SERVICE_CONTINUE_PENDING => ServiceRecoveryPlan::WaitRunning,
        SERVICE_STOP_PENDING => ServiceRecoveryPlan::WaitStopped,
        SERVICE_PAUSE_PENDING => ServiceRecoveryPlan::WaitPaused,
        SERVICE_STOPPED => ServiceRecoveryPlan::Start,
        SERVICE_PAUSED => ServiceRecoveryPlan::Continue,
        _ => ServiceRecoveryPlan::Unsupported,
    }
}

fn service_state_changed_error() -> windows::core::Error {
    windows::core::Error::new(
        windows::core::HRESULT::from_win32(windows::Win32::Foundation::ERROR_INVALID_STATE.0),
        "服务状态已变化或不支持当前恢复操作，请重新检测",
    )
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
        let result = (|| {
            let mut plan =
                service_recovery_plan(query_service_status(query_service)?.dwCurrentState);
            match plan {
                ServiceRecoveryPlan::Healthy => return Ok(false),
                ServiceRecoveryPlan::WaitRunning => {
                    wait_for_service_state(query_service, SERVICE_RUNNING)?;
                    // Another process initiated this transition; do not claim a repair.
                    return Ok(false);
                }
                ServiceRecoveryPlan::WaitStopped => {
                    wait_for_service_state(query_service, SERVICE_STOPPED)?;
                    plan = ServiceRecoveryPlan::Start;
                }
                ServiceRecoveryPlan::WaitPaused => {
                    wait_for_service_state(query_service, SERVICE_PAUSED)?;
                    plan = ServiceRecoveryPlan::Continue;
                }
                ServiceRecoveryPlan::Start | ServiceRecoveryPlan::Continue => {}
                ServiceRecoveryPlan::Unsupported => return Err(service_state_changed_error()),
            }
            let access = if plan == ServiceRecoveryPlan::Continue {
                SERVICE_PAUSE_CONTINUE
            } else {
                SERVICE_START
            };
            let service = OpenServiceW(
                scm,
                PCWSTR(wide_name.as_ptr()),
                access | SERVICE_QUERY_STATUS,
            )?;
            let operation = (|| {
                let current = service_recovery_plan(query_service_status(service)?.dwCurrentState);
                if current == ServiceRecoveryPlan::Healthy {
                    return Ok(false);
                }
                if current == ServiceRecoveryPlan::WaitRunning {
                    wait_for_service_state(service, SERVICE_RUNNING)?;
                    return Ok(false);
                }
                if current != plan {
                    return Err(service_state_changed_error());
                }
                if plan == ServiceRecoveryPlan::Continue {
                    let mut status = SERVICE_STATUS::default();
                    ControlService(service, SERVICE_CONTROL_CONTINUE, &mut status)?;
                } else if let Err(error) = StartServiceW(service, None) {
                    if error.code()
                        == windows::core::HRESULT::from_win32(
                            windows::Win32::Foundation::ERROR_SERVICE_ALREADY_RUNNING.0,
                        )
                    {
                        wait_for_service_state(service, SERVICE_RUNNING)?;
                        return Ok(false);
                    }
                    return Err(error);
                }
                wait_for_service_state(service, SERVICE_RUNNING)?;
                Ok(true)
            })();
            let _ = CloseServiceHandle(service);
            operation
        })();
        let _ = CloseServiceHandle(query_service);
        let _ = CloseServiceHandle(scm);
        result
    }
}

/// Explicit audio repair only: restart Windows Audio without stopping dependent
/// services or changing configuration. SCM rejects STOP when dependents are running.
pub fn restart_audio_service() -> Result<(), String> {
    let result: windows::core::Result<()> = unsafe {
        let scm = OpenSCManagerW(None, None, SC_MANAGER_CONNECT)
            .map_err(|error| format!("Audiosrv: 打开服务管理器失败: {error}"))?;
        let service = match OpenServiceW(
            scm,
            windows::core::w!("Audiosrv"),
            SERVICE_QUERY_CONFIG | SERVICE_QUERY_STATUS | SERVICE_STOP | SERVICE_START,
        ) {
            Ok(service) => service,
            Err(error) => {
                let _ = CloseServiceHandle(scm);
                return Err(format!("Audiosrv: 打开服务失败: {error}"));
            }
        };
        let operation = (|| {
            // A disabled service may still be running. Check before STOP so a
            // restart cannot knowingly leave an otherwise running engine stopped.
            let mut bytes_needed = 0u32;
            if let Err(error) = QueryServiceConfigW(service, None, 0, &mut bytes_needed) {
                if error.code()
                    != windows::core::HRESULT::from_win32(
                        windows::Win32::Foundation::ERROR_INSUFFICIENT_BUFFER.0,
                    )
                {
                    return Err(audio_service_error("读取启动配置", error));
                }
            }
            if bytes_needed < std::mem::size_of::<QUERY_SERVICE_CONFIGW>() as u32 {
                return Err(service_state_changed_error());
            }
            let mut config_buffer =
                vec![0usize; (bytes_needed as usize).div_ceil(std::mem::size_of::<usize>())];
            let config = config_buffer.as_mut_ptr().cast::<QUERY_SERVICE_CONFIGW>();
            QueryServiceConfigW(service, Some(config), bytes_needed, &mut bytes_needed)
                .map_err(|error| audio_service_error("读取启动配置", error))?;
            if (*config).dwStartType == SERVICE_DISABLED {
                return Err(windows::core::Error::new(
                    windows::core::HRESULT::from_win32(
                        windows::Win32::Foundation::ERROR_SERVICE_DISABLED.0,
                    ),
                    "Windows Audio 已被禁用，未停止服务或更改其启动设置",
                ));
            }
            let state = query_service_status(service)
                .map_err(|error| audio_service_error("读取当前状态", error))?
                .dwCurrentState;
            match service_recovery_plan(state) {
                ServiceRecoveryPlan::WaitRunning => {
                    wait_for_service_state(service, SERVICE_RUNNING)
                        .map_err(|error| audio_service_error("等待现有启动或恢复", error))?;
                }
                ServiceRecoveryPlan::WaitPaused => {
                    wait_for_service_state(service, SERVICE_PAUSED)
                        .map_err(|error| audio_service_error("等待现有暂停", error))?;
                }
                ServiceRecoveryPlan::WaitStopped => {
                    wait_for_service_state(service, SERVICE_STOPPED)
                        .map_err(|error| audio_service_error("等待现有停止", error))?;
                }
                ServiceRecoveryPlan::Unsupported => return Err(service_state_changed_error()),
                _ => {}
            }
            let state = query_service_status(service)
                .map_err(|error| audio_service_error("复查停止前状态", error))?
                .dwCurrentState;
            if matches!(state, SERVICE_RUNNING | SERVICE_PAUSED) {
                let mut status = SERVICE_STATUS::default();
                // Do not enumerate/stop dependents or broaden this operation on failure.
                ControlService(service, SERVICE_CONTROL_STOP, &mut status)
                    .map_err(|error| audio_service_error("停止", error))?;
                wait_for_service_state(service, SERVICE_STOPPED)
                    .map_err(|error| audio_service_error("等待停止", error))?;
            } else if state != SERVICE_STOPPED {
                return Err(service_state_changed_error());
            }
            // A competing start is not our successful restart; preserve its error.
            StartServiceW(service, None).map_err(|error| audio_service_error("启动", error))?;
            wait_for_service_state(service, SERVICE_RUNNING)
                .map_err(|error| audio_service_error("等待最终运行", error))
        })();
        let _ = CloseServiceHandle(service);
        let _ = CloseServiceHandle(scm);
        operation
    };
    result.map_err(|error| format!("Audiosrv: {error}"))
}

fn audio_service_error(stage: &str, error: windows::core::Error) -> windows::core::Error {
    windows::core::Error::new(error.code(), format!("{stage} Windows Audio 失败: {error}"))
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
    use super::{group_usb_power_nodes, service_recovery_plan, ServiceRecoveryPlan};
    use windows::Win32::System::Services::{
        SERVICE_CONTINUE_PENDING, SERVICE_PAUSED, SERVICE_PAUSE_PENDING, SERVICE_RUNNING,
        SERVICE_START_PENDING, SERVICE_STOPPED, SERVICE_STOP_PENDING,
    };

    #[test]
    fn service_recovery_distinguishes_external_transitions_from_required_actions() {
        assert_eq!(
            service_recovery_plan(SERVICE_RUNNING),
            ServiceRecoveryPlan::Healthy
        );
        for state in [SERVICE_START_PENDING, SERVICE_CONTINUE_PENDING] {
            assert_eq!(
                service_recovery_plan(state),
                ServiceRecoveryPlan::WaitRunning
            );
        }
        assert_eq!(
            service_recovery_plan(SERVICE_STOPPED),
            ServiceRecoveryPlan::Start
        );
        assert_eq!(
            service_recovery_plan(SERVICE_PAUSED),
            ServiceRecoveryPlan::Continue
        );
        assert_eq!(
            service_recovery_plan(SERVICE_STOP_PENDING),
            ServiceRecoveryPlan::WaitStopped
        );
        assert_eq!(
            service_recovery_plan(SERVICE_PAUSE_PENDING),
            ServiceRecoveryPlan::WaitPaused
        );
    }

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
