# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2026-07-15

### Fixed

- 修复各诊断面板、全面体检及相关系统操作在运行后台命令时反复弹出终端窗口的问题。

## [0.2.1] - 2026-07-15

### Added

- GitHub Release 现在会随安装包发布 `SHA256SUMS.txt`，便于核对下载文件的完整性。
- 正式发布产物现在会生成 GitHub Sigstore 构建来源证明，可追溯到对应仓库、工作流和提交。

### Documentation

- 中英文下载说明现已明确标注安装包尚未代码签名，并提供 Windows SHA-256 校验方法。

## [0.2.0] - 2026-07-15

### Added

- 新增网络诊断、网速测试、DNS 刷新、VPN/代理识别与代理程序溯源。
- 新增音频输入/输出管理、默认设备切换、音量与静音控制、独占模式和音频服务修复。
- 新增 USB 存储设备聚合、加密/未解锁状态、读卡器空插槽识别、程序占用检测、整体安全弹出及快速/完全格式化。
- 新增设备与驱动面板、硬件改动扫描和常见设备管理器问题说明。
- 新增蓝牙设备重连与安全移除流程，以及蓝牙服务和驱动诊断。
- 新增蓝屏原因归纳、WinDbg/事件日志证据和可执行修复建议。
- 新增普通模式与高级模式分层，并开放各诊断模块的扫描参数。
- 新增 GitHub Actions CI 与 NSIS/MSI 自动发布流程。

### Changed

- 全面重构 Windows 原生风格界面、面板信息层级、按钮布局、应用图标与行业术语。
- 将扫描流程拆分为并行原生查询并增加超时控制，减少 PowerShell 串行等待。
- 将错误码和日志化输出改为普通用户可理解的原因、影响和处理步骤；高级模式保留原始证据。
- 语言选择仅展示已经具备完整词条并通过占位符校验的语言。

### Fixed

- 修复“全面体检”无响应以及部分面板重复扫描、扫描状态不清晰的问题。
- 修复音频独占策略权限错误缺少清晰提示的问题。
- 修复带锁移动硬盘、虚拟光驱和多卡读卡器被错误拆分，以及整体弹出结果重复或误报失败的问题。
- 修复可访问卷被误判为离线、容量文案容易误导及程序占用来源不明确的问题。
- 修复代理存在时无法显示相关程序、设置页顺序不合理和多处文案未本地化的问题。

### Security

- 为格式化和蓝牙设备移除增加明确的破坏性操作确认。
- 收紧 Tauri 前端权限并启用内容安全策略。

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

[Unreleased]: https://github.com/ichenh/zerotick/compare/v0.2.2...HEAD
[0.2.2]: https://github.com/ichenh/zerotick/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/ichenh/zerotick/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ichenh/zerotick/compare/v0.1.4...v0.2.0
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
