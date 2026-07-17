import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import en from "../src/locales/en.js";
import zhCN from "../src/locales/zh-CN.js";
import { packs, terminology } from "../src/locales/packs.js";
import { currentLocalePatches } from "../src/locales/current-patches.js";
import { generatedLocalePatches } from "../src/locales/generated-patches.js";
import { OPTIONAL_LANGUAGE_OPTIONS } from "../src/locales/catalog.js";
import { officialWebsiteLabels } from "../src/locales/shared-labels.js";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
const checkOnly = process.argv.includes("--check");
const outputArg = process.argv.slice(2).find((arg) => arg !== "--check");
const outputDir = path.resolve(outputArg ?? path.join(root, "release-assets"));

const localeDefinitions = OPTIONAL_LANGUAGE_OPTIONS;

function deepMerge(base, patch) {
  const out = { ...base };
  for (const [key, value] of Object.entries(patch ?? {})) {
    out[key] = value && typeof value === "object" && !Array.isArray(value)
      && base[key] && typeof base[key] === "object" && !Array.isArray(base[key])
      ? deepMerge(base[key], value)
      : value;
  }
  return out;
}

const sourceBundles = {
  en,
  "zh-CN": zhCN,
  ...Object.fromEntries(Object.keys(packs).map((locale) => [
    locale,
    deepMerge(
      deepMerge(
        deepMerge(packs[locale] ?? {}, { terms: terminology[locale] ?? {} }),
        currentLocalePatches[locale] ?? {},
      ),
      generatedLocalePatches[locale] ?? {},
    ),
  ])),
};

const britishSpelling = [
  [/\bbehavior\b/gi, (value) => preserveCase(value, "behaviour")],
  [/\bcolor\b/gi, (value) => preserveCase(value, "colour")],
  [/\bcenter\b/gi, (value) => preserveCase(value, "centre")],
  [/\blicense\b/gi, (value) => preserveCase(value, "licence")],
];

function preserveCase(source, target) {
  return source[0] === source[0].toUpperCase()
    ? `${target[0].toUpperCase()}${target.slice(1)}`
    : target;
}

function mapStrings(value, transform) {
  if (typeof value === "string") return transform(value);
  if (Array.isArray(value)) return value.map((item) => mapStrings(item, transform));
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [key, mapStrings(item, transform)]),
    );
  }
  return value;
}

function cloneBundle(definition) {
  const source = definition.source ?? definition.base;
  const base = structuredClone(sourceBundles[source]);
  if (!base) throw new Error(`Missing source locale: ${source}`);
  const localized = definition.spelling === "british"
    ? mapStrings(base, (text) => britishSpelling.reduce(
        (value, [pattern, replacement]) => value.replace(pattern, replacement),
        text,
      ))
    : base;
  localized.meta = { ...(localized.meta ?? {}), locale: definition.code };
  localized.about.website = officialWebsiteLabels[definition.code]
    ?? officialWebsiteLabels[source]
    ?? en.about.website;
  return localized;
}

function flatten(value, prefix = "", out = {}) {
  for (const [key, item] of Object.entries(value ?? {})) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    if (item && typeof item === "object" && !Array.isArray(item)) flatten(item, fullKey, out);
    else out[fullKey] = String(item);
  }
  return out;
}

function placeholders(text) {
  return [...text.matchAll(/\{(\w+)\}/g)].map((match) => match[1]).sort().join(",");
}

const reference = flatten(en);
const outputPacks = [];
for (const definition of localeDefinitions) {
  const bundle = cloneBundle(definition);
  const translated = flatten(bundle);
  const missing = Object.keys(reference).filter((key) => !(key in translated));
  const mismatched = Object.keys(reference).filter(
    (key) => key in translated && placeholders(reference[key]) !== placeholders(translated[key]),
  );
  if (missing.length || mismatched.length) {
    throw new Error(
      `${definition.code} failed validation: ${missing.length} missing, ${mismatched.length} placeholder mismatches`,
    );
  }
  outputPacks.push({
    schema_version: 1,
    app_version: packageJson.version,
    locales: [{ code: definition.code, label: definition.label }],
    bundles: { [definition.code]: bundle },
  });
}

if (checkOnly) {
  console.log(`Validated ${localeDefinitions.length} independent optional language packs.`);
  process.exit(0);
}

fs.mkdirSync(outputDir, { recursive: true });
for (const pack of outputPacks) {
  const code = pack.locales[0].code;
  const outputPath = path.join(outputDir, `zerotick-language-${code}-v${packageJson.version}.json`);
  fs.writeFileSync(outputPath, `${JSON.stringify(pack)}\n`, "utf8");
  console.log(`Language pack: ${outputPath}`);
}

const manifest = {
  schema_version: 1,
  app_version: packageJson.version,
  languages: localeDefinitions.map(({ code, label }) => ({
    code,
    label,
    asset: `zerotick-language-${code}-v${packageJson.version}.json`,
  })),
};
const manifestPath = path.join(outputDir, `zerotick-languages-v${packageJson.version}.json`);
fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
console.log(`Language manifest: ${manifestPath}`);
