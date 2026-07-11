//! Task 3：BSOD 事后追溯

use crate::events::BsodAlertEvent;
use crate::notify;
use crate::settings;
use crate::tray::{self, TrayLevel};
use crate::utils::logging;
use crate::utils::wmi_runner;
use chrono::{DateTime, Duration, Local, Utc};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Emitter};
use wmi::WMIConnection;

const MINIDUMP_DIR: &str = r"C:\Windows\Minidump";
const ALERT_WINDOW: Duration = Duration::hours(24);

#[derive(Debug, Clone)]
pub struct BsodReport {
    pub dump_path: PathBuf,
    pub modified_at: DateTime<Local>,
    pub is_recent: bool,
    pub bugcheck_code: Option<String>,
    pub faulting_driver: Option<String>,
    pub event_message: Option<String>,
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

/// 启动时扫描 Minidump 并通过 Tauri Event 推送
pub fn startup_scan_emit(app: &AppHandle) {
    match analyze_latest_dump() {
        Ok(Some(report)) => {
            emit_report(&report);
            emit_bsod_event(app, &report);
        }
        Ok(None) => logging::info("Minidump 目录无 .dmp 文件，跳过 BSOD 追溯"),
        Err(e) => logging::error(format!("BSOD 扫描失败: {e}")),
    }
}

fn emit_bsod_event(app: &AppHandle, report: &BsodReport) {
    let event = BsodAlertEvent {
        timestamp: Local::now().to_rfc3339(),
        is_recent: report.is_recent,
        dump_path: report.dump_path.display().to_string(),
        bugcheck_code: report.bugcheck_code.clone(),
        faulting_driver: report.faulting_driver.clone(),
        message: report.event_message.clone(),
    };
    if let Err(e) = app.emit("bsod-alert", &event) {
        logging::error(format!("emit bsod-alert 失败: {e}"));
    }
    if report.is_recent {
        let reason = report
            .bugcheck_code
            .clone()
            .unwrap_or_else(|| "BSOD".into());
        tray::set_level(app, TrayLevel::Critical, &reason);
        let body = report
            .event_message
            .clone()
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
    Ok(Some(BsodReport {
        dump_path: path,
        modified_at,
        is_recent,
        bugcheck_code: event_info.0,
        faulting_driver: event_info.1,
        event_message: event_info.2,
    }))
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

fn query_bugcheck_event() -> (Option<String>, Option<String>, Option<String>) {
    wmi_runner::run(query_bugcheck_inner).unwrap_or_else(|e| {
        logging::warn(format!("BugCheck 事件 WMI 查询失败: {e}"));
        (None, None, None)
    })
}

fn query_bugcheck_inner(
    wmi: &WMIConnection,
) -> Result<(Option<String>, Option<String>, Option<String>), wmi::WMIError> {
    let query = "SELECT EventCode, Message, TimeGenerated, SourceName \
                 FROM Win32_NTLogEvent \
                 WHERE Logfile='System' AND EventCode=1001 \
                 ORDER BY TimeGenerated DESC";
    let events: Vec<NtLogEvent> = wmi.raw_query(query)?;
    let event = match events.first() {
        Some(e) => e,
        None => return Ok((None, None, None)),
    };
    let message = event.message.clone();
    let bugcheck = message.as_ref().and_then(|m| parse_bugcheck_code(m));
    let driver = message.as_ref().and_then(|m| parse_faulting_driver(m));
    Ok((bugcheck, driver, message))
}

fn parse_bugcheck_code(message: &str) -> Option<String> {
    let lower = message.to_lowercase();
    let marker = "bugcheck was:";
    let pos = lower.find(marker)?;
    let rest = &message[pos + marker.len()..];
    let code = rest.trim().split_whitespace().next()?;
    Some(code.to_uppercase())
}

fn parse_faulting_driver(message: &str) -> Option<String> {
    for token in message.split_whitespace() {
        let cleaned = token.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '.');
        if cleaned.ends_with(".sys") || cleaned.ends_with(".SYS") {
            return Some(cleaned.to_lowercase());
        }
    }
    None
}

fn emit_report(report: &BsodReport) {
    if report.is_recent {
        logging::critical("⚠ BSOD 高能预警 — 过去 24 小时内发生蓝屏！");
    }
    logging::warn(format!(
        "Dump: {} ({})",
        report.dump_path.display(),
        report.modified_at.format("%Y-%m-%d %H:%M:%S")
    ));
    if let Some(code) = &report.bugcheck_code {
        logging::critical(format!("BugCheck: {code}"));
    }
    if let Some(driver) = &report.faulting_driver {
        logging::critical(format!("肇事驱动: {driver}"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bugcheck_code() {
        let msg = "The bugcheck was: 0x000000d1 (0x1, 0x2, 0x0, 0x4).";
        assert_eq!(
            parse_bugcheck_code(msg).as_deref(),
            Some("0X000000D1")
        );
    }
}
