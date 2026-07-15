//! Windows 本地端口占用扫描与释放
use crate::utils::process::CommandExt;

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::process::Command;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

#[derive(Debug, Clone, Serialize)]
pub struct ExcludedRange {
    pub start: u16,
    pub end: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortEntry {
    pub port: u16,
    pub local_address: String,
    pub remote_address: String,
    pub protocol: String,
    pub state: String,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    /// releasable | in_use | time_wait | system_reserved
    pub category: String,
    pub category_label: String,
    pub message_id: String,
    pub can_release: bool,
    pub message: String,
    /// 排序权重：可解除残留 < 可解除连接 < 普通占用 < 系统保留
    pub sort_priority: u8,
    /// 残留连接解除键（local|remote|port|proto）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PortScanReport {
    pub excluded_ranges: Vec<ExcludedRange>,
    pub entries: Vec<PortEntry>,
    pub releasable_count: usize,
    pub residual_releasable_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReleaseReport {
    pub released_pids: Vec<u32>,
    pub errors: Vec<String>,
}

pub fn scan() -> Result<PortScanReport, String> {
    let (process_cache, excluded, rows) = std::thread::scope(|scope| {
        let process_task = scope.spawn(build_process_cache);
        let excluded_task = scope.spawn(load_excluded_ranges);
        let rows_task = scope.spawn(parse_netstat);
        let process_cache = process_task
            .join()
            .map_err(|_| "进程扫描异常终止".to_string())?;
        let excluded = excluded_task
            .join()
            .map_err(|_| "Windows 保留端口扫描异常终止".to_string())??;
        let rows = rows_task
            .join()
            .map_err(|_| "端口扫描异常终止".to_string())??;
        Ok::<_, String>((process_cache, excluded, rows))
    })?;
    let self_pid = std::process::id();

    let mut entries: Vec<PortEntry> = rows
        .into_iter()
        .map(|row| build_entry(&row, &excluded, self_pid, &process_cache))
        .collect();

    entries.sort_by(|a, b| {
        a.port
            .cmp(&b.port)
            .then(a.state.cmp(&b.state))
            .then(a.pid.unwrap_or(0).cmp(&b.pid.unwrap_or(0)))
    });

    let releasable_count = entries.iter().filter(|e| e.can_release).count();
    let residual_releasable_count = entries
        .iter()
        .filter(|e| e.can_release && e.connection_key.is_some())
        .count();

    Ok(PortScanReport {
        excluded_ranges: excluded
            .iter()
            .map(|(s, e)| ExcludedRange { start: *s, end: *e })
            .collect(),
        entries,
        releasable_count,
        residual_releasable_count,
    })
}

pub fn release_pid(pid: u32) -> Result<(), String> {
    let self_pid = std::process::id();
    if pid == 0 {
        return Err("无效 PID 0".into());
    }
    if pid == self_pid {
        return Err("不能结束当前 ZeroTick 进程".into());
    }

    let process_cache = build_process_cache();
    let name = process_name(pid, &process_cache).unwrap_or_else(|| "unknown".to_string());
    if is_protected_process(&name) {
        return Err(format!("{name} (PID {pid}) 为系统/关键进程，不可解除"));
    }
    if !is_releasable_process(&name, pid, self_pid) {
        return Err(format!("{name} (PID {pid}) 正在使用中，不可强制解除"));
    }

    terminate_pid(pid)?;
    Ok(())
}

pub fn release_all_releasable() -> Result<ReleaseReport, String> {
    let report = scan()?;
    let mut released = Vec::new();
    let mut errors = Vec::new();
    let mut seen = HashSet::new();

    for entry in report.entries.iter().filter(|e| e.can_release) {
        if let Some(key) = &entry.connection_key {
            if !seen.insert(key.clone()) {
                continue;
            }
            match release_connection_key(key) {
                Ok(()) => released.push(entry.pid.unwrap_or(0)),
                Err(e) => errors.push(e),
            }
            continue;
        }
        let Some(pid) = entry.pid else { continue };
        if !seen.insert(format!("pid:{pid}")) {
            continue;
        }
        match release_pid(pid) {
            Ok(()) => released.push(pid),
            Err(e) => errors.push(e),
        }
    }

    Ok(ReleaseReport {
        released_pids: released.into_iter().filter(|p| *p > 0).collect(),
        errors,
    })
}

pub fn release_connection(connection_key: &str) -> Result<(), String> {
    release_connection_key(connection_key)
}

fn release_connection_key(key: &str) -> Result<(), String> {
    let parts: Vec<&str> = key.splitn(4, '|').collect();
    if parts.len() < 4 {
        return Err("无效连接键".into());
    }
    let local = parts[0].replace('\'', "''");
    let remote = parts[1].replace('\'', "''");
    let port: u16 = parts[2].parse().map_err(|_| "无效端口")?;
    let proto = parts[3];
    let state_filter = if proto.eq_ignore_ascii_case("UDP") {
        ""
    } else {
        "-State TimeWait,CloseWait,FinWait2,LastAck"
    };
    let script = format!(
        "Get-NetTCPConnection -LocalAddress '{local}' -LocalPort {port} -RemoteAddress '{remote}' {state_filter} -ErrorAction SilentlyContinue | Remove-NetTCPConnection -Confirm:$false"
    );
    crate::utils::powershell::run_void(&script)
}

#[derive(Debug)]
struct NetRow {
    protocol: String,
    local: String,
    remote: String,
    state: String,
    pid: u32,
}

fn build_entry(
    row: &NetRow,
    excluded: &[(u16, u16)],
    self_pid: u32,
    process_cache: &HashMap<u32, String>,
) -> PortEntry {
    let port = parse_port(&row.local).unwrap_or(0);
    let pid_opt = if row.pid == 0 { None } else { Some(row.pid) };
    let process_name = pid_opt.and_then(|p| process_name(p, process_cache));

    let (category, category_label, message_id, can_release, message, sort_priority, connection_key) =
        if row.state.eq_ignore_ascii_case("TIME_WAIT")
            || row.state.eq_ignore_ascii_case("CLOSE_WAIT")
            || row.state.eq_ignore_ascii_case("FIN_WAIT_2")
        {
            let key = format!("{}|{}|{}|{}", row.local, row.remote, port, row.protocol);
            (
                "time_wait",
                "time_wait",
                if row.state.eq_ignore_ascii_case("TIME_WAIT") {
                    "time_wait_releasable"
                } else {
                    "residual_releasable"
                },
                true,
                row.state.as_str(),
                1,
                Some(key),
            )
        } else if port_in_excluded(port, excluded) && row.state.eq_ignore_ascii_case("LISTENING") {
            (
                "system_reserved",
                "system_reserved",
                "system_reserved",
                false,
                "system_reserved",
                4,
                None,
            )
        } else if pid_opt == Some(self_pid) {
            ("in_use", "in_use", "self_app", false, "self_app", 3, None)
        } else if let Some(ref name) = process_name {
            if is_protected_process(name) {
                ("in_use", "in_use", "protected", false, "protected", 3, None)
            } else if is_releasable_process(name, row.pid, self_pid) {
                (
                    "releasable",
                    "releasable",
                    "releasable",
                    true,
                    "releasable",
                    0,
                    None,
                )
            } else {
                ("in_use", "in_use", "in_use", false, "in_use", 2, None)
            }
        } else if row.state.eq_ignore_ascii_case("LISTENING") {
            ("in_use", "in_use", "unknown", false, "unknown", 2, None)
        } else {
            (
                "in_use",
                "in_use",
                "other",
                false,
                row.state.as_str(),
                2,
                None,
            )
        };

    PortEntry {
        port,
        local_address: row.local.clone(),
        remote_address: row.remote.clone(),
        protocol: row.protocol.clone(),
        state: row.state.clone(),
        pid: pid_opt,
        process_name,
        category: category.into(),
        category_label: category_label.into(),
        message_id: message_id.into(),
        can_release,
        message: message.to_string(),
        sort_priority,
        connection_key,
    }
}

fn is_releasable_process(name: &str, pid: u32, self_pid: u32) -> bool {
    if pid == self_pid {
        return false;
    }
    let lower = name.to_lowercase();
    if matches!(
        lower.as_str(),
        "node.exe" | "esbuild.exe" | "vite.exe" | "zerotick.exe"
    ) {
        return true;
    }
    false
}

fn is_protected_process(name: &str) -> bool {
    let lower = name.to_lowercase();
    matches!(
        lower.as_str(),
        "system"
            | "registry"
            | "smss.exe"
            | "csrss.exe"
            | "wininit.exe"
            | "services.exe"
            | "lsass.exe"
            | "svchost.exe"
            | "fontdrvhost.exe"
            | "dwm.exe"
            | "winlogon.exe"
            | "msmpeng.exe"
            | "system idle process"
    )
}

fn port_in_excluded(port: u16, ranges: &[(u16, u16)]) -> bool {
    ranges.iter().any(|(s, e)| port >= *s && port <= *e)
}

fn parse_port(addr: &str) -> Option<u16> {
    let addr = addr.trim();
    let (_, port_str) = if let Some(rest) = addr.strip_prefix('[') {
        rest.split_once(']')?
    } else {
        addr.rsplit_once(':')?
    };
    port_str.parse().ok()
}

fn parse_netstat() -> Result<Vec<NetRow>, String> {
    let output = Command::new("netstat")
        .hide_window()
        .args(["-ano"])
        .output()
        .map_err(|e| format!("执行 netstat 失败: {e}"))?;
    if !output.status.success() {
        return Err("netstat 返回非零状态".into());
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut rows = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("TCP") && !line.starts_with("UDP") {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }
        let protocol = parts[0].to_string();
        let local = parts[1].to_string();
        if !is_local_binding(&local) {
            continue;
        }
        if protocol == "TCP" && parts.len() >= 5 {
            rows.push(NetRow {
                protocol,
                local,
                remote: parts[2].to_string(),
                state: parts[3].to_string(),
                pid: parts[4].parse().unwrap_or(0),
            });
        } else if protocol == "UDP" && parts.len() >= 4 {
            rows.push(NetRow {
                protocol,
                local,
                remote: parts[2].to_string(),
                state: "LISTENING".into(),
                pid: parts[3].parse().unwrap_or(0),
            });
        }
    }
    Ok(rows)
}

fn is_local_binding(addr: &str) -> bool {
    let lower = addr.to_lowercase();
    lower.starts_with("127.0.0.1:")
        || lower.starts_with("[::1]:")
        || lower.starts_with("0.0.0.0:")
        || lower.starts_with("[::]:")
}

fn load_excluded_ranges() -> Result<Vec<(u16, u16)>, String> {
    let output = Command::new("netsh")
        .hide_window()
        .args([
            "interface",
            "ipv4",
            "show",
            "excludedportrange",
            "protocol=tcp",
        ])
        .output()
        .map_err(|e| format!("执行 netsh 失败: {e}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let mut ranges = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.contains("开始端口") || line.contains("---") {
            continue;
        }
        let nums: Vec<u16> = line
            .split_whitespace()
            .filter_map(|p| p.parse().ok())
            .collect();
        if nums.len() >= 2 {
            ranges.push((nums[0], nums[1]));
        }
    }
    Ok(ranges)
}

fn build_process_cache() -> HashMap<u32, String> {
    let mut map = HashMap::new();
    unsafe {
        let Ok(snapshot) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return map;
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let name = String::from_utf16_lossy(
                    &entry
                        .szExeFile
                        .iter()
                        .take_while(|&&c| c != 0)
                        .copied()
                        .collect::<Vec<_>>(),
                );
                map.insert(entry.th32ProcessID, name);
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
    map
}

fn process_name(pid: u32, cache: &HashMap<u32, String>) -> Option<String> {
    cache.get(&pid).cloned()
}

fn terminate_pid(pid: u32) -> Result<(), String> {
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, false, pid)
            .map_err(|e| format!("OpenProcess 失败 (PID {pid}): {e}"))?;
        TerminateProcess(handle, 1).map_err(|e| format!("结束进程失败 (PID {pid}): {e}"))?;
        let _ = CloseHandle(handle);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ipv4_port() {
        assert_eq!(parse_port("127.0.0.1:55555"), Some(55555));
    }

    #[test]
    fn detects_dev_residual_node() {
        assert!(is_releasable_process("node.exe", 123, 1));
        assert!(!is_releasable_process("chrome.exe", 123, 1));
    }
}
