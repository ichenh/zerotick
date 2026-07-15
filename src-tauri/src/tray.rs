//! 系统托盘状态指示 — 运行时着色图标 + Tooltip + 前端同步

use crate::events::TrayStatusEvent;
use crate::i18n;
use crate::settings;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use tauri::image::Image;
use tauri::menu::MenuItem;
use tauri::{AppHandle, Emitter};

const TRAY_ID: &str = "zerotick-tray";

static ALERT_GEN: AtomicU64 = AtomicU64::new(0);
static ICONS: OnceLock<TrayIcons> = OnceLock::new();
static TRAY_MENU: OnceLock<(MenuItem<tauri::Wry>, MenuItem<tauri::Wry>)> = OnceLock::new();

struct TrayIcons {
    normal: Image<'static>,
    warning: Image<'static>,
    critical: Image<'static>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayLevel {
    Normal,
    Warning,
    Critical,
}

impl TrayLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Warning => "warning",
            Self::Critical => "critical",
        }
    }
}

pub fn set_level(app: &AppHandle, level: TrayLevel, reason_id: impl Into<String>) {
    let reason_id = reason_id.into();
    let generation = ALERT_GEN.fetch_add(1, Ordering::SeqCst) + 1;

    apply_tray(app, level, &reason_id);
    emit_status(app, level, &reason_id);

    if level != TrayLevel::Normal {
        let app_clone = app.clone();
        let reason_id_owned = reason_id.clone();
        tauri::async_runtime::spawn(async move {
            let secs = settings::get().tray_recovery_secs;
            tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
            if ALERT_GEN.load(Ordering::SeqCst) == generation {
                apply_tray(&app_clone, TrayLevel::Normal, "normal");
                emit_status(&app_clone, TrayLevel::Normal, "normal");
                let _ = reason_id_owned;
            }
        });
    }
}

fn apply_tray(app: &AppHandle, level: TrayLevel, reason_id: &str) {
    let Some(tray) = app.tray_by_id(TRAY_ID) else {
        return;
    };

    let locale = settings::get().locale;
    let reason_text = i18n::tray_reason(&locale, reason_id);
    let tooltip = match level {
        TrayLevel::Normal => i18n::tray_tooltip_normal(&locale),
        TrayLevel::Warning => i18n::tray_tooltip_warning(&locale, &reason_text),
        TrayLevel::Critical => i18n::tray_tooltip_critical(&locale, &reason_text),
    };
    let _ = tray.set_tooltip(Some(tooltip.as_str()));

    let icons = tray_icons();
    let icon = match level {
        TrayLevel::Normal => &icons.normal,
        TrayLevel::Warning => &icons.warning,
        TrayLevel::Critical => &icons.critical,
    };
    let _ = tray.set_icon(Some(icon.clone()));
}

fn tray_icons() -> &'static TrayIcons {
    ICONS.get_or_init(|| {
        let base = Image::from_bytes(include_bytes!("../icons/icon.png"))
            .expect("icon.png")
            .to_owned();
        TrayIcons {
            normal: base.clone(),
            warning: tint_icon(&base, 1.0, 0.72, 0.18),
            critical: tint_icon(&base, 1.0, 0.28, 0.28),
        }
    })
}

/// 对 RGBA 图标通道加权着色（保留 alpha）
fn tint_icon(base: &Image<'static>, r: f32, g: f32, b: f32) -> Image<'static> {
    let mut rgba = base.rgba().to_vec();
    for px in rgba.chunks_exact_mut(4) {
        if px[3] == 0 {
            continue;
        }
        px[0] = (px[0] as f32 * r).clamp(0.0, 255.0) as u8;
        px[1] = (px[1] as f32 * g).clamp(0.0, 255.0) as u8;
        px[2] = (px[2] as f32 * b).clamp(0.0, 255.0) as u8;
    }
    Image::new_owned(rgba, base.width(), base.height())
}

/// 向前端同步当前托盘监控状态（启动时调用，不改变图标）
pub fn sync_frontend_status(app: &AppHandle) {
    emit_status(app, TrayLevel::Normal, "normal");
}

fn emit_status(app: &AppHandle, level: TrayLevel, reason_id: &str) {
    let _ = app.emit(
        "tray-status",
        &TrayStatusEvent {
            level: level.as_str().to_string(),
            reason_id: reason_id.to_string(),
        },
    );
}

pub fn register_tray_menu(show: MenuItem<tauri::Wry>, quit: MenuItem<tauri::Wry>) {
    let _ = TRAY_MENU.set((show, quit));
}

pub const TRAY_ICON_ID: &str = TRAY_ID;

/// 语言变更后更新托盘菜单与提示
pub fn refresh_locale(app: &AppHandle) {
    let locale = settings::get().locale;
    if let Some((show, quit)) = TRAY_MENU.get() {
        let _ = show.set_text(i18n::tray_show(&locale));
        let _ = quit.set_text(i18n::tray_quit(&locale));
    }
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(i18n::tray_tooltip_normal(&locale).as_str()));
    }
}
