# Contributing

感谢你对 ZeroTick 的关注。

## 开始之前

- 请先阅读 [README.md](README.md)（[中文版](README.zh-CN.md)）了解项目定位与构建方式
- 版本与 Agent 约定见 [AGENTS.md](AGENTS.md)
- 变更记录请写入 [CHANGELOG.md](CHANGELOG.md)（[Keep a Changelog](https://keepachangelog.com/) 格式）

## 开发环境

- Windows 10 / 11（x64）
- Rust 1.82+
- Node.js 18+

```powershell
git clone https://github.com/ichenh/zerotick.git
cd zerotick
npm install
npm run tauri dev
```

提交前建议执行：

```powershell
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```

## Release

维护者发版步骤：

1. 在 `CHANGELOG.md` 填写新版本段落（`## [x.y.z] - YYYY-MM-DD`）
2. 同步 bump `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 中的版本号
3. 确认已提交 `app-icon.png` 或 `src-tauri/icons/`
4. 合并并 **推送到 `main`**

推送后 [Release 工作流](.github/workflows/release.yml) 会：

- 读取 `package.json` 版本（如 `0.1.5` → 标签 `v0.1.5`）
- 若该标签已存在则跳过（同版本不会重复发布）
- 否则运行测试、`npm run tauri build`，将 NSIS `.exe` 与 MSI 上传到 GitHub Releases

也可在 GitHub Actions 页面手动 **Run workflow** 触发发布（仍受「标签已存在则跳过」规则约束）。

## Pull Request

1. Fork 本仓库并基于 `main` 创建分支
2. 保持 diff 聚焦，遵循现有代码风格
3. 在 CHANGELOG 的 `[Unreleased]` 下简要说明变更
4. 如涉及用户可见行为，更新 README 或相关文案

## 报告问题

在 [Issues](https://github.com/ichenh/zerotick/issues) 中请尽量包含：

- Windows 版本
- ZeroTick 版本
- 复现步骤
- 相关日志（`%APPDATA%\com.zerotick.desktop\zerotick_debug.log`）

## 许可证

贡献的代码以 [MIT](LICENSE) 许可证发布。
