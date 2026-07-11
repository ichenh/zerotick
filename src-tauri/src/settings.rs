//! 用户设置持久化 — settings.json（app_data_dir）

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static STORE: OnceLock<Mutex<SettingsStore>> = OnceLock::new();

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    /// 瞬断判定阈值（毫秒）：断连到此时间内重连视为瞬断
    pub transient_threshold_ms: u64,
    /// 托盘告警态自动恢复时长（秒）
    pub tray_recovery_secs: u64,
    /// 历史 JSON 最大保留条数
    pub max_history_entries: usize,
    /// 前端 Timeline 最大显示条数
    pub timeline_display_max: usize,
    /// 主窗口隐藏时发送 Windows 原生 Toast
    pub native_notifications: bool,
    /// 登录 Windows 时自动启动
    pub launch_at_startup: bool,
    /// 蓝牙 WMI 轮询间隔（秒），空闲时降低 CPU 占用
    pub bluetooth_poll_secs: u64,
    /// 界面语言（BCP 47，如 zh-CN、en）
    pub locale: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            transient_threshold_ms: 500,
            tray_recovery_secs: 45,
            max_history_entries: 500,
            timeline_display_max: 80,
            native_notifications: true,
            launch_at_startup: false,
            bluetooth_poll_secs: 60,
            locale: "zh-CN".into(),
        }
    }
}

impl AppSettings {
    pub fn validate(&self) -> Result<(), String> {
        if !(100..=10_000).contains(&self.transient_threshold_ms) {
            return Err("瞬断阈值须在 100–10000 ms 之间".into());
        }
        if !(5..=600).contains(&self.tray_recovery_secs) {
            return Err("托盘恢复时长须在 5–600 秒之间".into());
        }
        if !(50..=2000).contains(&self.max_history_entries) {
            return Err("历史条数须在 50–2000 之间".into());
        }
        if !(10..=500).contains(&self.timeline_display_max) {
            return Err("Timeline 显示条数须在 10–500 之间".into());
        }
        if !(15..=300).contains(&self.bluetooth_poll_secs) {
            return Err("蓝牙轮询间隔须在 15–300 秒之间".into());
        }
        if !crate::i18n::is_supported(&self.locale) {
            return Err(format!("不支持的语言: {}", self.locale));
        }
        Ok(())
    }
}

struct SettingsStore {
    path: PathBuf,
    current: AppSettings,
}

pub fn init(path: PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建设置目录失败: {e}"))?;
    }

    let current = if path.exists() {
        load_from_file(&path)?
    } else {
        AppSettings::default()
    };
    current.validate()?;

    let _ = STORE.set(Mutex::new(SettingsStore { path, current }));
    Ok(())
}

pub fn get() -> AppSettings {
    STORE
        .get()
        .and_then(|m| m.lock().ok())
        .map(|s| s.current.clone())
        .unwrap_or_default()
}

pub fn save(mut settings: AppSettings) -> Result<AppSettings, String> {
    settings.locale = crate::i18n::normalize_locale(&settings.locale);
    settings.validate()?;
    let mutex = STORE.get().ok_or("设置存储未初始化")?;
    let mut store = mutex.lock().map_err(|_| "设置存储锁失败")?;
    store.current = settings;
    persist(&store.path, &store.current)?;
    Ok(store.current.clone())
}

fn load_from_file(path: &PathBuf) -> Result<AppSettings, String> {
    let raw = fs::read_to_string(path).map_err(|e| format!("读取设置失败: {e}"))?;
    if raw.trim().is_empty() {
        return Ok(AppSettings::default());
    }
    serde_json::from_str(&raw).map_err(|e| format!("解析设置 JSON 失败: {e}"))
}

fn persist(path: &PathBuf, settings: &AppSettings) -> Result<(), String> {
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("序列化设置失败: {e}"))?;
    fs::write(path, json).map_err(|e| format!("写入设置失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_valid() {
        AppSettings::default().validate().unwrap();
    }

    #[test]
    fn rejects_invalid_threshold() {
        let mut s = AppSettings::default();
        s.transient_threshold_ms = 50;
        assert!(s.validate().is_err());
    }
}
