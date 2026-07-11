//! 开机自启 — 与 settings.launch_at_startup 同步

use crate::utils::logging;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

/// 启动后异步同步自启状态；失败仅记录警告
pub fn sync_on_startup(app: &AppHandle, enabled: bool) {
    if let Err(e) = sync(app, enabled) {
        logging::warn(format!("开机自启同步跳过: {e}"));
    }
}

/// 用户保存设置时同步；失败返回错误供前端提示
pub fn sync(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();

    if enabled {
        return manager.enable().map_err(|e| e.to_string());
    }

    match manager.is_enabled() {
        Ok(true) => match manager.disable() {
            Ok(()) => Ok(()),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("找不到") || msg.to_lowercase().contains("not found") {
                    logging::warn("检测到失效的开机自启项，已忽略");
                    Ok(())
                } else {
                    Err(msg)
                }
            }
        },
        Ok(false) => Ok(()),
        Err(e) => {
            logging::warn(format!("查询开机自启状态失败，视为未启用: {e}"));
            Ok(())
        }
    }
}
