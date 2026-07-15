import { bundles, LOCALE_OPTIONS } from "../src/i18n.js";

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
