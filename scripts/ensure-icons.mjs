/**
 * CI / 发布构建前确保 Tauri 图标存在。
 * 源图：仓库根目录 app-icon.png
 */
import { execSync } from "node:child_process";
import { existsSync, rmSync } from "node:fs";
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

// ZeroTick only ships NSIS/MSI packages. Keep the runtime and configured desktop
// bundle icons, but do not retain mobile or Microsoft Store generator outputs.
for (const generatedPath of [
  "android",
  "ios",
  "64x64.png",
  "Square107x107Logo.png",
  "Square142x142Logo.png",
  "Square150x150Logo.png",
  "Square284x284Logo.png",
  "Square30x30Logo.png",
  "Square310x310Logo.png",
  "Square44x44Logo.png",
  "Square71x71Logo.png",
  "Square89x89Logo.png",
  "StoreLogo.png",
]) {
  rmSync(path.join(root, "src-tauri", "icons", generatedPath), {
    recursive: true,
    force: true,
  });
}
