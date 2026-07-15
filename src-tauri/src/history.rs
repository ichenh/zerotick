//! 断连历史持久化 — 本地 JSON 存储

use crate::events::DeviceEvent;
use crate::settings;
use crate::utils::logging;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static STORE: OnceLock<Mutex<HistoryStore>> = OnceLock::new();

struct HistoryStore {
    path: PathBuf,
}

/// 初始化历史存储路径（通常在 app_data_dir 下）
pub fn init(path: PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建历史目录失败: {e}"))?;
    }
    let _ = STORE.set(Mutex::new(HistoryStore { path }));
    Ok(())
}

/// 追加一条设备事件并落盘
pub fn append(event: &DeviceEvent) {
    let Some(mutex) = STORE.get() else {
        return;
    };
    let Ok(mut store) = mutex.lock() else {
        return;
    };
    if let Err(e) = store.append_inner(event) {
        logging::error(format!("历史记录写入失败: {e}"));
    }
}

/// 读取全部历史（时间正序：旧 → 新）
pub fn list() -> Vec<DeviceEvent> {
    let Some(mutex) = STORE.get() else {
        return Vec::new();
    };
    let Ok(store) = mutex.lock() else {
        return Vec::new();
    };
    store.load_inner().unwrap_or_default()
}

/// 清空历史
pub fn clear() -> Result<(), String> {
    let Some(mutex) = STORE.get() else {
        return Ok(());
    };
    let Ok(store) = mutex.lock() else {
        return Err("历史存储锁失败".into());
    };
    store.save_inner(&[])?;
    Ok(())
}

/// 导出为 JSON 字符串
pub fn export_json() -> Result<String, String> {
    let items = list();
    serde_json::to_string_pretty(&items).map_err(|e| format!("导出 JSON 失败: {e}"))
}

/// 导出为 CSV 字符串
pub fn export_csv() -> Result<String, String> {
    let items = list();
    let mut out = String::from(
        "timestamp,event_type,category,friendly_name,vid_pid,disconnect_ms,device_path,message\n",
    );
    for e in &items {
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            csv_cell(&e.timestamp),
            csv_cell(&e.event_type),
            csv_cell(&e.category),
            csv_cell(e.friendly_name.as_deref().unwrap_or("")),
            csv_cell(e.vid_pid.as_deref().unwrap_or("")),
            csv_cell(&e.disconnect_ms.map(|v| v.to_string()).unwrap_or_default()),
            csv_cell(&e.device_path),
            csv_cell(&e.message),
        ));
    }
    Ok(out)
}

fn csv_cell(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

impl HistoryStore {
    fn load_inner(&self) -> Result<Vec<DeviceEvent>, String> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        let raw = fs::read_to_string(&self.path).map_err(|e| format!("读取历史失败: {e}"))?;
        if raw.trim().is_empty() {
            return Ok(Vec::new());
        }
        serde_json::from_str(&raw).map_err(|e| format!("解析历史 JSON 失败: {e}"))
    }

    fn append_inner(&mut self, event: &DeviceEvent) -> Result<(), String> {
        let mut entries = self.load_inner()?;
        entries.push(event.clone());
        let max = settings::get().max_history_entries;
        if entries.len() > max {
            let drain = entries.len() - max;
            entries.drain(0..drain);
        }
        self.save_inner(&entries)
    }

    fn save_inner(&self, entries: &[DeviceEvent]) -> Result<(), String> {
        let json =
            serde_json::to_string_pretty(entries).map_err(|e| format!("序列化历史失败: {e}"))?;
        fs::write(&self.path, json).map_err(|e| format!("写入历史失败: {e}"))
    }
}

#[cfg(test)]
mod export_tests {
    use super::csv_cell;

    #[test]
    fn csv_escapes_commas() {
        assert_eq!(csv_cell("a,b"), "\"a,b\"");
    }
}
