# ZeroTick — Agent Guide

> Cursor Agent 请优先阅读本文件。`.cursor/rules/` 为本地 IDE 配置，不入库。

## 项目简介

ZeroTick 是面向普通用户的 Windows 设备与系统故障诊断工具（Tauri 2）。核心能力：网络、音频、USB 存储、蓝牙、设备驱动、BSOD 与端口诊断；安全的一键修复；托盘监控、原生通知、历史导出；普通/高级模式分层；6 种经过完整校验的界面语言。

## 版本策略

```
v0.2.1  ← 当前版本
v0.2.2  ← 下一版常规开发（patch）
…
v0.3.0  ← 仅用户明确确认后才升 minor
```

**规则**：后续开发默认只 bump patch（末位）。升到 `v0.x.0` 必须等用户明确说明。

## 发布流程（GitHub Actions）

推送到 `main` 后由 [`.github/workflows/release.yml`](.github/workflows/release.yml) 自动发版：

1. 读取 `package.json` 的 `version`（如 `0.2.1`）
2. 若远程已存在同名标签 `v0.2.1`，**跳过**（避免重复发布）
3. 否则：测试 → `npm run tauri build` → 生成 SHA-256 清单与 GitHub 构建来源证明 → 上传 NSIS `.exe`、MSI 和 `SHA256SUMS.txt` 到 **GitHub Releases**，并创建标签 `v{version}`
4. Release 说明来自 `CHANGELOG.md` 中对应 `## [x.y.z]` 段落（`scripts/extract-changelog.mjs`）

**发版前人工检查清单**

- [ ] 功能完成 + 最小必要 diff
- [ ] `CHANGELOG.md` 已写入新版本条目（非空 `[Unreleased]` 需合并进版本段）
- [ ] `npm run check:release` 通过（版本号、锁文件与 CHANGELOG 一致）
- [ ] 仓库根目录已提交 `app-icon.png`（或已生成并提交 `src-tauri/icons/`）
- [ ] 本地：`npm run build`、`cargo test --manifest-path src-tauri/Cargo.toml`、严格 Clippy
- [ ] 合并/推送到 `main` 后观察 Actions → Release 工作流
- [ ] 不主动 commit / push（除非用户要求）

**手动触发**：GitHub → Actions → Release → Run workflow。

## CI

[`.github/workflows/ci.yml`](.github/workflows/ci.yml) 在 PR 与 `main` 推送时运行：版本与语言门禁、前端构建、`cargo test`、`cargo check` 和严格 Clippy（不产出安装包）。

## 架构速查

| 层 | 技术 |
|----|------|
| 前端 | Vite + Vanilla JS + CSS + i18n |
| 后端 | Rust + Tauri 2 Commands/Events |
| 监控 | Win32 `WM_DEVICECHANGE` 消息泵 |
| 诊断 | Win32、WMI、PowerShell 后备查询、Minidump + Event Log / WinDbg |
| 修复 | Win32 Services、设备管理、DNS、USB 省电与安全存储操作 |
| 集成 | 托盘着色、Windows Toast、开机自启、dialog 导出 |

## 关键路径

```
index.html, src/main.js, src/i18n.js, src/locales/
src-tauri/src/
  monitor.rs, network.rs, audio.rs, usb_storage.rs, bluetooth.rs
  devices.rs, bsod.rs, repair.rs, ports.rs, services.rs
  settings.rs, i18n.rs, tray.rs, notify.rs, autostart.rs
  commands.rs, events.rs, engine.rs, lib.rs
.github/workflows/ci.yml, release.yml
scripts/ensure-icons.mjs, check-locales.mjs, check-release.mjs
scripts/extract-changelog.mjs, prepare-dev.mjs, tauri-dev-admin.mjs
README.md, CHANGELOG.md, CONTRIBUTING.md, README.zh-CN.md, LICENSE
```

## 不要做的事

- 不要擅自升 minor（如 v0.3.0）或 major
- 不要擅自 git commit / push
- 不要添加用户未请求的重构、测试文件或 markdown
- 不要引入新框架替换 Vanilla JS 前端
- 不要恢复已删除的 CLI 入口（`src/main.rs`）
- 不要在未 bump 版本的情况下期望 GitHub 再次发版（同版本标签已存在会跳过）
