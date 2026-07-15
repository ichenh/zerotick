//! 执行 PowerShell 并解析 JSON 输出

use crate::utils::process::CommandExt;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

fn execute_with_timeout(wrapped: &str, timeout: Duration) -> Result<Output, String> {
    let mut child = Command::new("powershell")
        .hide_window()
        .args(["-NoProfile", "-NonInteractive", "-Command", wrapped])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("PowerShell 启动失败: {e}"))?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .map_err(|e| format!("PowerShell 输出读取失败: {e}"))
            }
            Ok(None) if started.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "PowerShell 系统查询超过 {} 秒，已安全停止；可在高级设置中调整系统查询超时",
                    timeout.as_secs()
                ));
            }
            Err(e) => return Err(format!("PowerShell 状态读取失败: {e}")),
        }
    }
}

fn execute(wrapped: &str) -> Result<Output, String> {
    execute_with_timeout(
        wrapped,
        Duration::from_secs(crate::settings::get().system_query_timeout_secs),
    )
}

#[cfg(windows)]
fn decode_os_text(bytes: &[u8]) -> String {
    if bytes.is_empty() {
        return String::new();
    }
    if let Ok(s) = std::str::from_utf8(bytes) {
        return s.to_string();
    }
    use windows::Win32::Globalization::{MultiByteToWideChar, CP_ACP};
    unsafe {
        let len = MultiByteToWideChar(CP_ACP, Default::default(), bytes, None);
        if len == 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }
        let mut wide = vec![0u16; len as usize];
        let written = MultiByteToWideChar(CP_ACP, Default::default(), bytes, Some(&mut wide));
        if written == 0 {
            return String::from_utf8_lossy(bytes).into_owned();
        }
        String::from_utf16_lossy(&wide)
    }
}

#[cfg(not(windows))]
fn decode_os_text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn wrap_json_script(script: &str) -> String {
    // 脚本块保证多行语句的输出进入管道，避免 EmptyPipeElement
    format!(
        "[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false); \
         $OutputEncoding = [Console]::OutputEncoding; \
         $ErrorActionPreference='Stop'; \
         & {{ {script} }} | ConvertTo-Json -Compress -Depth 8"
    )
}

pub fn run_json(script: &str) -> Result<serde_json::Value, String> {
    let wrapped = wrap_json_script(script);
    let output = execute(&wrapped)?;
    if !output.status.success() {
        let stderr = decode_os_text(&output.stderr);
        let stdout = decode_os_text(&output.stdout);
        let detail = if stderr.trim().is_empty() {
            stdout.trim().to_string()
        } else {
            format!("{stderr}{stdout}")
        };
        return Err(if detail.is_empty() {
            "PowerShell 执行失败".into()
        } else {
            detail
        });
    }
    let text = decode_os_text(&output.stdout).trim().to_string();
    if text.is_empty() {
        return Ok(serde_json::Value::Null);
    }
    serde_json::from_str(&text).map_err(|e| format!("JSON 解析失败: {e} — {text}"))
}

pub fn run_void(script: &str) -> Result<(), String> {
    run_void_with_timeout(
        script,
        Duration::from_secs(crate::settings::get().system_query_timeout_secs),
    )
}

pub fn run_void_with_timeout(script: &str, timeout: Duration) -> Result<(), String> {
    let wrapped = format!(
        "[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false); \
         $ErrorActionPreference='Stop'; {script}"
    );
    let output = execute_with_timeout(&wrapped, timeout)?;
    if !output.status.success() {
        let stderr = decode_os_text(&output.stderr);
        return Err(stderr.trim().to_string());
    }
    Ok(())
}
