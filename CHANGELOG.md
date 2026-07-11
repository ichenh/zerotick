# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- GitHub Actions：CI（测试与构建检查）+ Release（推送 `main` 自动打标签并发布 NSIS/MSI 安装包）

## [0.1.4] - 2026-07-11

### Added

- **国际化**：29 种界面语言；设置项「界面语言」；托盘菜单随语言切换
- **UI 重构**：Windows 11 Fluent 风格导航（概览 / 诊断 / 端口 / 设置）
- 开发脚本 `scripts/prepare-dev.mjs`（释放开发端口、结束调试进程）

### Changed

- 设置页分组与文案按正式版规范整理（常规 / 监控 / 历史记录）
- 通知与自启动开关与数字输入项统一为左右分栏布局
- 开发服务器端口改为 `55555`（与 `vite.config.js`、`ports.rs` 同步）

### Fixed

- 端口页扫描报错 `Cannot set properties of null`（移除已废弃的 `#dev-port-label` 引用）

### Removed

- 弃用的 CLI 入口 `src/main.rs`（项目已完全迁移至 Tauri 桌面版）

## [0.1.3] - 2026-07-11

### Fixed

- 双托盘图标：`tauri-plugin-single-instance` 禁止多实例；移除 `tauri.conf.json` 与代码重复创建托盘
- WMI 诊断失败 `0x80010106`：持久 WMI 工作线程 + `COMLibrary::without_security()` 单例初始化

## [0.1.1] - 2026-07-11

### Added

- **端口占用管理**：扫描本地 TCP/UDP 监听、Windows TCP 保留段、分类与一键解除
- Commands：`scan_ports` / `release_port` / `release_releasable_ports`

### Changed

- 开发服务器端口固定为 `14280`（后于 v0.1.4 调整为 `55555`）

## [0.1.0] - 2026-07-11

首个里程碑发布。

### Added

- `get_app_version`、设置项 `bluetooth_poll_secs`、`LICENSE`（MIT）
- 安装包元数据（NSIS / MSI）

### Changed

- 蓝牙 WMI 仅在健康状态**变更**时推送事件与通知
- 日志迁移至 `%APPDATA%\com.zerotick.desktop\zerotick_debug.log`

### Fixed

- 蓝牙正常时不再每轮询周期写 INFO 日志

## [0.0.9] - 2026-07-11

### Added

- Windows 原生 Toast、开机自启、历史导出 JSON/CSV
- 设置项：`native_notifications` / `launch_at_startup`

## [0.0.8] - 2026-07-11

### Added

- `settings.json` 持久化与行为设置面板
- Commands：`get_settings` / `save_settings`

## [0.0.7] - 2026-07-11

### Added

- 诊断面板结构化卡片、`repair-complete` 事件、`is_elevated` 权限检测

## [0.0.6] - 2026-07-11

### Added

- SetupAPI 设备名解析、历史持久化、托盘图标运行时着色

## [0.0.5] - 2026-07-11

### Added

- 友好设备名、托盘三态、`tray-status` 事件、Timeline 动画

## [0.0.4] - 2026-07-11

后端 Event 推送引擎接线。

## [0.0.3] - 2026-07-11

Tauri 2 桌面架构迁移。

## [0.0.2] - 2026-07-11

CLI 版核心诊断模块。

## [0.0.1] - 2026-07-11

项目初始化。

[Unreleased]: https://github.com/ichenh/zerotick/compare/v0.1.4...HEAD
[0.1.4]: https://github.com/ichenh/zerotick/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/ichenh/zerotick/compare/v0.1.1...v0.1.3
[0.1.1]: https://github.com/ichenh/zerotick/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ichenh/zerotick/compare/v0.0.9...v0.1.0
[0.0.9]: https://github.com/ichenh/zerotick/compare/v0.0.8...v0.0.9
[0.0.8]: https://github.com/ichenh/zerotick/compare/v0.0.7...v0.0.8
[0.0.7]: https://github.com/ichenh/zerotick/compare/v0.0.6...v0.0.7
[0.0.6]: https://github.com/ichenh/zerotick/compare/v0.0.5...v0.0.6
[0.0.5]: https://github.com/ichenh/zerotick/compare/v0.0.4...v0.0.5
[0.0.4]: https://github.com/ichenh/zerotick/compare/v0.0.3...v0.0.4
[0.0.3]: https://github.com/ichenh/zerotick/compare/v0.0.2...v0.0.3
[0.0.2]: https://github.com/ichenh/zerotick/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/ichenh/zerotick/releases/tag/v0.0.1
