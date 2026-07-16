import { readFileSync } from "node:fs";

const packageJson = JSON.parse(readFileSync("package.json", "utf8"));
const packageLock = JSON.parse(readFileSync("package-lock.json", "utf8"));
const tauriConfig = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
const cargoToml = readFileSync("src-tauri/Cargo.toml", "utf8");
const cargoLock = readFileSync("src-tauri/Cargo.lock", "utf8").replaceAll("\r\n", "\n");
const changelog = readFileSync("CHANGELOG.md", "utf8");

const cargoVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
const cargoLockVersion = cargoLock.match(/\[\[package\]\]\nname = "zerotick"\nversion = "([^"]+)"/)?.[1];
const versions = {
  "package.json": packageJson.version,
  "package-lock.json": packageLock.version,
  "package-lock root package": packageLock.packages?.[""]?.version,
  "src-tauri/Cargo.toml": cargoVersion,
  "src-tauri/Cargo.lock": cargoLockVersion,
  "src-tauri/tauri.conf.json": tauriConfig.version,
};
const expected = packageJson.version;
const mismatches = Object.entries(versions).filter(([, version]) => version !== expected);
if (mismatches.length) {
  console.error(`Version mismatch; expected ${expected}:`);
  for (const [file, version] of mismatches) console.error(`- ${file}: ${version ?? "missing"}`);
  process.exit(1);
}
if (!changelog.includes(`## [${expected}]`)) {
  console.error(`CHANGELOG.md is missing a ## [${expected}] release section.`);
  process.exit(1);
}
console.log(`Validated release metadata for ${expected}.`);
