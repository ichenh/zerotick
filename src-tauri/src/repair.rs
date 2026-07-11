//! Task 4：自动化一键修复

use crate::utils::logging;
use std::path::Path;
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE,
    KEY_READ,
};
use windows::Win32::System::Services::{
    CloseServiceHandle, ControlService, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx,
    StartServiceW, SC_MANAGER_CONNECT, SC_STATUS_PROCESS_INFO, SERVICE_CONTROL_STOP,
    SERVICE_QUERY_STATUS, SERVICE_START, SERVICE_STATUS, SERVICE_STATUS_PROCESS, SERVICE_STOP,
    SERVICE_STOPPED, SERVICE_RUNNING,
};

const TARGET_SERVICES: &[&str] = &["bthserv", "Audiosrv"];

#[derive(Debug, Default)]
pub struct RepairReport {
    pub services_restarted: Vec<String>,
    pub service_errors: Vec<String>,
    pub usb_hubs_with_power_mgmt: Vec<String>,
    pub power_scan_error: Option<String>,
}

pub fn run_auto_repair() -> windows::core::Result<RepairReport> {
    let mut report = RepairReport::default();
    logging::info("── 开始自动修复 ──");
    for name in TARGET_SERVICES {
        match restart_service(name) {
            Ok(()) => {
                logging::info(format!("  ✓ 服务 {name} 已重启"));
                report.services_restarted.push(name.to_string());
            }
            Err(e) => {
                logging::error(format!("  ✗ 服务 {name} 重启失败: {e}"));
                report.service_errors.push(format!("{name}: {e}"));
            }
        }
    }
    match scan_usb_power_management() {
        Ok(hubs) => {
            report.usb_hubs_with_power_mgmt = hubs;
        }
        Err(e) => {
            report.power_scan_error = Some(format!("{e}"));
        }
    }
    logging::info("── 自动修复完成 ──");
    Ok(report)
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
    if success && report.usb_hubs_with_power_mgmt.is_empty() {
        return ("ok_clean".into(), None);
    }
    if success && !report.usb_hubs_with_power_mgmt.is_empty() {
        return (
            "ok_usb_warnings".into(),
            Some(report.usb_hubs_with_power_mgmt.len()),
        );
    }
    if needs_admin {
        return ("needs_admin".into(), None);
    }
    if report.service_errors.is_empty() {
        return ("usb_scan_error".into(), None);
    }
    (
        "service_errors".into(),
        Some(report.service_errors.len()),
    )
}

fn restart_service(service_name: &str) -> windows::core::Result<()> {
    unsafe {
        let scm = OpenSCManagerW(None, None, SC_MANAGER_CONNECT)?;
        let wide_name: Vec<u16> = service_name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let service = OpenServiceW(
            scm,
            PCWSTR(wide_name.as_ptr()),
            SERVICE_STOP | SERVICE_START | SERVICE_QUERY_STATUS,
        )?;
        let mut status = SERVICE_STATUS::default();
        let _ = ControlService(service, SERVICE_CONTROL_STOP, &mut status);
        wait_for_service_state(service, SERVICE_STOP, 10_000)?;
        StartServiceW(service, None)?;
        wait_for_service_state(service, SERVICE_START, 10_000)?;
        CloseServiceHandle(service)?;
        CloseServiceHandle(scm)?;
    }
    Ok(())
}

unsafe fn wait_for_service_state(
    service: windows::Win32::System::Services::SC_HANDLE,
    target_state: u32,
    timeout_ms: u32,
) -> windows::core::Result<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms as u64);
    let desired = if target_state == SERVICE_STOP {
        SERVICE_STOPPED
    } else {
        SERVICE_RUNNING
    };
    loop {
        let mut status = SERVICE_STATUS_PROCESS::default();
        let mut bytes_needed = 0u32;
        let mut buf = std::slice::from_raw_parts_mut(
            (&mut status as *mut SERVICE_STATUS_PROCESS).cast::<u8>(),
            std::mem::size_of::<SERVICE_STATUS_PROCESS>(),
        );
        QueryServiceStatusEx(service, SC_STATUS_PROCESS_INFO, Some(&mut buf), &mut bytes_needed)?;
        if status.dwCurrentState == desired {
            return Ok(());
        }
        if std::time::Instant::now() >= deadline {
            return Err(windows::core::Error::new(
                windows::core::HRESULT::from_win32(windows::Win32::Foundation::ERROR_TIMEOUT.0),
                "等待服务状态变更超时",
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }
}

fn scan_usb_power_management() -> windows::core::Result<Vec<String>> {
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
    Ok(results)
}

unsafe fn scan_registry_tree(
    key: HKEY,
    path_prefix: &Path,
    results: &mut Vec<String>,
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
                results.push(format!(
                    "{}\\{} (EnhancedPowerManagementEnabled=1)",
                    path_prefix.display(),
                    sub_name
                ));
            }
        }
        let sub_path = path_prefix.join(&sub_name);
        let _ = scan_registry_tree(sub_key, &sub_path, results);
        let _ = RegCloseKey(sub_key);
    }
    Ok(())
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
