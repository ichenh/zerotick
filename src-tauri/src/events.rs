//! Tauri 与前端通信的事件载荷定义

use serde::{Deserialize, Serialize};

/// 硬件断连/接入事件 — 通过 `device-event` 通道推送到前端
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceEvent {
    /// ISO 8601 本地时间戳
    pub timestamp: String,
    /// `arrival` | `remove` | `transient_reconnect`
    pub event_type: String,
    /// 设备分类标签，如 "USB外设" / "蓝牙设备/驱动"
    pub category: String,
    pub vid_pid: Option<String>,
    pub device_path: String,
    /// 断连到重连的毫秒数（仅 transient / arrival-after-remove 时有值）
    pub disconnect_ms: Option<u64>,
    pub message: String,
    /// 设备友好名称（注册表 DeviceDesc / FriendlyName）
    pub friendly_name: Option<String>,
}

/// 托盘 / 引擎状态 — 同步前端 status pill
#[derive(Debug, Clone, Serialize)]
pub struct TrayStatusEvent {
    pub level: String,
    /// 前端翻译键，如 `transient_hw`、`device_remove`
    pub reason_id: String,
}

/// 蓝牙诊断问题项 — `id` 供前端 i18n
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothIssue {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<u32>,
}

/// 蓝牙已配对/连接设备
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDeviceEntry {
    pub name: String,
    pub instance_id: String,
    pub status: String,
    /// Live WinRT connection evidence. `None` means the device is registered but
    /// its current connection has not yet been verified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub connected: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battery_percent: Option<u8>,
    /// `refreshing` | `cached` | `live` | `unavailable`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battery_state: Option<String>,
}

/// 蓝牙诊断结果 — 通过 `bluetooth-status` 通道推送
#[derive(Debug, Clone, Serialize)]
pub struct BluetoothStatusEvent {
    pub timestamp: String,
    pub healthy: bool,
    pub bthserv_state: Option<String>,
    pub issues: Vec<BluetoothIssue>,
    pub adapter_count: usize,
    pub adapters: Vec<String>,
    pub devices: Vec<BluetoothDeviceEntry>,
}

/// BSOD 扫描结果 — 通过 `bsod-alert` 通道推送
#[derive(Debug, Clone, Serialize)]
pub struct BsodFixAction {
    pub id: String,
    pub automatic: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BsodAlertEvent {
    pub timestamp: String,
    pub is_recent: bool,
    pub dump_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dump_time: Option<String>,
    pub bugcheck_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_name: Option<String>,
    pub analysis_id: String,
    /// `root_cause` only when debugger module evidence exists; otherwise `error_type`.
    pub analysis_kind: String,
    pub faulting_driver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub faulting_module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debugger: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_cause: Option<String>,
    pub fixes: Vec<BsodFixAction>,
    pub message: Option<String>,
}

/// 系统服务状态项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    pub name: String,
    pub label_id: String,
    pub state: Option<String>,
    pub start_mode: Option<String>,
    /// The service is intentionally idle and will be started by Windows when needed.
    #[serde(default)]
    pub expected_stopped: bool,
}

/// 系统服务诊断问题项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceIssue {
    pub id: String,
    pub service_name: String,
    pub label_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}
