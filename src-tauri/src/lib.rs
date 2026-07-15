//! ZeroTick Tauri 后端入口

mod audio;
mod autostart;
mod bluetooth;
mod bsod;
mod commands;
mod devices;
mod engine;
mod events;
mod history;
mod i18n;
mod monitor;
mod network;
mod notify;
mod ports;
mod repair;
mod services;
mod settings;
mod tray;
mod usb_storage;
mod utils;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = run_inner() {
        eprintln!("ZeroTick 致命错误: {e}");
        std::process::exit(1);
    }
}

fn run_inner() -> Result<(), Box<dyn std::error::Error>> {
    let builder = tauri::Builder::default();

    // 开发模式允许多开，避免与已安装版抢单实例导致 dev 秒退
    #[cfg(not(debug_assertions))]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
        show_main_window(app);
    }));

    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("无法获取 app_data_dir: {e}"))?;

            utils::logging::init(data_dir.join("zerotick_debug.log"))
                .map_err(|e| format!("日志初始化失败: {e}"))?;
            utils::logging::info(format!(
                "ZeroTick 启动 pid={} v={}",
                std::process::id(),
                env!("CARGO_PKG_VERSION")
            ));
            settings::init(data_dir.join("settings.json"))
                .map_err(|e| format!("设置加载失败: {e}"))?;
            bsod::init_seen_store(data_dir.join("bsod_seen.json"));

            // 正式版：若设置要求管理员启动且当前未提升，则 UAC 重启后退出本进程
            #[cfg(not(debug_assertions))]
            if settings::get().run_as_admin && !utils::elevated::is_elevated() {
                match utils::elevated::relaunch_as_admin() {
                    Ok(()) => std::process::exit(0),
                    Err(e) => utils::logging::warn(format!("管理员提升跳过: {e}")),
                }
            }

            history::init(data_dir.join("device_history.json"))
                .map_err(|e| format!("历史存储初始化失败: {e}"))?;

            setup_tray(app).map_err(|e| format!("托盘初始化失败: {e}"))?;
            engine::start(app.handle()).map_err(|e| format!("诊断引擎启动失败: {e}"))?;
            tray::sync_frontend_status(app.handle());

            // 开发模式直接显示主窗口；正式版默认驻留托盘
            #[cfg(debug_assertions)]
            show_main_window(app.handle());

            // 自启同步延后执行，避免注册表指向旧 exe 时阻断启动
            let handle = app.handle().clone();
            let launch = settings::get().launch_at_startup;
            tauri::async_runtime::spawn(async move {
                autostart::sync_on_startup(&handle, launch);
            });

            utils::logging::info("ZeroTick Tauri 后端初始化完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::check_bluetooth,
            commands::bluetooth_remove_device,
            commands::bluetooth_reconnect_device,
            commands::repair_bluetooth,
            commands::diagnose_network,
            commands::network_speed_test,
            commands::network_flush_dns,
            commands::repair_network,
            commands::diagnose_audio,
            commands::set_default_audio_device,
            commands::set_audio_mode,
            commands::set_audio_volume,
            commands::set_audio_mute,
            commands::repair_audio,
            commands::diagnose_usb,
            commands::usb_list_drives,
            commands::usb_locking_processes,
            commands::usb_close_process,
            commands::usb_open_volume,
            commands::usb_eject,
            commands::usb_format_volume,
            commands::repair_usb,
            commands::diagnose_devices,
            commands::rescan_devices,
            commands::scan_bsod,
            commands::apply_bsod_repairs,
            commands::run_repair,
            commands::get_device_history,
            commands::clear_device_history,
            commands::is_elevated,
            commands::restart_elevated,
            commands::get_settings,
            commands::save_settings,
            commands::export_device_history,
            commands::get_app_version,
            commands::scan_ports,
            commands::release_port,
            commands::release_connection,
            commands::release_releasable_ports,
        ])
        .build(tauri::generate_context!())
        .map_err(|e| format!("构建 Tauri 应用失败: {e}"))?
        .run(|_app_handle, _event| {});

    Ok(())
}

fn setup_tray(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    if app.tray_by_id(tray::TRAY_ICON_ID).is_some() {
        utils::logging::warn("托盘已存在，跳过重复创建");
        return Ok(());
    }

    let locale = settings::get().locale;
    let show_i = MenuItem::with_id(app, "show", i18n::tray_show(&locale), true, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", i18n::tray_quit(&locale), true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

    let icon = app
        .default_window_icon()
        .ok_or("缺少应用图标 — 请运行 npm run tauri icon")?
        .clone();

    TrayIconBuilder::with_id(tray::TRAY_ICON_ID)
        .icon(icon)
        .menu(&menu)
        .tooltip(i18n::tray_tooltip_normal(&locale))
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_main_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(tray.app_handle());
            }
        })
        .build(app)?;

    tray::register_tray_menu(show_i.clone(), quit_i.clone());

    // 主窗口关闭：release 下按设置驻留托盘或完全退出；dev 始终退出以便热重载
    #[cfg(not(debug_assertions))]
    if let Some(window) = app.get_webview_window("main") {
        let window_clone = window.clone();
        window.on_window_event(move |event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if settings::get().close_to_tray {
                    let _ = window_clone.hide();
                    api.prevent_close();
                }
            }
        });
    }

    Ok(())
}

fn show_main_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}
