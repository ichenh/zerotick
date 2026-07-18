# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.7] - 2026-07-19

### Added

- 为“设备与驱动”加入基于错误码的真实修复闭环：按证据提供设备启用、设备重启、从 Driver Store 重装现有驱动，以及用户确认后的官方 INF 驱动包安装。

### Changed

- 驱动操作完成后重新读取设备状态；只有 Windows 对目标设备报告设备管理器代码 0 时才显示为已修复，命令成功、硬件重扫或驱动包已加入系统均不会冒充最终修复结果。
- 音频常规枚举改为优先使用 Windows IMMDevice 原生接口，注册表与 PowerShell 仅作为兼容后备；同名的不同物理端点按端点 ID 保持独立。
- 全面体检不再复用已完成的缓存结果；并发请求只共享仍在采集实时证据的同一次扫描，完成后立即清除。

### Fixed

- 修复网络查询失败时被误报为“0 个活动适配器”“无 VPN”或“网关不可达”的问题；无法取得证据时本次诊断会明确失败。
- 收紧蓝牙设备移除/重连、端口进程结束和可移动存储占用解除的目标复核，避免 PID、设备 ID 或进程状态变化后操作错误对象。
- 修复长时间系统检查沿用短查询超时、PowerShell 输出管道可能阻塞、损坏设置导致启动失败，以及 CSV 导出可能触发电子表格公式的问题。

## [0.2.6] - 2026-07-18

### Changed

- 蓝牙诊断改为优先使用 SetupAPI、Configuration Manager 与 WinRT，分别验证设备登记、驱动问题码、服务状态和实时连接；PowerShell 仅保留为兼容后备。
- 可移动存储面板先通过 Win32 原生卷信息快速显示已连接设备，再在后台完成硬件拓扑、读卡器插槽、驱动、挂载与 BitLocker 状态检测；重复的完整扫描会共享同一任务。
- 蓝牙电量仅在 WinRT 实时确认设备已连接后通过 Uncached GATT 读取；未连接设备不再发起电量查询，缓存值也不会作为最终结果展示。
- 所有耗时在小于 1 秒时使用毫秒，达到 1 秒后使用秒并保留毫秒级小数精度。
- 将功能完整、诊断证据、Windows 原生能力优先、缓存诚实性和性能边界写入 Agent 开发指南。

### Fixed

- 修复概览蓝牙检测可能超时并显示“本项检测未完成”，而蓝牙面板单独检测正常的问题。
- 修复安装完成或手动启动后主窗口直接缩入托盘的问题；仅开机自启使用后台启动参数。
- 修复安全弹出读卡器全部卷后，短暂残留的设备节点被误报为驱动异常、已识别但挂载状态未知的问题。
- 修复蓝牙设备仅因出现在 Windows 已配对列表或设备节点未报错就被误判为已连接的问题。
- 修复可移动存储和蓝牙检测被串行 PowerShell 查询拖慢，以及重复点击产生多份昂贵扫描相互争抢的问题。
- 修复开发模式耗时显示始终使用冗长毫秒数，以及界面暴露翻译键或内部缓存状态文本的问题。

## [0.2.5] - 2026-07-17

### Added

- 内置全部 37 种受支持的界面语言，用户可在设置页即时切换，无需联网、下载语言包或重新安装。
- 首次启动会按照 Windows 首选语言顺序自动选择最接近的可用翻译；NSIS 安装界面也会匹配系统语言并提供语言选择器。
- 设置页和左上角版本号均可检查 GitHub Release 更新，并提供安全限定的更新下载、发布页、官网、Issue 与支持邮箱入口。
- 蓝牙设备电量同时读取 Windows 设备属性缓存和标准 GATT Battery Service，在设备支持时显示剩余电量。
- 概览历史支持最新优先或最早优先排序，排序选择会随设置保存。

### Changed

- GitHub Release 仅保留安装包、校验清单等面向用户的必要产物，不再发布应用运行时依赖的语言附件。
- 构建工具升级至 Vite 8 与 Oxc，Windows 管理查询升级至 WMI 0.18，并完成相应 API 迁移。
- 普通模式优先显示 Windows 的友好名称和总线报告产品名，不再把 `USB Composite Device`、VID/PID、实例路径等技术标识当作设备名称；原始标识仅在高级模式显示。
- “USB 存储”统一简化为“可移动存储”，不再在普通界面重复说明 USB、Type-C 或内部检测实现。
- 所有下拉选择器统一为语言选择器风格的悬浮面板，并补齐键盘操作、视口边缘翻转和多语言宽度适配。
- 统一页面右上角操作按钮尺寸、侧边栏顺序、设置页信息层级及高级模式换行布局。
- 所有耗时展示在小于 1 秒时使用精确毫秒值，达到 1 秒后使用秒并保留毫秒级精度。

### Fixed

- 移除依赖 GitHub、代理和用户网络环境的运行时语言包下载链路；语言选择器在后端初始化期间也可立即响应，避免首次启动显示英文或点击无反应。
- 修复概览声称倒序但实际按正序渲染、英文 `Disconnected` 与右侧内容重叠，以及高级模式和长文本语言下多处内容覆盖的问题。
- 修复音频默认设备、音量、静音和独占模式操作缺少后验验证的问题；现在仅在实际写入并复读一致后报告成功，并提供明确的权限与失败原因。
- 修复蓝牙重连命令可能忽略 PowerShell 非终止错误，以及 Windows 设置中可见电量但应用未读取的问题。
- 修复关机但仍连接的复合设备被模糊名称反复报告断开/重连的问题，并将短暂断连与普通重连按实际持续时间区分。
- 修复端口页把系统自动回收的 `TIME_WAIT` 连接误标为可解除的问题；终止可解除进程后会等待并确认进程实际退出。
- 修复官网按钮被外部链接白名单错误拦截的问题，同时继续拒绝非 ZeroTick 官方域名。
- 修复可移动存储普通视图可能暴露 VID/PID、重复展示检测说明，以及部分读卡器、无盘符卷和锁定卷状态不符合用户认知的问题。

## [0.2.4] - 2026-07-16

### Changed

- 全面体检改由单个 Rust 后端命令统一编排，限制昂贵系统查询的并发量，短时复用重复请求，并记录分项排队、执行与总耗时。
- 全面体检现在按诊断项完成顺序逐项推送并即时渲染，不再等待最慢项目结束后才一次性显示全部结果。
- 网络、音频、USB 与蓝牙诊断共享短时服务状态快照，将全面体检中的重复服务查询合并为一次，并复用稳定的 Windows 版本判断；修复后立即失效状态缓存以保证后验结果新鲜。
- 端口扫描改为首次进入端口页时按需执行，避免应用启动阶段无条件运行 `netstat` 与 `netsh`。
- Windows 保留端口范围使用短时缓存，重复刷新仍会实时获取监听连接与进程，仅避免反复启动不必要的 `netsh` 查询。
- 启动阶段并行读取版本与设置、窗口标题与历史记录，并将管理员状态检查延迟到首次进入修复页。
- 页面切换复用导航与页面节点，跳过重复的 class 切换，并在当前进程内复用稳定的管理员状态检查结果。
- 设置保存改为按差异触发副作用：仅在对应选项变化时同步开机自启、刷新托盘语言、窗口标题或历史列表，并跳过内容未变化的设置文件写入。
- 一键修复在保持组内服务依赖顺序的前提下，并行处理网络、音频、蓝牙、USB 服务组及 USB 电源配置检查。
- 将长期复用的语言与发布元数据校验脚本集中到 `scripts/checks/`，并清理不参与 Windows NSIS/MSI 构建的生成图标。
- CI 与发布门禁新增 Rust 格式检查、全 feature 严格 Clippy，并限制常规 CI 的 GitHub Token 为只读权限。

### Added

- 新增安全漏洞报告策略、结构化 Issue/PR 模板与 npm、Cargo、GitHub Actions 的 Dependabot 周期更新配置。

### Fixed

- 修复蓝牙后台轮询直接占用异步运行时线程，以及手动扫描与轮询在短时间内重复查询的问题。
- 为 WMI 查询增加后端超时与队列满保护，避免前端停止等待后系统查询仍无限堆积。

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

[Unreleased]: https://github.com/ichenh/zerotick/compare/v0.2.5...HEAD
[0.2.5]: https://github.com/ichenh/zerotick/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/ichenh/zerotick/compare/v0.2.2...v0.2.4
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
