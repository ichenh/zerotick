# ZeroTick — Agent Guide

> Cursor Agent 请优先阅读本文件。`.cursor/rules/` 为本地 IDE 配置，不入库。

## 项目简介

ZeroTick 是 Windows 上的轻量级系统故障诊断工具（Tauri 2）。核心能力：USB/蓝牙断连实时监控、蓝牙服务诊断、BSOD 追溯、一键修复、托盘驻留、原生通知、历史导出、端口管理、29 种界面语言。

## 版本策略

```
v0.1.4  ← 当前版本
v0.1.5  ← 下一版常规开发（patch）
…
v0.2.0  ← 仅用户明确确认后才升 minor
```

**规则**：后续开发默认只 bump patch（末位）。升到 `v0.x.0` 必须等用户明确说明。

## 发布流程（GitHub Actions）

推送到 `main` 后由 [`.github/workflows/release.yml`](.github/workflows/release.yml) 自动发版：

1. 读取 `package.json` 的 `version`（如 `0.1.4`）
2. 若远程已存在同名标签 `v0.1.4`，**跳过**（避免重复发布）
3. 否则：测试 → `npm run tauri build` → 上传 NSIS `.exe` 与 MSI 到 **GitHub Releases**，并创建标签 `v{version}`
4. Release 说明来自 `CHANGELOG.md` 中对应 `## [x.y.z]` 段落（`scripts/extract-changelog.mjs`）

**发版前人工检查清单**

- [ ] 功能完成 + 最小必要 diff
- [ ] `CHANGELOG.md` 已写入新版本条目（非空 `[Unreleased]` 需合并进版本段）
- [ ] 三处版本号一致：`package.json` / `src-tauri/Cargo.toml` / `src-tauri/tauri.conf.json`
- [ ] 仓库根目录已提交 `app-icon.png`（或已生成并提交 `src-tauri/icons/`）
- [ ] 本地：`npm run build`、`cargo test --manifest-path src-tauri/Cargo.toml`
- [ ] 合并/推送到 `main` 后观察 Actions → Release 工作流
- [ ] 不主动 commit / push（除非用户要求）

**手动触发**：GitHub → Actions → Release → Run workflow。

## CI

[`.github/workflows/ci.yml`](.github/workflows/ci.yml) 在 PR 与 `main` 推送时运行：前端构建 + `cargo test` + `cargo check`（不产出安装包）。

## 架构速查

| 层 | 技术 |
|----|------|
| 前端 | Vite + Vanilla JS + CSS + i18n |
| 后端 | Rust + Tauri 2 Commands/Events |
| 监控 | Win32 `WM_DEVICECHANGE` 消息泵 |
| 诊断 | WMI（蓝牙）、Minidump + Event Log（BSOD） |
| 修复 | Win32 Services + 注册表 USB 省电扫描 |
| 集成 | 托盘着色、Windows Toast、开机自启、dialog 导出 |

## 关键路径

```
index.html, src/main.js, src/i18n.js, src/locales/
src-tauri/src/
  monitor.rs, bluetooth.rs, bsod.rs, repair.rs, ports.rs
  settings.rs, i18n.rs, tray.rs, notify.rs, autostart.rs
  commands.rs, events.rs, engine.rs, lib.rs
.github/workflows/ci.yml, release.yml
scripts/ensure-icons.mjs, extract-changelog.mjs, prepare-dev.mjs
README.md, CHANGELOG.md, CONTRIBUTING.md, README.zh-CN.md, LICENSE
```

## 不要做的事

- 不要擅自升 minor（v0.2.0）或 major
- 不要擅自 git commit / push
- 不要添加用户未请求的重构、测试文件或 markdown
- 不要引入新框架替换 Vanilla JS 前端
- 不要恢复已删除的 CLI 入口（`src/main.rs`）
- 不要在未 bump 版本的情况下期望 GitHub 再次发版（同版本标签已存在会跳过）
