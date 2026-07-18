<p align="center">
  <img src="app-icon.png" width="96" alt="ZeroTick 应用图标" />
</p>

<h1 align="center">ZeroTick</h1>

<p align="center">
  <strong>让 Windows 日常故障变得看得懂、能处理，并在安全时一键修复。</strong><br />
  将网络、音频、可移动存储、蓝牙、驱动、蓝屏和端口问题集中到一个地方。
</p>

<p align="center">
  <a href="README.md">English</a> · <b>简体中文</b>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT 许可证" /></a>
  <img src="https://img.shields.io/badge/platform-Windows%2010%2F11-0078D4?logo=windows&logoColor=white" alt="Windows 10 和 11" />
  <img src="https://img.shields.io/badge/Tauri-2-FFC131?logo=tauri&logoColor=black" alt="Tauri 2" />
  <a href="https://github.com/ichenh/zerotick/releases/latest"><img src="https://img.shields.io/github/v/release/ichenh/zerotick?display_name=tag&sort=semver" alt="最新版本" /></a>
  <a href="https://github.com/ichenh/zerotick/actions/workflows/ci.yml"><img src="https://github.com/ichenh/zerotick/actions/workflows/ci.yml/badge.svg" alt="CI 状态" /></a>
  <a href="https://github.com/ichenh/zerotick/actions/workflows/release.yml"><img src="https://github.com/ichenh/zerotick/actions/workflows/release.yml/badge.svg" alt="发布状态" /></a>
</p>

<p align="center">
  <a href="#为什么需要-zerotick">初衷</a> ·
  <a href="#它能做什么">功能</a> ·
  <a href="#安装">安装</a> ·
  <a href="#快速开始">快速开始</a> ·
  <a href="#从源码构建">开发</a> ·
  <a href="CHANGELOG.md">更新日志</a>
</p>

---

## 为什么需要 ZeroTick

Windows 的很多问题来得毫无征兆：Wi-Fi 或蓝牙模块突然消失、声音莫名失效、没有打开任何文件却无法弹出移动硬盘、加密硬盘尚未解锁就看起来像是损坏，或蓝屏后只留下一串普通人看不懂的代码。相关开关和证据又分散在“设置”、设备管理器、磁盘管理、事件查看器、系统服务、注册表和命令行工具中。

**ZeroTick 的初衷，就是填补这段体验断层。** 它把 Windows 上常用设备和故障处理能力集中在一个容易理解的软件里，让普通用户不用先学习系统原理，也能知道发生了什么、有什么影响、接下来应该怎么做。能够安全自动恢复时，ZeroTick 直接提供操作；暂时不能修复时，也会给出清晰的排查路径，而不是只抛出错误码。

对于熟悉系统的用户，高级模式会在同一套流程中补充设备实例 ID、PID、服务状态、原始错误、蓝屏证据和扫描参数，不牺牲底层可诊断性。

## 它能做什么

| 板块 | 检测与管理能力 |
|------|----------------|
| **全面体检** | 并行运行主要诊断，为较慢的系统查询设置超时，并汇总真正需要处理的问题 |
| **网络** | 活动网卡、默认网关、DNS、关键服务、VPN/代理配置、本地代理所属程序、网速测试、DNS 刷新和引导式修复 |
| **音频** | 输出/输入设备、默认设备、音量、静音、共享/独占模式、Windows 音频服务和常见权限问题 |
| **可移动存储** | 按物理设备聚合卷，区分未解锁介质、读卡器空插槽和无法读取的存储卡，检测可能的占用程序，请求安全关闭，弹出单个卷或整个设备，并提供有风险确认的快速/完全格式化 |
| **蓝牙** | 适配器消失、驱动和支持服务状态、已配对外设、重连、修复，以及带重新配对说明的设备移除确认 |
| **设备与驱动** | 常见设备管理器故障、已安装驱动证据、硬件重扫、设备启用/重启、经验证的 Driver Store 驱动重装，以及带确认的官方 INF 驱动包安装 |
| **蓝屏追溯** | Minidump 与 BugCheck 历史、可用时的 WinDbg 证据、可能原因归纳，以及可执行的后续处理或修复建议 |
| **端口** | 本机监听端口和连接、所属程序、Windows 保留端口段，以及对已识别开发残留的谨慎解除 |
| **监控与历史** | 事件驱动的 USB/蓝牙断连监控、瞬断识别、托盘状态、原生通知、本地历史和 JSON/CSV 导出 |

### 同时照顾普通用户和高级用户

- **普通模式**优先展示状态、影响、安全操作和容易理解的下一步建议。
- **高级模式**补充底层证据，但不会用日志和错误码取代用户说明。
- 破坏性操作必须明确确认：完全格式化和蓝牙设备移除会说明后果；USB 弹出则遵循 Windows 最终的安全判断。
- 驱动操作只有在 ZeroTick 重新读取设备且 Windows 报告设备管理器代码 0 后才会显示为已修复；命令执行成功本身不作为修复证据。
- 只有修改受保护的 Windows 服务、设备或设置时才需要管理员权限；普通只读诊断无需长期保持管理员模式。

## 安装

### 下载正式版本

前往 [GitHub Releases](https://github.com/ichenh/zerotick/releases/latest) 下载最新安装包：

- `ZeroTick_*_x64-setup.exe`：推荐普通用户下载；安装时可选择“仅为当前用户”或“为这台电脑的所有用户”
- `ZeroTick_*_x64_en-US.msi`：用于 IT 管理和集中部署，普通用户无需下载

> GitHub Release 中的安装包目前尚未进行代码签名。多数系统可以正常安装，但 Windows 是否显示“未知发布者”或 Microsoft Defender SmartScreen 提示，仍取决于本机策略和文件信誉。请只从本仓库的 Releases 页面下载 ZeroTick。每个版本都会附带 `SHA256SUMS.txt`；如需核对文件完整性，可运行 `Get-FileHash .\ZeroTick_*.exe -Algorithm SHA256` 并比较结果。高级用户还可运行 `gh attestation verify .\ZeroTick_*.exe --repo ichenh/zerotick` 验证 GitHub 构建来源。

ZeroTick 支持 Windows 10/11 x64，并使用 Microsoft Edge WebView2。大多数受支持的 Windows 系统已经包含 WebView2；缺失时安装程序可以引导安装。

> 部分修复操作需要管理员权限。ZeroTick 会在提权前说明原因，普通检测不要求长期以管理员身份运行。

## 快速开始

1. 安装并启动 ZeroTick，应用会进入 Windows 通知区域。
2. 点击托盘图标并选择“打开仪表盘”。
3. 使用“全面体检”检查整体状态，或直接进入出现问题的板块。
4. 阅读状态和建议步骤，仅在修复项与当前问题匹配时执行“修复”。
5. 需要设备 ID、原始错误或扫描参数时，在设置中打开“高级模式”。

ZeroTick 内置全部 37 种受支持的界面语言。首次启动时，软件会按照 Windows 首选语言顺序自动选择最接近的可用翻译；此后可在设置页即时切换任意语言，无需下载或重新安装。构建门禁会拒绝词条不完整的翻译。

## 安全与隐私

- 格式化、调整分区或处理疑似故障硬盘前，请先备份重要数据。
- ZeroTick 不会绕过 Windows 的设备移除否决。如果仍有句柄未关闭，软件会展示能够获得的占用证据和后续建议。
- “一键修复”是有边界的恢复尝试，不能保证修复硬件损坏、数据损坏或厂商特有的驱动缺陷。
- 软件不需要账号；设置、设备历史和调试日志保存在 `%APPDATA%\com.zerotick.desktop\`。
- 大多数检测完全在本机完成。可选网速测试会从 Cloudflare 下载约 1 MiB 的测试数据；高级蓝屏分析可能从微软公共符号服务器获取符号。

## 本地数据

| 文件 | 用途 |
|------|------|
| `settings.json` | 应用偏好与扫描设置 |
| `device_history.json` | 本机 USB/蓝牙事件历史 |
| `zerotick_debug.log` | 用于排查问题的诊断日志 |

## 从源码构建

项目采用 Windows 原生开发环境。请按照 [Tauri 环境要求](https://v2.tauri.app/zh-cn/start/prerequisites/) 准备：

- Windows 10/11 x64
- Microsoft C++ Build Tools，并勾选“使用 C++ 的桌面开发”
- Microsoft Edge WebView2
- Rust 1.85 或更高版本
- Node.js 22 LTS 或更高版本

```powershell
git clone https://github.com/ichenh/zerotick.git
cd zerotick
npm ci
npm run check:release
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm run tauri build
```

安装包输出到：

```text
src-tauri\target\release\bundle\nsis\
src-tauri\target\release\bundle\msi\
```

本地开发：

```powershell
npm run tauri dev
# 测试需要管理员权限的流程时：
npm run tauri:dev:admin
```

### 架构概览

| 层 | 实现 |
|----|------|
| 界面 | Vite、Vanilla JavaScript、CSS、37 种内置且独立校验的界面语言 |
| 桌面集成 | Tauri 2、托盘、通知、开机启动和原生对话框 |
| 诊断 | Rust 调度 Win32 API、WMI、PowerShell 后备查询、事件日志和 Minidump/WinDbg 分析 |
| 监控 | 事件驱动的 `WM_DEVICECHANGE` 消息泵与有明确超时的后台检测 |
| 安全 | 后端参数校验、受保护进程规则、权限检查、超时和破坏性操作确认 |

主要实现位于 [`src/`](src/) 和 [`src-tauri/src/`](src-tauri/src/)。参与开发和发布请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)，版本记录见 [CHANGELOG.md](CHANGELOG.md)，自动化约定见 [AGENTS.md](AGENTS.md)。

## 参与贡献与反馈

- 可在 [GitHub Issues](https://github.com/ichenh/zerotick/issues) 报告可复现问题或提出功能建议。
- 请提供 Windows 版本、ZeroTick 版本、复现步骤和相关高级信息或日志；公开提交前请移除个人路径和设备标识。
- 欢迎提交贡献，发起 Pull Request 前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可证

[MIT](LICENSE) © ZeroTick Contributors
