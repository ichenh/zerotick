/**
 * CI / 发布构建前确保 Tauri 图标存在。
 * 源图：仓库根目录 app-icon.png
 */
import { execSync } from "node:child_process";
import { existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const iconPng = path.join(root, "src-tauri", "icons", "icon.png");
const source = path.join(root, "app-icon.png");

if (existsSync(iconPng)) {
  process.exit(0);
}

if (!existsSync(source)) {
  console.error(
    "Missing src-tauri/icons/icon.png and app-icon.png.\n" +
      "Add app-icon.png (1024×1024 PNG recommended) or run:\n" +
      "  npm run tauri icon app-icon.png",
  );
  process.exit(1);
}

console.log("[ensure-icons] Generating Tauri icons from app-icon.png …");
execSync("npm run tauri icon app-icon.png", { cwd: root, stdio: "inherit" });
