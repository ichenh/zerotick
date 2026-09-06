//! Explicit, administrator-only stack resets. Windows documents netsh for
//! these operations; there is no equivalent supported single-call reset API.
//! A zero exit code is command evidence, never proof of restored connectivity.

use crate::utils::{elevated, process::CommandExt};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResetKind {
    Winsock,
    Tcpip,
}

impl ResetKind {
    fn args(self) -> &'static [&'static str] {
        match self {
            Self::Winsock => &["winsock", "reset"],
            Self::Tcpip => &["int", "ip", "reset"],
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ResetResult {
    pub kind: ResetKind,
    pub needs_admin: bool,
    pub started: bool,
    pub exit_code: Option<i32>,
    pub timed_out: bool,
    pub output: String,
    pub output_truncated: bool,
    pub error: Option<String>,
}

pub fn run(kind: ResetKind, confirmed: bool) -> Result<ResetResult, String> {
    perform(kind, confirmed, elevated::is_elevated(), execute)
}

fn perform(
    kind: ResetKind,
    confirmed: bool,
    elevated: bool,
    execute: impl FnOnce(&mut ResetResult) -> Result<(), String>,
) -> Result<ResetResult, String> {
    if !confirmed {
        return Err("network_reset:confirmation_required".into());
    }
    let mut result = ResetResult {
        kind,
        needs_admin: !elevated,
        started: false,
        exit_code: None,
        timed_out: false,
        output: String::new(),
        output_truncated: false,
        error: None,
    };
    if elevated {
        result.error = execute(&mut result).err();
    }
    Ok(result)
}

struct OutputFile(std::path::PathBuf);

impl Drop for OutputFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn execute(result: &mut ResetResult) -> Result<(), String> {
    let mut directory = [0u16; 32768];
    let length = unsafe {
        windows::Win32::System::SystemInformation::GetSystemDirectoryW(Some(&mut directory))
    } as usize;
    if length == 0 || length >= directory.len() {
        return Err("system_directory_unavailable".into());
    }
    let executable =
        std::path::PathBuf::from(String::from_utf16_lossy(&directory[..length])).join("netsh.exe");
    // A private create_new file avoids pipe backpressure and keeps stdout/stderr
    // together, including netsh's per-component failures on a zero exit code.
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "zerotick-network-reset-{}-{}.log",
        std::process::id(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    ));
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| format!("output_create_failed:{error}"))?;
    let output_file = OutputFile(path);
    let stderr = file.try_clone().map_err(|error| error.to_string())?;
    let mut child = Command::new(executable)
        .hide_window()
        .args(result.kind.args())
        .stdin(Stdio::null())
        .stdout(file)
        .stderr(stderr)
        .spawn()
        .map_err(|error| format!("start_failed:{error}"))?;
    result.started = true;
    let started = Instant::now();
    let wait_result = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                result.exit_code = status.code();
                break Ok(());
            }
            Ok(None) if started.elapsed() < Duration::from_secs(30) => {
                std::thread::sleep(Duration::from_millis(50));
            }
            state => {
                result.timed_out = state.is_ok();
                let _ = child.kill();
                let _ = child.wait();
                break Err(match state {
                    Err(error) => format!("wait_failed:{error}"),
                    _ => "timeout".into(),
                });
            }
        }
    };
    const LIMIT: usize = 65536;
    let mut bytes = Vec::new();
    let read_result = std::fs::File::open(&output_file.0)
        .and_then(|file| file.take((LIMIT + 1) as u64).read_to_end(&mut bytes));
    result.output_truncated = bytes.len() > LIMIT;
    bytes.truncate(LIMIT);
    result.output = decode_output(&bytes);
    wait_result?;
    read_result.map_err(|error| format!("output_read_failed:{error}"))?;
    Ok(())
}

fn decode_output(bytes: &[u8]) -> String {
    if let Ok(text) = std::str::from_utf8(bytes) {
        return text.to_owned();
    }
    use windows::Win32::Globalization::{MultiByteToWideChar, CP_OEMCP};
    unsafe {
        let length = MultiByteToWideChar(CP_OEMCP, Default::default(), bytes, None);
        if length > 0 {
            let mut wide = vec![0u16; length as usize];
            let written = MultiByteToWideChar(CP_OEMCP, Default::default(), bytes, Some(&mut wide));
            if written > 0 {
                return String::from_utf16_lossy(&wide[..written as usize]);
            }
        }
    }
    String::from_utf8_lossy(bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resets_require_confirmation_and_elevation_without_executing() {
        assert!(perform(ResetKind::Winsock, false, true, |_| panic!("must not run")).is_err());
        let result = perform(ResetKind::Tcpip, true, false, |_| panic!("must not run")).unwrap();
        assert!(result.needs_admin);
        assert!(!result.started);
    }

    #[test]
    fn reset_result_preserves_partial_execution_and_failure_output() {
        let result = perform(ResetKind::Tcpip, true, true, |result| {
            result.started = true;
            result.output = "Some components failed".into();
            result.timed_out = true;
            Err("timeout".into())
        })
        .unwrap();
        assert!(result.started && result.timed_out);
        assert_eq!(result.output, "Some components failed");
        assert_eq!(result.error.as_deref(), Some("timeout"));
        assert_eq!(result.exit_code, None);
    }

    #[test]
    fn reset_kind_rejects_arbitrary_commands() {
        assert!(serde_json::from_str::<ResetKind>("\"winsock & whoami\"").is_err());
        assert_eq!(ResetKind::Winsock.args(), &["winsock", "reset"]);
        assert_eq!(ResetKind::Tcpip.args(), &["int", "ip", "reset"]);
    }
}
