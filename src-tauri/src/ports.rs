//! Windows 本地端口占用扫描与释放
use crate::utils::process::CommandExt;

use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, TerminateProcess, WaitForSingleObject, PROCESS_ACCESS_RIGHTS, PROCESS_TERMINATE,
};

const EXCLUDED_RANGES_TTL: Duration = Duration::from_secs(30);
const PROCESS_SYNCHRONIZE: PROCESS_ACCESS_RIGHTS = PROCESS_ACCESS_RIGHTS(0x0010_0000);
type ExcludedRangesCache = Option<(Instant, Vec<(u16, u16)>)>;
static EXCLUDED_RANGES_CACHE: OnceLock<Mutex<ExcludedRangesCache>> = OnceLock::new();

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
    /// The user may explicitly terminate this known, non-critical process.
    /// This is intentionally broader than `can_release`, which is reserved for
    /// the conservative batch cleanup allowlist.
    pub can_terminate: bool,
    pub message: String,
    /// 排序权重：可解除残留 < 可解除连接 < 普通占用 < 系统保留
    pub sort_priority: u8,
    /// Reserved for backward-compatible payloads. Windows does not expose a
    /// supported API for deleting an individual TCP TIME_WAIT entry.
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

pub fn release_pid(pid: u32, expected_process_name: &str, port: u16) -> Result<(), String> {
    let name = validate_terminable_process(pid)?;
    validate_port_owner(pid, &name, expected_process_name, port)?;
    terminate_pid(pid)?;
    Ok(())
}

fn release_releasable_pid(pid: u32, expected_process_name: &str, port: u16) -> Result<(), String> {
    let name = validate_terminable_process(pid)?;
    validate_port_owner(pid, &name, expected_process_name, port)?;
    if !is_releasable_process(&name, pid, std::process::id()) {
        return Err(format!("{name} (PID {pid}) 不属于可安全清理的进程"));
    }
    terminate_pid(pid)
}

pub fn release_all_releasable() -> Result<ReleaseReport, String> {
    let report = scan()?;
    let mut released = Vec::new();
    let mut errors = Vec::new();
    let mut seen = HashSet::new();

    for entry in report.entries.iter().filter(|e| e.can_release) {
        let Some(pid) = entry.pid else { continue };
        let Some(process_name) = entry.process_name.as_deref() else {
            continue;
        };
        if !seen.insert(format!("pid:{pid}")) {
            continue;
        }
        match release_releasable_pid(pid, process_name, entry.port) {
            Ok(()) => released.push(pid),
            Err(e) => errors.push(e),
        }
    }

    Ok(ReleaseReport {
        released_pids: released.into_iter().filter(|p| *p > 0).collect(),
        errors,
    })
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
        if row.state.eq_ignore_ascii_case("TIME_WAIT") {
            (
                "time_wait",
                "time_wait",
                "time_wait",
                false,
                row.state.as_str(),
                1,
                None,
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

    let can_terminate = pid_opt.is_some_and(|pid| {
        pid != self_pid
            && matches!(category, "releasable" | "in_use")
            && process_name
                .as_deref()
                .is_some_and(|name| !is_protected_process(name))
    });

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
        can_terminate,
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
    let cache = EXCLUDED_RANGES_CACHE.get_or_init(|| Mutex::new(None));
    let mut cache = cache
        .lock()
        .map_err(|_| "Reserved port range cache lock failed".to_string())?;
    if let Some((finished_at, ranges)) = cache.as_ref() {
        if finished_at.elapsed() <= EXCLUDED_RANGES_TTL {
            return Ok(ranges.clone());
        }
    }

    let ranges = query_excluded_ranges()?;
    *cache = Some((Instant::now(), ranges.clone()));
    Ok(ranges)
}

fn query_excluded_ranges() -> Result<Vec<(u16, u16)>, String> {
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

fn validate_terminable_process(pid: u32) -> Result<String, String> {
    let self_pid = std::process::id();
    if pid == 0 {
        return Err("无效 PID 0".into());
    }
    if pid == self_pid {
        return Err("不能结束当前 ZeroTick 进程".into());
    }

    let process_cache = build_process_cache();
    let Some(name) = process_name(pid, &process_cache) else {
        return Err(format!("PID {pid} 已退出或无法识别，请重新扫描端口"));
    };
    if is_protected_process(&name) {
        return Err(format!("{name} (PID {pid}) 为系统/关键进程，不可结束"));
    }
    Ok(name)
}

fn validate_port_owner(
    pid: u32,
    current_process_name: &str,
    expected_process_name: &str,
    port: u16,
) -> Result<(), String> {
    if !current_process_name.eq_ignore_ascii_case(expected_process_name) {
        return Err(format!(
            "PID {pid} 当前属于 {current_process_name}，与扫描结果 {expected_process_name} 不一致；请重新扫描端口"
        ));
    }
    let still_owns_port = parse_netstat()?
        .iter()
        .any(|row| row.pid == pid && parse_port(&row.local) == Some(port));
    if !still_owns_port {
        return Err(format!(
            "{current_process_name} (PID {pid}) 已不再占用端口 {port}；未结束进程，请重新扫描"
        ));
    }
    Ok(())
}

fn terminate_pid(pid: u32) -> Result<(), String> {
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE | PROCESS_SYNCHRONIZE, false, pid)
            .map_err(|e| format!("OpenProcess 失败 (PID {pid}): {e}"))?;
        if let Err(error) = TerminateProcess(handle, 1) {
            let _ = CloseHandle(handle);
            return Err(format!("结束进程失败 (PID {pid}): {error}"));
        }
        let wait_result = WaitForSingleObject(handle, 5_000);
        let _ = CloseHandle(handle);
        if wait_result != WAIT_OBJECT_0 {
            return Err(format!("进程未在预期时间内退出 (PID {pid})"));
        }
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

    #[test]
    fn ordinary_process_is_manual_only() {
        let row = NetRow {
            protocol: "TCP".into(),
            local: "127.0.0.1:8080".into(),
            remote: "0.0.0.0:0".into(),
            state: "LISTENING".into(),
            pid: 123,
        };
        let cache = HashMap::from([(123, "chrome.exe".to_string())]);
        let entry = build_entry(&row, &[], 999, &cache);
        assert_eq!(entry.category, "in_use");
        assert!(!entry.can_release);
        assert!(entry.can_terminate);
    }

    #[test]
    fn protected_process_has_no_termination_action() {
        let row = NetRow {
            protocol: "TCP".into(),
            local: "0.0.0.0:135".into(),
            remote: "0.0.0.0:0".into(),
            state: "LISTENING".into(),
            pid: 456,
        };
        let cache = HashMap::from([(456, "svchost.exe".to_string())]);
        let entry = build_entry(&row, &[], 999, &cache);
        assert!(!entry.can_release);
        assert!(!entry.can_terminate);
    }

    #[test]
    fn time_wait_is_informational_not_a_fake_release_action() {
        let row = NetRow {
            protocol: "TCP".into(),
            local: "127.0.0.1:55555".into(),
            remote: "127.0.0.1:60000".into(),
            state: "TIME_WAIT".into(),
            pid: 0,
        };
        let entry = build_entry(&row, &[], 999, &HashMap::new());
        assert_eq!(entry.category, "time_wait");
        assert!(!entry.can_release);
        assert!(!entry.can_terminate);
        assert!(entry.connection_key.is_none());
    }
}
