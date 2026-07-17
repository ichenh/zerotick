# ZeroTick — Agent Guide

> Cursor Agent 请优先阅读本文件。`.cursor/rules/` 为本地 IDE 配置，不入库。

## 项目简介

ZeroTick 是面向普通用户的 Windows 设备与系统故障诊断工具（Tauri 2）。核心能力：网络、音频、可移动存储、蓝牙、设备驱动、BSOD 与端口诊断；安全的一键修复；托盘监控、原生通知、历史导出；普通/高级模式分层；37 种随应用内置并经过完整校验的界面语言。

## 产品与诊断原则（最高优先级）

ZeroTick 首先是 Windows 常规功能的聚合工具，以及故障排查和安全解决工具；它不是 PowerShell 脚本的图形界面，也不是只追求跑分的扫描器。

优先级固定如下，后续功能不得颠倒：

1. **功能完整与结果可信**：覆盖用户需要的常规功能，不因提速而省略检测项、隐藏失败或降低证据质量。
2. **故障定位与安全解决**：区分问题发生在硬件、设备节点、驱动、服务、实时连接、卷/文件系统还是应用占用，并提供与证据匹配的修复操作。
3. **诚实的用户反馈**：缓存、阶段性结果、未知状态与最终实测结果必须明确区分，不能把“Windows 已登记”“驱动未报错”或缓存值表述为设备实际可用。
4. **性能与体验**：在不损害前三项的前提下缩短关键路径、并行独立查询并渐进更新界面。

### 诊断证据规则

- Windows 自身也可能处于错误或陈旧状态；读取 Windows 设置中的列表、缓存、注册表或单一 API 结果，不等于完成诊断。
- 同一结论应尽可能交叉验证不同证据层。例如蓝牙应分别判断：适配器/设备节点是否存在、设备管理器问题码、`bthserv` 状态、WinRT 实时连接状态，以及 GATT 实时电量。
- “设备节点存在且问题码为 0”只表示 Windows 当前登记该设备且节点未报告驱动错误，**不能推导为已连接**。
- “已配对”“已识别”“已挂载”“可读写”“驱动正常”是不同状态，不得互相替代。
- 无法取得实时证据时显示“未知”“正在读取”或明确失败原因；不要用乐观默认值填充。
- 概览与面板必须使用一致的事实来源。概览可以先返回基础检测，但后续必须完成深度检测并无感更新，不能永久停留在简化结果。

### Windows 原生能力优先

正常检测路径按以下顺序选择实现：

1. Win32、SetupAPI、Configuration Manager、COM、WinRT、IP Helper、Event Log 等 Windows 原生 API。
2. 必要时使用 Rust 中的 WMI/COM 查询补充原生 API 难以取得的信息。
3. PowerShell 只允许作为兼容后备，或用于用户明确触发且没有等价稳定 API 的系统操作。

具体约束：

- 不得把 PowerShell 放在常规面板首屏、概览扫描或高频监控的必经路径上。
- 不得仅把 PowerShell 移到后台就宣称“已原生化”；后台化只是交互优化。
- 新增 PowerShell 调用前，必须先确认没有合适的原生 API，并在代码中说明为何只能后备使用。
- PowerShell 后备失败不能抹掉原生路径已经获得的有效证据；不同来源的错误应保留在诊断日志中。
- 显式修复、格式化、SFC/CHKDSK 等长时间系统操作可以调用系统命令，但必须有超时、权限、风险提示和可解释结果。

### 缓存、渐进更新与性能

- 缓存只能用于快速首屏或减少重复系统查询，不能冒充本次实时检测结果。
- UI 不直接显示内部状态码、翻译键或实现术语（例如 `common.loading`、`CACHE`、`unknown_state`）。
- 需要实时结果时，先显示自然语言的“正在读取…”，成功后直接替换为最终值；失败时明确显示不可用或未知。
- 渐进检测必须继续执行完整检测项；不能为了快速而取消驱动、插槽、加密、挂载、连接或硬件异常判断。
- 相同的昂贵查询应 single-flight/共享结果，避免概览、面板和重复点击同时启动多份 WMI、Storage 或 PowerShell 扫描。
- 重任务继续通过 `run_blocking(...)` 或专用后台任务离开 UI 线程；互不依赖的查询可以并行，但避免无界并发压垮 Windows Provider。
- 性能优化先用分阶段日志确认瓶颈，再替换关键路径；不要以删除功能作为优化手段。

### 面向普通用户的界面

- 普通模式使用用户可理解的结论和操作，不暴露设备实例 ID、内部枚举值、脚本名称或诊断实现细节。
- 高级模式可以展示证据和原始状态，但仍不得把内部代码或翻译键当作用户文案。
- 任何“正常”“已连接”“可安全弹出”“修复成功”等结论都必须有相应证据，不能根据缺少错误而推断成功。
- 危险操作必须保持明确范围、确认提示和失败解释；不得为了自动化而扩大修复范围。

## 版本策略

```
v0.2.5  ← 当前版本
v0.2.6  ← 下一版常规开发（patch）
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
scripts/checks/check-locales.mjs, scripts/checks/check-release.mjs
scripts/ensure-icons.mjs
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
