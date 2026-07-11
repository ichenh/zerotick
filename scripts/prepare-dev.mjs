/**
 * tauri dev 启动前清理：结束调试版 zerotick.exe + 释放 55555 端口
 */
import { execSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const PORT = 55555;
const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const DEBUG_EXE = path.join(ROOT, "src-tauri", "target", "debug", "zerotick.exe").toLowerCase();

function run(cmd) {
  try {
    return execSync(cmd, { encoding: "utf8", stdio: ["pipe", "pipe", "pipe"] }).trim();
  } catch {
    return "";
  }
}

function killPid(pid, reason) {
  try {
    execSync(`taskkill /F /PID ${pid}`, { stdio: "ignore" });
    console.log(`[prepare-dev] ${reason} (PID ${pid})`);
  } catch {
    /* 忽略 */
  }
}

if (process.platform !== "win32") {
  process.exit(0);
}

// 结束本项目的调试版 zerotick（避免 exe 被锁定）
const wmic = run(`wmic process where "name='zerotick.exe'" get ProcessId,ExecutablePath /FORMAT:CSV`);
for (const line of wmic.split(/\r?\n/)) {
  if (!line.includes("zerotick.exe")) continue;
  const parts = line.split(",");
  const exe = (parts[1] || "").trim().toLowerCase();
  const pid = (parts[2] || "").trim();
  if (!pid || !/^\d+$/.test(pid)) continue;
  if (exe.includes("target\\debug\\zerotick.exe") || exe === DEBUG_EXE) {
    killPid(pid, "已结束调试版 zerotick.exe");
  }
}

// 释放开发端口上的 node 残留
const lines = run(`netstat -ano | findstr :${PORT}`).split(/\r?\n/);
const pids = new Set();
for (const line of lines) {
  if (!line.includes("LISTENING")) continue;
  const parts = line.trim().split(/\s+/);
  const pid = parts[parts.length - 1];
  if (pid && /^\d+$/.test(pid) && pid !== "0") pids.add(pid);
}
for (const pid of pids) {
  const name = run(`tasklist /FI "PID eq ${pid}" /FO CSV /NH`);
  if (!name.toLowerCase().includes("node.exe")) continue;
  killPid(pid, `已结束占用端口 ${PORT} 的 node 进程`);
}
