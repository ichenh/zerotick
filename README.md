<p align="center">
  <img src="app-icon.png" width="96" alt="ZeroTick" />
</p>

<h1 align="center">ZeroTick</h1>

<p align="center">
  <strong>Lightweight Windows diagnostics for USB/Bluetooth disconnects, BSOD tracing, and one-click repair</strong><br />
  System tray · millisecond device events · Bluetooth health · crash dumps · automated fixes
</p>

<p align="center">
  <b>English</b> · <a href="README.zh-CN.md">简体中文</a>
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
  <a href="#features">Features</a> ·
  <a href="#installation">Installation</a> ·
  <a href="#usage">Usage</a> ·
  <a href="#development">Development</a> ·
  <a href="CHANGELOG.md">Changelog</a> ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

---

## Features

| Module | Description |
|--------|-------------|
| **Hardware monitoring** | Real-time USB / Bluetooth disconnect and transient events via `WM_DEVICECHANGE` (configurable threshold) |
| **Bluetooth diagnostics** | WMI polling of `bthserv` and PnP devices; tray alerts on failure |
| **BSOD tracing** | Minidump scan plus Event Log BugCheck analysis |
| **One-click repair** | Restarts `bthserv` / `Audiosrv` and scans USB selective suspend settings |
| **System tray** | Close to tray; icon reflects status (normal / warning / alert) |
| **Notifications** | Native Windows toasts when the window is hidden |
| **History** | Persistent event log with JSON / CSV export |
| **Port manager** | Scan local listeners, flag dev leftovers and reserved ranges, release safe processes |
| **Localization** | 29 display languages, including tray menu strings |

## Requirements

- Windows 10 / 11 (x64)
- [WebView2 Runtime](https://developer.microsoft.com/microsoft-edge/webview2/)
- **One-click repair** works best when run as administrator (right-click → Run as administrator)

## Installation

### From Releases (recommended)

Download the latest build from [Releases](https://github.com/ichenh/zerotick/releases):

- `ZeroTick_*_x64-setup.exe` — NSIS installer
- `ZeroTick_*_x64_en-US.msi` — MSI package

After installation, ZeroTick stays in the system tray by default.

> Pushing to `main` with a new `package.json` version triggers GitHub Actions to tag `v*` and upload installers. See [Contributing → Release](CONTRIBUTING.md#release).

### Build from source

**Prerequisites:** [Rust](https://rustup.rs/) 1.82+ · [Node.js](https://nodejs.org/) 18+

```powershell
git clone https://github.com/ichenh/zerotick.git
cd zerotick
npm install
npm run tauri build
```

Installers are written to:

```
src-tauri\target\release\bundle\nsis\
src-tauri\target\release\bundle\msi\
```

> If icons are missing, generate them from `app-icon.png`:
> `npm run ensure-icons` or `npm run tauri icon app-icon.png`
>
> **Commit `app-icon.png` or `src-tauri/icons/` before publishing** — CI requires them to build installers.

## Usage

1. **Launch** — ZeroTick runs in the system tray; the main window starts hidden
2. **Open the app** — Left-click the tray icon, or right-click → *Open Dashboard*
3. **Overview** — Live USB / Bluetooth timeline; transient disconnects are highlighted
4. **Diagnostics** — Check Bluetooth, scan for BSOD dumps, run one-click repair
5. **Ports** — Scan local usage and release node / vite dev leftovers
6. **Settings** — Thresholds, history limits, display language, notifications, start at sign-in
7. **Quit** — Right-click tray → *Quit ZeroTick*

## Data directory

User data is stored under `%APPDATA%\com.zerotick.desktop\`:

| File | Purpose |
|------|---------|
| `settings.json` | User preferences |
| `device_history.json` | Device event history |
| `zerotick_debug.log` | Debug log |

## Performance

Target idle usage is about **0% CPU**:

- Hardware monitoring uses a Win32 message pump (event-driven, no polling loop)
- Bluetooth WMI polls every 60s by default and only notifies on **state changes**
- BSOD analysis runs once at startup

Adjust the Bluetooth poll interval in Settings (15–300 seconds).

## Development

```powershell
npm install
npm run tauri dev    # dev mode (runs prepare-dev automatically)
npm run build        # frontend only
cargo test --manifest-path src-tauri/Cargo.toml
```

Dev server port: `55555` (see `vite.config.js`, `tauri.conf.json`, `src-tauri/src/ports.rs`).

### Project layout

```
zerotick/
├── index.html
├── src/                    # Vite + Vanilla JS frontend
│   ├── main.js
│   ├── i18n.js
│   ├── locales/
│   └── styles.css
├── src-tauri/              # Rust / Tauri backend
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

Agent conventions: [AGENTS.md](AGENTS.md)

## License

[MIT](LICENSE) © ZeroTick Contributors
