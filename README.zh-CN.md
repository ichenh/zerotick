<p align="center">
  <img src="app-icon.png" width="96" alt="ZeroTick" />
</p>

<h1 align="center">ZeroTick</h1>

<p align="center">
  <strong>轻量级 Windows 系统故障诊断与实时追踪工具</strong><br />
  托盘静默驻留 · 毫秒级 USB/蓝牙断连捕捉 · 蓝牙诊断 · BSOD 追溯 · 一键修复
</p>

<p align="center">
  <a href="README.md">English</a> · <b>简体中文</b>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT" /></a>
  <img src="https://img.shields.io/badge/platform-Windows%2010%2F11-0078D4?logo=windows&logoColor=white" alt="Windows 10/11" />
  <img src="https://img.shields.io/badge/Tauri-2-FFC131?logo=tauri&logoColor=black" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/version-v0.1.4-22c55e" alt="v0.1.4" />
  <a href="https://github.com/ichenh/zerotick/actions/workflows/ci.yml"><img src="https://github.com/ichenh/zerotick/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://github.com/ichenh/zerotick/actions/workflows/release.yml"><img src="https://github.com/ichenh/zerotick/actions/workflows/release.yml/badge.svg" alt="Release" /></a>
</p>

<p align="center">
  <a href="#功能">功能</a> ·
  <a href="#安装">安装</a> ·
  <a href="#使用">使用</a> ·
  <a href="#开发">开发</a> ·
  <a href="CHANGELOG.md">更新日志</a> ·
  <a href="CONTRIBUTING.md">参与贡献</a>
</p>

---

## 功能

| 模块 | 说明 |
|------|------|
| **硬件监控** | `WM_DEVICECHANGE` 实时捕获 USB / 蓝牙断连与瞬断（阈值可配置） |
| **蓝牙诊断** | WMI 轮询 `bthserv` 与 PnP 设备，异常时托盘告警 |
| **BSOD 追溯** | Minidump 扫描 + 事件日志 BugCheck 解析 |
| **一键修复** | 重启 `bthserv` / `Audiosrv`，扫描 USB 选择性暂停 |
| **托盘驻留** | 关闭窗口隐藏至托盘，图标随状态变色（正常 / 警告 / 告警） |
| **系统通知** | 窗口隐藏时推送 Windows 原生 Toast |
| **历史记录** | 事件持久化，支持 JSON / CSV 导出 |
| **端口管理** | 扫描本地占用，识别开发残留与系统保留段，支持一键解除 |
| **多语言** | 29 种界面语言，含托盘菜单本地化 |

## 系统要求

- Windows 10 / 11（x64）
- [WebView2 运行时](https://developer.microsoft.com/microsoft-edge/webview2/)
- **一键修复**建议以管理员身份运行（右键 → 以管理员身份运行）

## 安装

### 从 Release 安装（推荐）

在 [Releases](https://github.com/ichenh/zerotick/releases) 页面下载最新安装包：

- `ZeroTick_*_x64-setup.exe`（NSIS 安装程序）
- `ZeroTick_*_x64_en-US.msi`（MSI）

安装后应用默认驻留系统托盘。

> 推送至 `main` 且 `package.json` 版本为新版本时，GitHub Actions 会自动创建 `v*` 标签并上传安装包到 Releases。详见 [CONTRIBUTING.md](CONTRIBUTING.md#release)。

### 从源码构建

**依赖：** [Rust](https://rustup.rs/) 1.82+ · [Node.js](https://nodejs.org/) 18+

```powershell
git clone https://github.com/ichenh/zerotick.git
cd zerotick
npm install
npm run tauri build
```

构建产物位于：

```
src-tauri\target\release\bundle\nsis\
src-tauri\target\release\bundle\msi\
```

> 首次构建前若缺少应用图标，可使用根目录 `app-icon.png` 生成：
> `npm run ensure-icons` 或 `npm run tauri icon app-icon.png`
>
> **发布到 GitHub 前请提交 `app-icon.png` 或已生成的 `src-tauri/icons/`**，否则 CI 无法构建安装包。

## 使用

1. **启动** — 应用驻留系统托盘，主窗口默认隐藏
2. **打开界面** — 左键托盘图标，或右键 →「打开仪表盘」
3. **概览** — Timeline 实时显示 USB / 蓝牙事件；瞬断会高亮并提醒
4. **诊断** — 检测蓝牙服务、扫描 BSOD、执行一键修复
5. **端口** — 扫描本地占用，解除 node / vite 等开发残留
6. **设置** — 调整监控阈值、历史条数、界面语言、通知与登录时启动
7. **退出** — 右键托盘 →「退出 ZeroTick」

## 数据目录

用户数据保存在 `%APPDATA%\com.zerotick.desktop\`：

| 文件 | 用途 |
|------|------|
| `settings.json` | 用户设置 |
| `device_history.json` | 设备事件历史 |
| `zerotick_debug.log` | 调试日志 |

## 性能

空闲时目标约 **0% CPU**：

- 硬件监控为 Win32 消息泵（事件驱动，无轮询）
- 蓝牙 WMI 默认 60 秒轮询，且仅在状态**变更**时推送 UI / 通知
- BSOD 仅在启动时扫描一次

可在设置中调整蓝牙轮询间隔（15–300 秒）。

## 开发

```powershell
npm install
npm run tauri dev    # 开发模式（自动执行 prepare-dev）
npm run build        # 仅构建前端
cargo test --manifest-path src-tauri/Cargo.toml
```

开发服务器端口：`55555`（见 `vite.config.js`、`tauri.conf.json`、`src-tauri/src/ports.rs`）。

### 项目结构

```
zerotick/
├── index.html
├── src/                    # 前端（Vite + Vanilla JS）
│   ├── main.js
│   ├── i18n.js
│   ├── locales/
│   └── styles.css
├── src-tauri/              # Rust 后端
│   ├── src/
│   │   ├── monitor.rs
│   │   ├── bluetooth.rs
│   │   ├── bsod.rs
│   │   ├── repair.rs
│   │   ├── ports.rs
│   │   ├── settings.rs
│   │   ├── i18n.rs
│   │   └── tray.rs
│   └── locales/tray.json
├── scripts/
├── CHANGELOG.md
├── CONTRIBUTING.md
└── LICENSE
```

Agent / 自动化开发约定见 [AGENTS.md](AGENTS.md)。

## 许可证

[MIT](LICENSE) © ZeroTick Contributors
