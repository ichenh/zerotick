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

/// 蓝牙诊断结果 — 通过 `bluetooth-status` 通道推送
#[derive(Debug, Clone, Serialize)]
pub struct BluetoothStatusEvent {
    pub timestamp: String,
    pub healthy: bool,
    pub bthserv_state: Option<String>,
    pub issues: Vec<BluetoothIssue>,
    pub radio_count: usize,
}

/// BSOD 扫描结果 — 通过 `bsod-alert` 通道推送
#[derive(Debug, Clone, Serialize)]
pub struct BsodAlertEvent {
    pub timestamp: String,
    pub is_recent: bool,
    pub dump_path: String,
    pub bugcheck_code: Option<String>,
    pub faulting_driver: Option<String>,
    pub message: Option<String>,
}

/// 一键修复完成 — 通过 `repair-complete` 通道推送
#[derive(Debug, Clone, Serialize)]
pub struct RepairCompleteEvent {
    pub timestamp: String,
    pub success: bool,
    pub needs_admin: bool,
    pub services_restarted: Vec<String>,
    pub service_errors: Vec<String>,
    pub usb_power_warnings: Vec<String>,
    pub power_scan_error: Option<String>,
    /// 前端翻译键：`ok_clean` | `ok_usb_warnings` | `needs_admin` | `usb_scan_error` | `service_errors`
    pub summary_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_count: Option<usize>,
}
