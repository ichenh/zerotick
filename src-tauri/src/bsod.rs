//! Task 3：BSOD 事后追溯 — 转储分析、根因推断与修复

use crate::events::{BsodAlertEvent, BsodFixAction};
use crate::notify;
use crate::settings;
use crate::tray::{self, TrayLevel};
use crate::utils::{logging, powershell, process::CommandExt, wmi_runner};
use chrono::{DateTime, Duration, Local, Utc};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::thread;
use std::time::{Duration as StdDuration, Instant};
use tauri::{AppHandle, Emitter};
use wmi::WMIConnection;

const MINIDUMP_DIR: &str = r"C:\Windows\Minidump";
const ALERT_WINDOW: Duration = Duration::hours(24);
const SYSTEM_REPAIR_TIMEOUT: StdDuration = StdDuration::from_secs(2 * 60 * 60);
static SEEN_STORE: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct BsodReport {
    pub dump_path: PathBuf,
    pub modified_at: DateTime<Local>,
    pub is_recent: bool,
    pub bugcheck_code: Option<String>,
    pub faulting_driver: Option<String>,
    pub event_message: Option<String>,
    debugger_evidence: Option<DebuggerEvidence>,
}

#[derive(Debug, Clone)]
struct BsodAnalysis {
    code_name: Option<String>,
    analysis_id: String,
    analysis_kind: String,
    root_cause: String,
    fixes: Vec<BsodFixAction>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
struct DebuggerEvidence {
    debugger: String,
    bugcheck_code: Option<String>,
    module: Option<String>,
    image: Option<String>,
    failure_bucket: Option<String>,
    stack_summary: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct SeenDump {
    path: String,
    modified_at: i64,
    debugger_attempted: bool,
    debugger_evidence: Option<DebuggerEvidence>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NtLogEvent {
    #[serde(rename = "EventCode")]
    event_code: Option<u32>,
    #[serde(rename = "Message")]
    message: Option<String>,
    #[serde(rename = "TimeGenerated")]
    time_generated: Option<String>,
    #[serde(rename = "SourceName")]
    source_name: Option<String>,
}

pub fn init_seen_store(path: PathBuf) {
    let _ = SEEN_STORE.set(path);
}

/// 启动时扫描 Minidump 并通过 Tauri Event 推送
pub fn startup_scan_emit(app: &AppHandle) {
    match analyze_latest_dump() {
        Ok(Some(report)) => {
            if should_log_report(&report) {
                emit_report(&report);
            }
            emit_bsod_event(app, &report);
        }
        Ok(None) => logging::info("Minidump 目录无 .dmp 文件，跳过 BSOD 追溯"),
        Err(e) => logging::error(format!("BSOD 扫描失败: {e}")),
    }
}

pub fn report_to_event(report: &BsodReport) -> BsodAlertEvent {
    let module = report
        .debugger_evidence
        .as_ref()
        .and_then(|evidence| evidence.image.as_deref().or(evidence.module.as_deref()));
    let analysis = build_analysis(report.bugcheck_code.as_deref(), module);
    BsodAlertEvent {
        timestamp: Local::now().to_rfc3339(),
        is_recent: report.is_recent,
        dump_path: report.dump_path.display().to_string(),
        dump_time: Some(report.modified_at.format("%Y-%m-%d %H:%M:%S").to_string()),
        bugcheck_code: report.bugcheck_code.clone(),
        code_name: analysis.code_name,
        analysis_id: analysis.analysis_id,
        analysis_kind: analysis.analysis_kind,
        faulting_driver: report.faulting_driver.clone(),
        faulting_module: module.map(str::to_string),
        stack_summary: report
            .debugger_evidence
            .as_ref()
            .and_then(|evidence| evidence.stack_summary.clone()),
        debugger: report
            .debugger_evidence
            .as_ref()
            .map(|evidence| evidence.debugger.clone()),
        failure_bucket: report
            .debugger_evidence
            .as_ref()
            .and_then(|evidence| evidence.failure_bucket.clone()),
        root_cause: Some(analysis.root_cause),
        fixes: analysis.fixes,
        message: report.event_message.clone(),
    }
}

fn emit_bsod_event(app: &AppHandle, report: &BsodReport) {
    let event = report_to_event(report);
    if let Err(e) = app.emit("bsod-alert", &event) {
        logging::error(format!("emit bsod-alert 失败: {e}"));
    }
    if report.is_recent {
        let reason = report
            .bugcheck_code
            .clone()
            .unwrap_or_else(|| "BSOD".into());
        tray::set_level(app, TrayLevel::Critical, &reason);
        let body = event
            .root_cause
            .clone()
            .or_else(|| report.event_message.clone())
            .unwrap_or_else(|| format!("BugCheck {reason}"));
        notify::send_if_background(
            app,
            &crate::i18n::notify_bsod_title(&settings::get().locale),
            &body,
        );
    }
}

pub fn analyze_latest_dump() -> windows::core::Result<Option<BsodReport>> {
    let dump_dir = Path::new(MINIDUMP_DIR);
    if !dump_dir.is_dir() {
        return Ok(None);
    }
    let latest = find_latest_dump(dump_dir)?;
    let Some((path, modified_utc)) = latest else {
        return Ok(None);
    };
    let modified_at: DateTime<Local> = modified_utc.into();
    let is_recent = Utc::now().signed_duration_since(modified_utc) < ALERT_WINDOW;
    let event_info = query_bugcheck_event();
    let cached = (!is_recent)
        .then(|| load_seen_dump(&path, modified_at.timestamp_millis()))
        .flatten();
    let debugger_evidence = match cached {
        Some(seen) if seen.debugger_attempted => seen.debugger_evidence,
        _ => analyze_dump_with_debugger(&path),
    };
    let bugcheck_code = debugger_evidence
        .as_ref()
        .and_then(|evidence| evidence.bugcheck_code.clone())
        .or(event_info.0);
    let faulting_driver = debugger_evidence.as_ref().and_then(|evidence| {
        evidence
            .image
            .as_deref()
            .or(evidence.module.as_deref())
            .filter(|name| name.to_ascii_lowercase().ends_with(".sys"))
            .map(str::to_string)
    });
    Ok(Some(BsodReport {
        dump_path: path,
        modified_at,
        is_recent,
        bugcheck_code,
        faulting_driver,
        event_message: event_info.2,
        debugger_evidence,
    }))
}

pub fn apply_repairs(fix_ids: Vec<String>) -> Result<Vec<String>, String> {
    if !crate::utils::elevated::is_elevated() {
        return Err("bsod_repair:admin_required".into());
    }
    let mut results = Vec::new();
    for id in fix_ids {
        let result = match id.as_str() {
            "scan_devices" => run_pnputil_scan(),
            "check_image" => run_dism_check_health(),
            "chkdsk_scan" => run_chkdsk_scan(),
            "sfc_verify" => run_sfc_verify(),
            _ => continue,
        };
        match result {
            Ok(message) => results.push(format!("✓ {message}")),
            Err(error) => results.push(format!("✗ {id}: {error}")),
        }
    }
    if results.is_empty() {
        return Err("没有可执行的修复项".into());
    }
    Ok(results)
}

fn run_pnputil_scan() -> Result<String, String> {
    run_repair_script(
        r#"
$text = & pnputil.exe /scan-devices 2>&1 | Out-String
[pscustomobject]@{ exit_code = [int]$LASTEXITCODE; output = $text.Trim() }
"#,
        "PnPUtil 设备扫描",
        &[0],
    )
}

fn run_dism_check_health() -> Result<String, String> {
    run_repair_script(
        r#"
$text = & DISM.exe /Online /Cleanup-Image /RestoreHealth 2>&1 | Out-String
[pscustomobject]@{ exit_code = [int]$LASTEXITCODE; output = $text.Trim() }
"#,
        "DISM 组件存储修复",
        &[0],
    )
}

fn run_chkdsk_scan() -> Result<String, String> {
    run_repair_script(
        r#"
$drive = $env:SystemDrive
if (-not $drive) { $drive = 'C:' }
$text = & chkdsk.exe $drive /scan 2>&1 | Out-String
[pscustomobject]@{ exit_code = [int]$LASTEXITCODE; output = $text.Trim() }
"#,
        "CHKDSK 系统卷扫描",
        &[0, 1],
    )
}

fn run_sfc_verify() -> Result<String, String> {
    run_repair_script(
        r#"
$text = & sfc.exe /scannow 2>&1 | Out-String
[pscustomobject]@{ exit_code = [int]$LASTEXITCODE; output = $text.Trim() }
"#,
        "SFC 系统文件检查",
        &[0],
    )
}

fn run_repair_script(script: &str, label: &str, success_codes: &[i64]) -> Result<String, String> {
    // DISM, SFC and online CHKDSK are explicit user-triggered maintenance tasks and can
    // legitimately take far longer than an ordinary diagnostic query. Keep a finite
    // upper bound without reusing the short system-query timeout.
    let value = powershell::run_json_with_timeout(script, SYSTEM_REPAIR_TIMEOUT)?;
    let exit_code = value
        .get("exit_code")
        .and_then(|item| item.as_i64())
        .unwrap_or(-1);
    let output = value
        .get("output")
        .and_then(|item| item.as_str())
        .unwrap_or("");
    let detail = summarize_command_output(output);
    if success_codes.contains(&exit_code) {
        if !detail.is_empty() {
            logging::info(format!("{label}输出: {detail}"));
        }
        Ok(format!("{label}已完成"))
    } else {
        if !detail.is_empty() {
            logging::warn(format!("{label}失败（退出码 {exit_code}）: {detail}"));
        }
        Err(format!(
            "{label}失败（退出码 {exit_code}），详细信息已写入诊断日志"
        ))
    }
}

fn summarize_command_output(output: &str) -> String {
    let lines: Vec<&str> = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let start = lines.len().saturating_sub(4);
    let mut summary = lines[start..].join(" ");
    if summary.chars().count() > 600 {
        summary = summary.chars().take(597).collect::<String>() + "…";
    }
    summary
}

fn build_analysis(code: Option<&str>, driver: Option<&str>) -> BsodAnalysis {
    let normalized = code.map(normalize_bugcheck);
    let (code_name, root_cause, mut fixes) = match normalized.as_deref() {
        Some("0x0000000A") => (
            Some("IRQL_NOT_LESS_OR_EQUAL"),
            "内核或驱动在无效内存地址上执行操作，常见于驱动冲突、超频或内存故障。",
            base_fixes(true),
        ),
        Some("0x0000001E") => (
            Some("KMODE_EXCEPTION_NOT_HANDLED"),
            "内核模式代码抛出未处理异常，多为驱动或杀毒/虚拟化软件冲突。",
            base_fixes(true),
        ),
        Some("0x0000003B") => (
            Some("SYSTEM_SERVICE_EXCEPTION"),
            "系统服务执行时发生异常，常见于损坏的驱动或系统文件。",
            base_fixes(true),
        ),
        Some("0x00000050") => (
            Some("PAGE_FAULT_IN_NONPAGED_AREA"),
            "访问了无效的不可分页内存，可能由驱动、内存条或磁盘故障引起。",
            base_fixes(true),
        ),
        Some("0x0000007E") => (
            Some("SYSTEM_THREAD_EXCEPTION_NOT_HANDLED"),
            "系统线程未处理异常，多见于显卡/存储/网络驱动。",
            base_fixes(true),
        ),
        Some("0x0000007A") => (
            Some("KERNEL_DATA_INPAGE_ERROR"),
            "无法将所需数据读入内存，常见于硬盘/线缆/存储驱动问题。",
            vec![fix("chkdsk_scan", true), fix("check_image", true)],
        ),
        Some("0x000000D1") => (
            Some("DRIVER_IRQL_NOT_LESS_OR_EQUAL"),
            "驱动在过高的中断级别访问了无效内存，几乎总是驱动程序问题。",
            base_fixes(true),
        ),
        Some("0x000000EF") => (
            Some("CRITICAL_PROCESS_DIED"),
            "关键系统进程意外退出，可能由驱动、系统文件损坏或恶意软件导致。",
            base_fixes(true),
        ),
        Some("0x00000109") => (
            Some("CRITICAL_STRUCTURE_CORRUPTION"),
            "关键内核数据结构损坏，常见于驱动或硬盘/内存硬件故障。",
            base_fixes(true),
        ),
        Some("0x00000133") => (
            Some("DPC_WATCHDOG_VIOLATION"),
            "延迟过程调用超时，常见于驱动挂起、固件或存储/网卡驱动问题。",
            base_fixes(true),
        ),
        Some("0x00000139") => (
            Some("KERNEL_SECURITY_CHECK_FAILURE"),
            "内核安全检查失败，多见于驱动越界写或内存损坏。",
            base_fixes(true),
        ),
        Some("0x0000009F") => (
            Some("DRIVER_POWER_STATE_FAILURE"),
            "驱动电源状态切换失败，常见于笔记本睡眠/唤醒或 USB/显卡驱动。",
            base_fixes(true),
        ),
        _ => (
            None,
            "系统记录了蓝屏转储。请结合错误码、驱动与下方修复项进一步排查。",
            base_fixes(false),
        ),
    };

    let mut root_cause = root_cause.to_string();
    if let Some(name) = code_name {
        root_cause = format!("{root_cause}（{name}）");
    }
    let analysis_kind = if let Some(module) =
        driver.filter(|module| is_specific_fault_module(module))
    {
        root_cause = format!(
            "WinDbg/DbgEng 的 !analyze -v 与调用栈取得模块证据：{module}。错误类型背景：{root_cause}"
        );
        if is_driver_module(module) {
            fixes.push(fix("review_faulting_driver", false));
            fixes.push(fix("scan_devices", true));
        }
        "root_cause"
    } else {
        "error_type"
    };

    BsodAnalysis {
        code_name: code_name.map(str::to_string),
        analysis_id: code_name
            .map(|name| name.to_ascii_lowercase())
            .unwrap_or_else(|| "unknown".into()),
        analysis_kind: analysis_kind.into(),
        root_cause,
        fixes,
    }
}

fn fix(id: &str, automatic: bool) -> BsodFixAction {
    BsodFixAction {
        id: id.into(),
        automatic,
    }
}

fn base_fixes(_driver_likely: bool) -> Vec<BsodFixAction> {
    vec![
        fix("check_image", true),
        fix("sfc_verify", true),
        fix("chkdsk_scan", true),
    ]
}

fn is_driver_module(module: &str) -> bool {
    let lower = module.to_ascii_lowercase();
    lower.ends_with(".sys") && !matches!(lower.as_str(), "ntoskrnl.exe" | "ntkrnlmp.exe")
}

fn is_specific_fault_module(module: &str) -> bool {
    !matches!(
        module.trim().to_ascii_lowercase().as_str(),
        "nt" | "ntoskrnl"
            | "ntoskrnl.exe"
            | "ntkrnlmp"
            | "ntkrnlmp.exe"
            | "memory_corruption"
            | "unknown"
    )
}

fn normalize_bugcheck(code: &str) -> String {
    let c = code.trim().to_uppercase();
    let hex = c.strip_prefix("0X").unwrap_or(&c);
    if let Ok(n) = u32::from_str_radix(hex, 16) {
        return format!("0x{:08X}", n);
    }
    c
}

fn find_latest_dump(dir: &Path) -> windows::core::Result<Option<(PathBuf, DateTime<Utc>)>> {
    let mut best: Option<(PathBuf, DateTime<Utc>)> = None;
    let entries = std::fs::read_dir(dir).map_err(|e| {
        windows::core::Error::new(
            windows::core::HRESULT::from_win32(windows::Win32::Foundation::ERROR_PATH_NOT_FOUND.0),
            format!("无法读取 Minidump 目录: {e}"),
        )
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("dmp") {
            continue;
        }
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        let Some(modified) = meta.modified().ok() else {
            continue;
        };
        let modified_utc: DateTime<Utc> = modified.into();
        if best.as_ref().is_none_or(|(_, t)| modified_utc > *t) {
            best = Some((path, modified_utc));
        }
    }
    Ok(best)
}

type BugcheckEvent = (Option<String>, Option<String>, Option<String>);

fn query_bugcheck_event() -> BugcheckEvent {
    if let Ok(info) = wmi_runner::run(query_bugcheck_inner) {
        return info;
    }
    query_bugcheck_powershell().unwrap_or_else(|e| {
        logging::warn(format!("BugCheck 事件查询失败: {e}"));
        (None, None, None)
    })
}

fn query_bugcheck_inner(wmi: &WMIConnection) -> Result<BugcheckEvent, wmi::WMIError> {
    let query = "SELECT EventCode, Message, TimeGenerated, SourceName \
                 FROM Win32_NTLogEvent \
                 WHERE Logfile='System' AND EventCode=1001";
    let mut events: Vec<NtLogEvent> = wmi.raw_query(query).unwrap_or_default();
    events.sort_by(|a, b| {
        b.time_generated
            .as_deref()
            .unwrap_or("")
            .cmp(a.time_generated.as_deref().unwrap_or(""))
    });
    let event = match events.iter().find(|event| {
        event
            .message
            .as_deref()
            .and_then(parse_bugcheck_code)
            .is_some()
    }) {
        Some(e) => e,
        None => return Ok((None, None, None)),
    };
    let message = event.message.clone();
    let bugcheck = message.as_ref().and_then(|m| parse_bugcheck_code(m));
    let driver = message.as_ref().and_then(|m| parse_faulting_driver(m));
    Ok((bugcheck, driver, message))
}

fn query_bugcheck_powershell() -> Result<BugcheckEvent, String> {
    let script = r#"
$e = Get-WinEvent -FilterHashtable @{LogName='System'; Id=1001} -MaxEvents 20 -ErrorAction SilentlyContinue |
  Where-Object { $_.Message -match '0x[0-9a-fA-F]{1,8}' } |
  Select-Object -First 1
if (-not $e) { return $null }
[pscustomobject]@{ message = [string]$e.Message }
"#;
    let val = powershell::run_json(script)?;
    if val.is_null() {
        return Ok((None, None, None));
    }
    let message = val
        .get("message")
        .and_then(|v| v.as_str())
        .map(str::to_string);
    let bugcheck = message.as_ref().and_then(|m| parse_bugcheck_code(m));
    let driver = message.as_ref().and_then(|m| parse_faulting_driver(m));
    Ok((bugcheck, driver, message))
}

fn parse_bugcheck_code(message: &str) -> Option<String> {
    let bytes = message.as_bytes();
    for index in 0..bytes.len().saturating_sub(2) {
        if bytes[index] != b'0' || !matches!(bytes[index + 1], b'x' | b'X') {
            continue;
        }
        let start = index + 2;
        let mut end = start;
        while end < bytes.len() && end - start < 8 && bytes[end].is_ascii_hexdigit() {
            end += 1;
        }
        if end > start {
            return Some(normalize_bugcheck(&message[index..end]));
        }
    }
    None
}

fn parse_faulting_driver(message: &str) -> Option<String> {
    let lower = message.to_lowercase();
    for marker in [
        "probably caused by",
        "faulting driver",
        "fault bucket",
        "image name",
    ] {
        if let Some(pos) = lower.find(marker) {
            let rest = &message[pos + marker.len()..];
            if let Some(drv) = extract_sys_name(rest) {
                return Some(drv);
            }
        }
    }
    extract_sys_name(message)
}

fn extract_sys_name(text: &str) -> Option<String> {
    for token in text.split_whitespace() {
        let cleaned = token.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.');
        if cleaned.to_ascii_lowercase().ends_with(".sys") {
            return Some(cleaned.to_ascii_lowercase());
        }
    }
    None
}

fn analyze_dump_with_debugger(dump_path: &Path) -> Option<DebuggerEvidence> {
    let debugger = match find_cdb() {
        Some(path) => path,
        None => {
            logging::info("WinDbg/DbgEng 未安装，保留错误类型分析");
            return None;
        }
    };
    let symbol_cache = std::env::temp_dir().join("ZeroTickSymbols");
    let _ = std::fs::create_dir_all(&symbol_cache);
    let symbol_path = format!(
        "srv*{}*https://msdl.microsoft.com/download/symbols",
        symbol_cache.display()
    );
    let mut command = Command::new(&debugger);
    command
        .hide_window()
        .args(["-y", &symbol_path, "-z"])
        .arg(dump_path)
        .args([
            "-c",
            "!analyze -v; .echo ZEROTICK_STACK_BEGIN; kv 20; .echo ZEROTICK_STACK_END; q",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let debugger_timeout = StdDuration::from_secs(settings::get().bsod_debugger_timeout_secs);
    let output = match run_with_timeout(command, debugger_timeout) {
        Ok(output) => output,
        Err(error) => {
            logging::info(format!(
                "WinDbg/DbgEng 分析未完成，保留错误类型分析: {error}"
            ));
            return None;
        }
    };
    let text = String::from_utf8_lossy(&output).to_string();
    let evidence = parse_debugger_output(
        debugger
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("cdb.exe"),
        &text,
    );
    if evidence.module.is_some() || evidence.image.is_some() {
        logging::info(format!(
            "WinDbg/DbgEng 分析完成，模块证据={}",
            evidence
                .image
                .as_deref()
                .or(evidence.module.as_deref())
                .unwrap_or("—")
        ));
    } else {
        logging::info("WinDbg/DbgEng 已运行，但未取得可验证的故障模块");
    }
    Some(evidence)
}

fn find_cdb() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    for root in [
        std::env::var_os("ProgramFiles(x86)"),
        std::env::var_os("ProgramFiles"),
    ]
    .into_iter()
    .flatten()
    {
        for arch in ["x64", "x86", "arm64"] {
            candidates.push(
                PathBuf::from(&root)
                    .join("Windows Kits")
                    .join("10")
                    .join("Debuggers")
                    .join(arch)
                    .join("cdb.exe"),
            );
        }
    }
    if let Some(path) = candidates.into_iter().find(|path| path.is_file()) {
        return Some(path);
    }
    let output = Command::new("where.exe")
        .hide_window()
        .arg("cdb.exe")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

fn run_with_timeout(mut command: Command, timeout: StdDuration) -> Result<Vec<u8>, String> {
    let mut child = command
        .spawn()
        .map_err(|error| format!("无法启动调试器: {error}"))?;
    let mut stdout = child.stdout.take().ok_or("无法捕获调试器输出")?;
    let mut stderr = child.stderr.take().ok_or("无法捕获调试器错误输出")?;
    let stdout_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stdout.read_to_end(&mut bytes);
        bytes
    });
    let stderr_reader = thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stderr.read_to_end(&mut bytes);
        bytes
    });
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("读取调试器状态失败: {error}"))?
        {
            break status;
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(format!("调试器超过 {} 秒未完成", timeout.as_secs()));
        }
        thread::sleep(StdDuration::from_millis(100));
    };
    let mut output = stdout_reader.join().unwrap_or_default();
    let stderr = stderr_reader.join().unwrap_or_default();
    output.extend_from_slice(&stderr);
    if !status.success() {
        return Err(format!(
            "调试器退出码 {}: {}",
            status.code().unwrap_or(-1),
            summarize_command_output(&String::from_utf8_lossy(&output))
        ));
    }
    Ok(output)
}

fn parse_debugger_output(debugger: &str, output: &str) -> DebuggerEvidence {
    DebuggerEvidence {
        debugger: debugger.to_string(),
        bugcheck_code: debugger_field(output, "BUGCHECK_CODE")
            .map(|code| normalize_bugcheck(code.trim_start_matches("0x"))),
        module: debugger_field(output, "MODULE_NAME"),
        image: debugger_field(output, "IMAGE_NAME"),
        failure_bucket: debugger_field(output, "FAILURE_BUCKET_ID"),
        stack_summary: summarize_stack(output),
    }
}

fn debugger_field(output: &str, field: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if key.trim().eq_ignore_ascii_case(field) {
            let value = value.trim();
            (!value.is_empty()).then(|| value.to_string())
        } else {
            None
        }
    })
}

fn summarize_stack(output: &str) -> Option<String> {
    let stack = output
        .split_once("ZEROTICK_STACK_BEGIN")?
        .1
        .split_once("ZEROTICK_STACK_END")?
        .0;
    let mut frames = Vec::new();
    for token in stack.split_whitespace().filter(|token| token.contains('!')) {
        let frame = token
            .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && !matches!(ch, '!' | '_' | '.'))
            .split('+')
            .next()
            .unwrap_or("");
        if !frame.is_empty() && !frames.iter().any(|existing| existing == frame) {
            frames.push(frame.to_string());
        }
        if frames.len() == 8 {
            break;
        }
    }
    (!frames.is_empty()).then(|| frames.join(" → "))
}

fn should_log_report(report: &BsodReport) -> bool {
    if report.is_recent {
        return true;
    }
    let Some(store) = SEEN_STORE.get() else {
        return true;
    };
    let current = SeenDump {
        path: report.dump_path.display().to_string(),
        modified_at: report.modified_at.timestamp_millis(),
        debugger_attempted: true,
        debugger_evidence: report.debugger_evidence.clone(),
    };
    let previous = std::fs::read_to_string(store)
        .ok()
        .and_then(|text| serde_json::from_str::<SeenDump>(&text).ok());
    if previous.as_ref() == Some(&current) {
        logging::info(format!(
            "旧转储已记录，跳过重复诊断日志: {}",
            report.dump_path.display()
        ));
        return false;
    }
    if let Ok(text) = serde_json::to_string(&current) {
        let _ = std::fs::write(store, text);
    }
    true
}

fn load_seen_dump(path: &Path, modified_at: i64) -> Option<SeenDump> {
    let store = SEEN_STORE.get()?;
    let seen = std::fs::read_to_string(store)
        .ok()
        .and_then(|text| serde_json::from_str::<SeenDump>(&text).ok())?;
    (seen.path == path.display().to_string() && seen.modified_at == modified_at).then_some(seen)
}

fn emit_report(report: &BsodReport) {
    if report.is_recent {
        logging::critical("⚠ BSOD 高能预警 — 过去 24 小时内发生蓝屏！");
        logging::warn(format!(
            "Dump: {} ({})",
            report.dump_path.display(),
            report.modified_at.format("%Y-%m-%d %H:%M:%S")
        ));
    } else {
        logging::info(format!(
            "历史 Dump: {} ({})",
            report.dump_path.display(),
            report.modified_at.format("%Y-%m-%d %H:%M:%S")
        ));
    }
    if let Some(code) = &report.bugcheck_code {
        if report.is_recent {
            logging::critical(format!("BugCheck: {code}"));
        } else {
            logging::info(format!("历史 BugCheck: {code}"));
        }
    }
    if let Some(driver) = &report.faulting_driver {
        if report.is_recent {
            logging::critical(format!("WinDbg 驱动证据: {driver}"));
        } else {
            logging::info(format!("历史 WinDbg 驱动证据: {driver}"));
        }
    }
    let module = report
        .debugger_evidence
        .as_ref()
        .and_then(|evidence| evidence.image.as_deref().or(evidence.module.as_deref()));
    let analysis = build_analysis(report.bugcheck_code.as_deref(), module);
    let label = if analysis.analysis_kind == "root_cause" {
        "具体根因"
    } else {
        "错误类型分析"
    };
    if report.is_recent {
        logging::warn(format!("{label}: {}", analysis.root_cause));
    } else {
        logging::info(format!("历史{label}: {}", analysis.root_cause));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bugcheck_code() {
        let msg = "The bugcheck was: 0x000000d1 (0x1, 0x2, 0x0, 0x4).";
        assert_eq!(parse_bugcheck_code(msg).as_deref(), Some("0x000000D1"));
    }

    #[test]
    fn parses_localized_bugcheck_code() {
        let msg = "计算机已经从检测错误后重新启动。检测错误: 0x00000133 (0x0, 0x1)。";
        assert_eq!(parse_bugcheck_code(msg).as_deref(), Some("0x00000133"));
    }

    #[test]
    fn command_output_summary_keeps_the_actionable_tail() {
        let output = "line 1\n\nline 2\nline 3\nline 4\nfinal result";
        assert_eq!(
            summarize_command_output(output),
            "line 2 line 3 line 4 final result"
        );
    }

    #[test]
    fn parses_debugger_module_bucket_and_stack() {
        let output = r#"
BUGCHECK_CODE:  a
MODULE_NAME: vendor_driver
IMAGE_NAME: vendor_driver.sys
FAILURE_BUCKET_ID: AV_vendor_driver!HandleRequest
ZEROTICK_STACK_BEGIN
fffff800`00000000 fffff800`00000001 nt!KeBugCheckEx
fffff800`00000010 fffff800`00000011 vendor_driver!HandleRequest+0x42
ZEROTICK_STACK_END
"#;
        let evidence = parse_debugger_output("cdb.exe", output);
        assert_eq!(evidence.bugcheck_code.as_deref(), Some("0x0000000A"));
        assert_eq!(evidence.image.as_deref(), Some("vendor_driver.sys"));
        assert_eq!(
            evidence.stack_summary.as_deref(),
            Some("nt!KeBugCheckEx → vendor_driver!HandleRequest")
        );
    }

    #[test]
    fn stop_code_alone_is_not_reported_as_a_specific_root_cause() {
        let analysis = build_analysis(Some("0xA"), None);
        assert_eq!(analysis.analysis_kind, "error_type");
        assert!(!analysis.fixes.iter().any(|fix| fix.id == "scan_devices"));
    }

    #[test]
    fn debugger_driver_evidence_enables_targeted_driver_actions() {
        let analysis = build_analysis(Some("0xA"), Some("vendor_driver.sys"));
        assert_eq!(analysis.analysis_kind, "root_cause");
        assert!(analysis
            .fixes
            .iter()
            .any(|fix| fix.id == "review_faulting_driver"));
        assert!(analysis.fixes.iter().any(|fix| fix.id == "scan_devices"));
    }

    #[test]
    fn kernel_image_alone_is_not_a_specific_root_cause() {
        let analysis = build_analysis(Some("0xA"), Some("ntkrnlmp.exe"));
        assert_eq!(analysis.analysis_kind, "error_type");
    }
}
