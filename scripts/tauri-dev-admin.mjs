/**
 * 以管理员身份启动 `npm run tauri dev`（Windows UAC）
 * 用法：npm run tauri:dev:admin
 */
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

if (process.platform !== "win32") {
  console.error("tauri:dev:admin 仅支持 Windows。");
  process.exit(1);
}

function isElevated() {
  const r = spawnSync("net.exe", ["session"], { stdio: "ignore" });
  return r.status === 0;
}

function runDev() {
  const r = spawnSync("npm.cmd", ["run", "tauri", "dev"], {
    cwd: root,
    stdio: "inherit",
    env: { ...process.env, ZEROTICK_ELEVATED: "1" },
  });
  process.exit(r.status ?? 1);
}

if (isElevated()) {
  runDev();
} else {
  // 使用 EncodedCommand 传递提权后的命令，避免项目路径、中文和引号被
  // PowerShell -> cmd.exe -> npm 的多层解析破坏。
  const escapedRoot = root.replace(/'/g, "''");
  const elevatedScript = [
    "$ErrorActionPreference = 'Stop'",
    "$env:ZEROTICK_ELEVATED = '1'",
    `Set-Location -LiteralPath '${escapedRoot}'`,
    "& npm.cmd run tauri dev",
    "exit $LASTEXITCODE",
  ].join("; ");
  const encoded = Buffer.from(elevatedScript, "utf16le").toString("base64");
  const launcher = [
    "$childArgs = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-EncodedCommand', '" + encoded + "')",
    "Start-Process -FilePath 'powershell.exe' -ArgumentList $childArgs -Verb RunAs -Wait",
  ].join("; ");
  const r = spawnSync(
    "powershell.exe",
    ["-NoProfile", "-Command", launcher],
    { stdio: "inherit" },
  );
  process.exit(r.status ?? 1);
}
