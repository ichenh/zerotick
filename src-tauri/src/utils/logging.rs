//! 双通道日志：stderr 高亮 + zerotick_debug.log 文件追加。

use chrono::Local;
use owo_colors::OwoColorize;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static LOG_FILE: OnceLock<Mutex<File>> = OnceLock::new();

#[derive(Debug, Clone, Copy)]
enum LogLevel {
    Info,
    Warn,
    Error,
    Critical,
}

impl LogLevel {
    fn tag(self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Critical => "CRIT",
        }
    }
}

pub fn init(log_path: PathBuf) -> io::Result<PathBuf> {
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;
    let _ = LOG_FILE.set(Mutex::new(file));
    enable_vt_mode();
    info(format!("ZeroTick 日志系统已初始化 → {}", log_path.display()));
    Ok(log_path)
}

fn enable_vt_mode() {
    use windows::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
        STD_OUTPUT_HANDLE,
    };
    unsafe {
        if let Ok(handle) = GetStdHandle(STD_OUTPUT_HANDLE) {
            let mut mode = windows::Win32::System::Console::CONSOLE_MODE(0);
            if GetConsoleMode(handle, &mut mode).is_ok() {
                let _ = SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
    }
}

fn write_log(level: LogLevel, message: &str) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let plain = format!("[{timestamp}] [{}] {message}", level.tag());
    let colored = match level {
        LogLevel::Info => plain.cyan().to_string(),
        LogLevel::Warn => plain.yellow().to_string(),
        LogLevel::Error => plain.red().to_string(),
        LogLevel::Critical => plain.on_red().white().bold().to_string(),
    };
    let _ = writeln!(io::stderr(), "{colored}");
    if let Some(mutex) = LOG_FILE.get() {
        if let Ok(mut file) = mutex.lock() {
            let _ = writeln!(file, "{plain}");
            let _ = file.flush();
        }
    }
}

pub fn info(message: impl AsRef<str>) {
    write_log(LogLevel::Info, message.as_ref());
}
pub fn warn(message: impl AsRef<str>) {
    write_log(LogLevel::Warn, message.as_ref());
}
pub fn error(message: impl AsRef<str>) {
    write_log(LogLevel::Error, message.as_ref());
}
pub fn critical(message: impl AsRef<str>) {
    write_log(LogLevel::Critical, message.as_ref());
}
