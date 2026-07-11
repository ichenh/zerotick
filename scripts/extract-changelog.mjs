/**
 * 从 CHANGELOG.md 提取指定版本的发布说明（供 GitHub Release body 使用）。
 * Usage: node scripts/extract-changelog.mjs 0.1.4
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const version = process.argv[2];
if (!version) {
  console.error("Usage: node scripts/extract-changelog.mjs <version>");
  process.exit(1);
}

const changelogPath = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "CHANGELOG.md",
);
const md = fs.readFileSync(changelogPath, "utf8");
const escaped = version.replace(/\./g, "\\.");
const re = new RegExp(`## \\[${escaped}\\][\\s\\S]*?(?=\\n## \\[|\\n## \\[Unreleased\\]|$)`);
const match = md.match(re);

if (match) {
  process.stdout.write(match[0].trim());
} else {
  process.stdout.write(`## ZeroTick v${version}\n\nSee [CHANGELOG.md](CHANGELOG.md).`);
}
