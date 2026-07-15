//! Windows 系统服务诊断 — 按领域分组

use crate::events::{ServiceEntry, ServiceIssue};
use crate::repair;
use crate::utils::wmi_runner;
use serde::{Deserialize, Serialize};
use wmi::WMIConnection;

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

pub fn all_repair_targets() -> Vec<&'static str> {
    let mut names = Vec::new();
    for group in [NETWORK, AUDIO, BLUETOOTH, USB] {
        for def in group {
            if def.repairable {
                names.push(def.name);
            }
        }
    }
    names
}

pub fn repair_group(defs: &'static [ServiceDef]) -> (Vec<String>, Vec<String>) {
    let names: Vec<&str> = defs
        .iter()
        .filter(|d| d.repairable)
        .map(|d| d.name)
        .collect();
    repair::restart_services(&names)
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Default, Serialize)]
pub struct ServicesReport {
    pub services: Vec<ServiceEntry>,
    pub issues: Vec<ServiceIssue>,
}

pub fn diagnose_group(defs: &'static [ServiceDef]) -> Result<ServicesReport, String> {
    wmi_runner::run(move |wmi| diagnose_subset(wmi, defs))
}

fn diagnose_subset(
    wmi: &WMIConnection,
    defs: &'static [ServiceDef],
) -> Result<ServicesReport, wmi::WMIError> {
    if defs.is_empty() {
        return Ok(ServicesReport::default());
    }
    let conditions: Vec<String> = defs
        .iter()
        .map(|s| format!("Name='{}'", s.name.replace('\'', "''")))
        .collect();
    let query = format!(
        "SELECT Name, State, StartMode FROM Win32_Service WHERE {}",
        conditions.join(" OR ")
    );
    let found: Vec<Win32Service> = wmi.raw_query(&query)?;
    let windows_11_workstation = defs
        .iter()
        .any(|def| def.run_policy == ServiceRunPolicy::Windows11OnDemand)
        && is_windows_11_workstation(wmi);

    let mut report = ServicesReport::default();
    for def in defs {
        let svc = found.iter().find(|s| s.name.as_deref() == Some(def.name));
        let (state, start_mode) = match svc {
            Some(s) => (s.state.clone(), s.start_mode.clone()),
            None => (None, None),
        };

        let expected_stopped = is_expected_stopped(
            def.run_policy,
            state.as_deref(),
            start_mode.as_deref(),
            windows_11_workstation,
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

    Ok(report)
}

fn is_windows_11_workstation(wmi: &WMIConnection) -> bool {
    let systems: Vec<Win32OperatingSystem> = wmi
        .raw_query("SELECT BuildNumber, ProductType FROM Win32_OperatingSystem")
        .unwrap_or_default();
    systems.first().is_some_and(|system| {
        system.product_type == Some(1)
            && system
                .build_number
                .as_deref()
                .and_then(|build| build.parse::<u32>().ok())
                .is_some_and(|build| build >= 22_000)
    })
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
    use super::{is_expected_stopped, ServiceRunPolicy};

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
}
