//! Windows 系统服务诊断 — 按领域分组

use crate::events::{ServiceEntry, ServiceIssue};
use crate::repair;
use crate::utils::wmi_runner;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use windows::core::PCWSTR;
use windows::Win32::System::Services::{
    CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceConfigW, QueryServiceStatusEx,
    QUERY_SERVICE_CONFIGW, SC_HANDLE, SC_MANAGER_CONNECT, SC_STATUS_PROCESS_INFO,
    SERVICE_AUTO_START, SERVICE_BOOT_START, SERVICE_CONTINUE_PENDING, SERVICE_DEMAND_START,
    SERVICE_DISABLED, SERVICE_PAUSED, SERVICE_PAUSE_PENDING, SERVICE_QUERY_CONFIG,
    SERVICE_QUERY_STATUS, SERVICE_RUNNING, SERVICE_START_PENDING, SERVICE_STATUS_CURRENT_STATE,
    SERVICE_STATUS_PROCESS, SERVICE_STOPPED, SERVICE_STOP_PENDING, SERVICE_SYSTEM_START,
};
use wmi::WMIConnection;

const SNAPSHOT_TTL: Duration = Duration::from_secs(2);
static SNAPSHOT_CACHE: OnceLock<Mutex<ServiceSnapshotCache>> = OnceLock::new();
static WINDOWS_11_WORKSTATION: OnceLock<bool> = OnceLock::new();

/// 监控项：name = Win32 服务名，label_id = 前端 i18n 键
pub struct ServiceDef {
    pub name: &'static str,
    pub label_id: &'static str,
    pub repairable: bool,
    run_policy: ServiceRunPolicy,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ServiceRunPolicy {
    Required,
    /// Since Windows 11, NLA duties moved to Network List Manager and NlaSvc may
    /// legitimately remain stopped when configured for manual/triggered start.
    Windows11OnDemand,
}

pub const NETWORK: &[ServiceDef] = &[
    ServiceDef {
        name: "Dnscache",
        label_id: "dns",
        repairable: true,
        run_policy: ServiceRunPolicy::Required,
    },
    ServiceDef {
        name: "Dhcp",
        label_id: "dhcp",
        repairable: true,
        run_policy: ServiceRunPolicy::Required,
    },
    ServiceDef {
        name: "NlaSvc",
        label_id: "network",
        repairable: false,
        run_policy: ServiceRunPolicy::Windows11OnDemand,
    },
];

pub const AUDIO: &[ServiceDef] = &[
    ServiceDef {
        name: "AudioEndpointBuilder",
        label_id: "audio_endpoint",
        repairable: true,
        run_policy: ServiceRunPolicy::Required,
    },
    ServiceDef {
        name: "Audiosrv",
        label_id: "audio",
        repairable: true,
        run_policy: ServiceRunPolicy::Required,
    },
];

pub const BLUETOOTH: &[ServiceDef] = &[ServiceDef {
    name: "bthserv",
    label_id: "bluetooth",
    repairable: true,
    run_policy: ServiceRunPolicy::Required,
}];

pub const USB: &[ServiceDef] = &[ServiceDef {
    name: "PlugPlay",
    label_id: "plugplay",
    repairable: true,
    run_policy: ServiceRunPolicy::Required,
}];

/// Independent service groups may be repaired in parallel. Ordering inside a
/// group is preserved for dependencies such as AudioEndpointBuilder -> Audiosrv.
pub fn repair_target_groups() -> Vec<Vec<&'static str>> {
    let mut groups = Vec::new();
    for group in [NETWORK, AUDIO, BLUETOOTH, USB] {
        let mut names = Vec::new();
        for def in group {
            if def.repairable {
                names.push(def.name);
            }
        }
        if !names.is_empty() {
            groups.push(names);
        }
    }
    groups
}

pub fn repair_group(defs: &'static [ServiceDef]) -> (Vec<String>, Vec<String>) {
    let names: Vec<&str> = defs
        .iter()
        .filter(|d| d.repairable)
        .map(|d| d.name)
        .collect();
    let result = repair::restart_services(&names);
    invalidate_diagnostic_cache();
    result
}

#[derive(Debug, Clone, Deserialize)]
struct Win32Service {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "State")]
    state: Option<String>,
    #[serde(rename = "StartMode")]
    start_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Win32OperatingSystem {
    #[serde(rename = "BuildNumber")]
    build_number: Option<String>,
    #[serde(rename = "ProductType")]
    product_type: Option<u32>,
}

#[derive(Debug, Clone)]
struct ServiceSnapshot {
    services: Vec<Win32Service>,
    windows_11_workstation: bool,
}

#[derive(Default)]
struct ServiceSnapshotCache {
    finished_at: Option<Instant>,
    snapshot: Option<ServiceSnapshot>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ServicesReport {
    pub services: Vec<ServiceEntry>,
    pub issues: Vec<ServiceIssue>,
}

pub fn diagnose_group(defs: &'static [ServiceDef]) -> Result<ServicesReport, String> {
    let snapshot = service_snapshot()?;
    Ok(build_report(defs, &snapshot))
}

/// Query required services directly through the Service Control Manager. This
/// avoids waiting behind unrelated WMI jobs while preserving the same state and
/// start-mode evidence. Groups with on-demand OS-specific policy keep using the
/// WMI snapshot so their Windows-version interpretation remains unchanged.
pub fn diagnose_group_native(defs: &'static [ServiceDef]) -> Result<ServicesReport, String> {
    if defs
        .iter()
        .any(|definition| definition.run_policy != ServiceRunPolicy::Required)
    {
        return Err("native service query does not support OS-specific run policies".into());
    }
    let services = defs
        .iter()
        .map(|definition| query_service_native(definition.name))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(build_report(
        defs,
        &ServiceSnapshot {
            services,
            windows_11_workstation: false,
        },
    ))
}

pub fn invalidate_diagnostic_cache() {
    if let Some(cache) = SNAPSHOT_CACHE.get() {
        if let Ok(mut cache) = cache.lock() {
            cache.finished_at = None;
            cache.snapshot = None;
        }
    }
}

fn service_snapshot() -> Result<ServiceSnapshot, String> {
    let cache = SNAPSHOT_CACHE.get_or_init(|| Mutex::new(ServiceSnapshotCache::default()));
    let mut cache = cache
        .lock()
        .map_err(|_| "Service diagnostic cache lock failed".to_string())?;
    if let (Some(finished_at), Some(snapshot)) = (cache.finished_at, cache.snapshot.as_ref()) {
        if finished_at.elapsed() <= SNAPSHOT_TTL {
            return Ok(snapshot.clone());
        }
    }

    let snapshot = wmi_runner::run(query_service_snapshot)?;
    cache.finished_at = Some(Instant::now());
    cache.snapshot = Some(snapshot.clone());
    Ok(snapshot)
}

fn query_service_snapshot(wmi: &WMIConnection) -> Result<ServiceSnapshot, wmi::WMIError> {
    let conditions = [NETWORK, AUDIO, BLUETOOTH, USB]
        .into_iter()
        .flat_map(|group| group.iter())
        .map(|service| format!("Name='{}'", service.name.replace('\'', "''")))
        .collect::<Vec<_>>();
    let query = format!(
        "SELECT Name, State, StartMode FROM Win32_Service WHERE {}",
        conditions.join(" OR ")
    );
    let services = wmi.raw_query(&query)?;
    Ok(ServiceSnapshot {
        services,
        windows_11_workstation: is_windows_11_workstation(wmi),
    })
}

fn query_service_native(name: &str) -> Result<Win32Service, String> {
    unsafe {
        let manager = OpenSCManagerW(None, None, SC_MANAGER_CONNECT)
            .map_err(|error| format!("open service manager failed: {error}"))?;
        let wide_name = name
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let service = match OpenServiceW(
            manager,
            PCWSTR(wide_name.as_ptr()),
            SERVICE_QUERY_STATUS | SERVICE_QUERY_CONFIG,
        ) {
            Ok(service) => service,
            Err(error) => {
                let _ = CloseServiceHandle(manager);
                return Err(format!("open service {name} failed: {error}"));
            }
        };
        let result = (|| {
            let status = query_service_status_native(service)?;
            let start_type = query_service_start_type_native(service)?;
            Ok(Win32Service {
                name: Some(name.to_string()),
                state: Some(service_state_name(status.dwCurrentState).to_string()),
                start_mode: Some(service_start_mode_name(start_type).to_string()),
            })
        })();
        let _ = CloseServiceHandle(service);
        let _ = CloseServiceHandle(manager);
        result
    }
}

unsafe fn query_service_status_native(
    service: SC_HANDLE,
) -> Result<SERVICE_STATUS_PROCESS, String> {
    let mut status = SERVICE_STATUS_PROCESS::default();
    let mut bytes_needed = 0_u32;
    let buffer = unsafe {
        std::slice::from_raw_parts_mut(
            (&mut status as *mut SERVICE_STATUS_PROCESS).cast::<u8>(),
            std::mem::size_of::<SERVICE_STATUS_PROCESS>(),
        )
    };
    unsafe {
        QueryServiceStatusEx(
            service,
            SC_STATUS_PROCESS_INFO,
            Some(buffer),
            &mut bytes_needed,
        )
    }
    .map_err(|error| format!("query service status failed: {error}"))?;
    Ok(status)
}

unsafe fn query_service_start_type_native(service: SC_HANDLE) -> Result<u32, String> {
    let mut bytes_needed = 0_u32;
    let first = unsafe { QueryServiceConfigW(service, None, 0, &mut bytes_needed) };
    if bytes_needed < std::mem::size_of::<QUERY_SERVICE_CONFIGW>() as u32 {
        return Err(first
            .err()
            .map(|error| format!("query service config size failed: {error}"))
            .unwrap_or_else(|| "query service config returned an invalid size".into()));
    }
    let word_size = std::mem::size_of::<usize>();
    let words = (bytes_needed as usize).div_ceil(word_size);
    let mut buffer = vec![0_usize; words];
    let config = buffer.as_mut_ptr().cast::<QUERY_SERVICE_CONFIGW>();
    unsafe { QueryServiceConfigW(service, Some(config), bytes_needed, &mut bytes_needed) }
        .map_err(|error| format!("query service config failed: {error}"))?;
    Ok(unsafe { (*config).dwStartType.0 })
}

fn service_state_name(state: SERVICE_STATUS_CURRENT_STATE) -> &'static str {
    match state {
        SERVICE_RUNNING => "Running",
        SERVICE_STOPPED => "Stopped",
        SERVICE_START_PENDING => "Start Pending",
        SERVICE_STOP_PENDING => "Stop Pending",
        SERVICE_PAUSED => "Paused",
        SERVICE_PAUSE_PENDING => "Pause Pending",
        SERVICE_CONTINUE_PENDING => "Continue Pending",
        _ => "Unknown",
    }
}

fn service_start_mode_name(start_type: u32) -> &'static str {
    match start_type {
        value if value == SERVICE_AUTO_START.0 => "Auto",
        value if value == SERVICE_DEMAND_START.0 => "Manual",
        value if value == SERVICE_DISABLED.0 => "Disabled",
        value if value == SERVICE_BOOT_START.0 => "Boot",
        value if value == SERVICE_SYSTEM_START.0 => "System",
        _ => "Unknown",
    }
}

fn build_report(defs: &'static [ServiceDef], snapshot: &ServiceSnapshot) -> ServicesReport {
    if defs.is_empty() {
        return ServicesReport::default();
    }

    let mut report = ServicesReport::default();
    for def in defs {
        let svc = snapshot
            .services
            .iter()
            .find(|service| service.name.as_deref() == Some(def.name));
        let (state, start_mode) = match svc {
            Some(s) => (s.state.clone(), s.start_mode.clone()),
            None => (None, None),
        };

        let expected_stopped = is_expected_stopped(
            def.run_policy,
            state.as_deref(),
            start_mode.as_deref(),
            snapshot.windows_11_workstation,
        );

        report.services.push(ServiceEntry {
            name: def.name.to_string(),
            label_id: def.label_id.to_string(),
            state: state.clone(),
            start_mode: start_mode.clone(),
            expected_stopped,
        });

        if svc.is_none() {
            report.issues.push(ServiceIssue {
                id: "missing".into(),
                service_name: def.name.to_string(),
                label_id: def.label_id.to_string(),
                state: None,
            });
            continue;
        }

        if start_mode.as_deref() == Some("Disabled") {
            report.issues.push(ServiceIssue {
                id: "disabled".into(),
                service_name: def.name.to_string(),
                label_id: def.label_id.to_string(),
                state: state.clone(),
            });
            continue;
        }

        match state.as_deref() {
            Some("Running") => {}
            Some(_) if expected_stopped => {}
            Some(s) => report.issues.push(ServiceIssue {
                id: "not_running".into(),
                service_name: def.name.to_string(),
                label_id: def.label_id.to_string(),
                state: Some(s.to_string()),
            }),
            None => report.issues.push(ServiceIssue {
                id: "status_unknown".into(),
                service_name: def.name.to_string(),
                label_id: def.label_id.to_string(),
                state: None,
            }),
        }
    }

    report
}

fn is_windows_11_workstation(wmi: &WMIConnection) -> bool {
    if let Some(cached) = WINDOWS_11_WORKSTATION.get() {
        return *cached;
    }
    let Ok(systems) = wmi.raw_query::<Win32OperatingSystem>(
        "SELECT BuildNumber, ProductType FROM Win32_OperatingSystem",
    ) else {
        return false;
    };
    let value = systems.first().is_some_and(|system| {
        system.product_type == Some(1)
            && system
                .build_number
                .as_deref()
                .and_then(|build| build.parse::<u32>().ok())
                .is_some_and(|build| build >= 22_000)
    });
    let _ = WINDOWS_11_WORKSTATION.set(value);
    value
}

fn is_expected_stopped(
    policy: ServiceRunPolicy,
    state: Option<&str>,
    start_mode: Option<&str>,
    windows_11_workstation: bool,
) -> bool {
    policy == ServiceRunPolicy::Windows11OnDemand
        && windows_11_workstation
        && state.is_some_and(|value| value.eq_ignore_ascii_case("Stopped"))
        && start_mode.is_some_and(|value| value.eq_ignore_ascii_case("Manual"))
}

#[cfg(test)]
mod tests {
    use super::{
        is_expected_stopped, service_start_mode_name, service_state_name, ServiceRunPolicy,
    };
    use windows::Win32::System::Services::{
        SERVICE_AUTO_START, SERVICE_DEMAND_START, SERVICE_DISABLED, SERVICE_RUNNING,
        SERVICE_STOPPED,
    };

    #[test]
    fn windows_11_manual_nla_is_expected_to_be_idle() {
        assert!(is_expected_stopped(
            ServiceRunPolicy::Windows11OnDemand,
            Some("Stopped"),
            Some("Manual"),
            true,
        ));
    }

    #[test]
    fn disabled_or_pre_windows_11_nla_is_not_accepted_as_idle() {
        assert!(!is_expected_stopped(
            ServiceRunPolicy::Windows11OnDemand,
            Some("Stopped"),
            Some("Disabled"),
            true,
        ));
        assert!(!is_expected_stopped(
            ServiceRunPolicy::Windows11OnDemand,
            Some("Stopped"),
            Some("Manual"),
            false,
        ));
    }

    #[test]
    fn native_service_values_match_existing_diagnostic_terms() {
        assert_eq!(service_state_name(SERVICE_RUNNING), "Running");
        assert_eq!(service_state_name(SERVICE_STOPPED), "Stopped");
        assert_eq!(service_start_mode_name(SERVICE_AUTO_START.0), "Auto");
        assert_eq!(service_start_mode_name(SERVICE_DEMAND_START.0), "Manual");
        assert_eq!(service_start_mode_name(SERVICE_DISABLED.0), "Disabled");
    }
}
