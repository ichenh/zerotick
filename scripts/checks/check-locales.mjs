import { bundles, LOCALE_OPTIONS } from "../../src/i18n.js";
import { ALL_LANGUAGE_OPTIONS } from "../../src/locales/catalog.js";
import fs from "node:fs";

function flatten(value, prefix = "", output = {}) {
  for (const [key, child] of Object.entries(value)) {
    const path = prefix ? `${prefix}.${key}` : key;
    if (child && typeof child === "object" && !Array.isArray(child)) {
      flatten(child, path, output);
    } else {
      output[path] = child;
    }
  }
  return output;
}

function placeholders(value) {
  return [...String(value).matchAll(/\{\w+\}/g)].map((match) => match[0]).sort();
}

const english = flatten(bundles.en);
const expectedKeys = Object.keys(english).sort();
const failures = [];

const backendSource = fs.readFileSync(new URL("../../src-tauri/src/i18n.rs", import.meta.url), "utf8");
const supportedBlock = backendSource.match(/pub const SUPPORTED:[\s\S]*?=\s*&\[([\s\S]*?)\];/);
const backendLocales = supportedBlock
  ? [...supportedBlock[1].matchAll(/"([^"]+)"/g)].map((match) => match[1]).sort()
  : [];
const catalogLocales = ALL_LANGUAGE_OPTIONS.map(({ code }) => code).sort();
if (JSON.stringify(backendLocales) !== JSON.stringify(catalogLocales)) {
  failures.push("frontend language catalog and backend SUPPORTED list differ");
}

const trayBundles = JSON.parse(
  fs.readFileSync(new URL("../../src-tauri/locales/tray.json", import.meta.url), "utf8"),
);
const trayEnglish = trayBundles.en ?? {};
const trayKeys = Object.keys(trayEnglish).sort();
for (const locale of ALL_LANGUAGE_OPTIONS) {
  const source = locale.source ?? locale.base ?? locale.code;
  const tray = trayBundles[source];
  if (!tray) {
    failures.push(`${locale.code}: missing backend tray source ${source}`);
    continue;
  }
  const missing = trayKeys.filter((key) => !(key in tray));
  if (missing.length) failures.push(`${source}: ${missing.length} missing backend tray keys`);
  for (const key of trayKeys) {
    if (!(key in tray)) continue;
    if (JSON.stringify(placeholders(trayEnglish[key])) !== JSON.stringify(placeholders(tray[key]))) {
      failures.push(`${source}:${key}: backend tray placeholder mismatch`);
    }
  }
}

for (const { code } of LOCALE_OPTIONS) {
  const bundle = bundles[code];
  if (!bundle) {
    failures.push(`${code}: locale option has no bundle`);
    continue;
  }
  const flat = flatten(bundle);
  const missing = expectedKeys.filter((key) => !(key in flat));
  if (missing.length) failures.push(`${code}: ${missing.length} missing keys (${missing.slice(0, 8).join(", ")})`);
  for (const key of expectedKeys) {
    if (!(key in flat)) continue;
    const sourceParams = placeholders(english[key]);
    const localeParams = placeholders(flat[key]);
    if (JSON.stringify(sourceParams) !== JSON.stringify(localeParams)) {
      failures.push(`${code}:${key}: placeholder mismatch`);
    }
  }
}

if (failures.length) {
  console.error(failures.join("\n"));
  process.exit(1);
}

console.log(`Validated ${LOCALE_OPTIONS.length} complete locale bundles with ${expectedKeys.length} keys each.`);
