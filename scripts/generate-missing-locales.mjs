import { spawn } from "node:child_process";
import fs from "node:fs";
import en from "../src/locales/en.js";
import { packs, terminology } from "../src/locales/packs.js";
import { currentLocalePatches } from "../src/locales/current-patches.js";

const completed = new Set(["zh-TW", "ja", "ko", "de"]);
const targets = Object.keys(packs).filter((locale) => !completed.has(locale));
const languageMap = { "pt-BR": "pt" };

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

function flatten(value, prefix = "", output = {}) {
  for (const [key, item] of Object.entries(value ?? {})) {
    const path = prefix ? `${prefix}.${key}` : key;
    if (item && typeof item === "object" && !Array.isArray(item)) flatten(item, path, output);
    else output[path] = String(item);
  }
  return output;
}

function setPath(target, path, value) {
  const parts = path.split(".");
  let cursor = target;
  for (const part of parts.slice(0, -1)) cursor = cursor[part] ??= {};
  cursor[parts.at(-1)] = value;
}

function protectPlaceholders(text) {
  const placeholders = [];
  const protectedText = text.replace(/\{\w+\}/g, (value) => {
    const token = `__ZTPH_${placeholders.length}__`;
    placeholders.push([token, value]);
    return token;
  });
  return { protectedText, placeholders };
}

function restorePlaceholders(text, placeholders) {
  return placeholders.reduce((value, [token, original]) => value.replaceAll(token, original), text);
}

function placeholderSignature(text) {
  return [...String(text).matchAll(/\{\w+\}/g)].map((match) => match[0]).sort().join(",");
}

function runCurl(args) {
  return new Promise((resolve, reject) => {
    const child = spawn("curl.exe", args, { windowsHide: true });
    let stdout = "";
    let stderr = "";
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => { stdout += chunk; });
    child.stderr.on("data", (chunk) => { stderr += chunk; });
    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0) resolve(stdout);
      else reject(new Error(`curl failed (${code}): ${stderr.trim()}`));
    });
  });
}

async function translateBatch(locale, entries) {
  const prepared = entries.map(({ key, value }, index) => ({
    key,
    marker: `__ZTSEP_${String(index).padStart(3, "0")}__`,
    ...protectPlaceholders(value),
  }));
  const payload = prepared.map((entry) => entry.protectedText).join(
    prepared.length > 1
      ? `\n${prepared.slice(0, -1).map((entry) => entry.marker).join("\n")}`
      : "",
  );
  const joined = prepared
    .map((entry, index) => index === prepared.length - 1
      ? entry.protectedText
      : `${entry.protectedText}\n${entry.marker}\n`)
    .join("");
  void payload;
  const raw = await runCurl([
    "-sS", "--fail", "--retry", "3", "--get",
    "--data-urlencode", "client=gtx",
    "--data-urlencode", "sl=en",
    "--data-urlencode", `tl=${languageMap[locale] ?? locale}`,
    "--data-urlencode", "dt=t",
    "--data-urlencode", `q=${joined}`,
    "https://translate.googleapis.com/translate_a/single",
  ]);
  const response = JSON.parse(raw);
  const translated = response[0].map((segment) => segment[0]).join("");
  const markerPattern = /\n?__ZTSEP_\d{3}__\n?/g;
  const values = translated.split(markerPattern);
  if (values.length !== prepared.length) {
    throw new Error(`${locale}: batch boundary mismatch (${values.length}/${prepared.length})`);
  }
  return prepared.map((entry, index) => ({
    key: entry.key,
    value: restorePlaceholders(values[index].trim(), entry.placeholders),
  }));
}

async function mapConcurrent(items, limit, task) {
  const results = new Array(items.length);
  let cursor = 0;
  await Promise.all(Array.from({ length: Math.min(limit, items.length) }, async () => {
    while (cursor < items.length) {
      const index = cursor;
      cursor += 1;
      results[index] = await task(items[index], index);
    }
  }));
  return results;
}

const reference = flatten(en);
const patches = {};
for (const locale of targets) {
  const existing = flatten(deepMerge(
    deepMerge(packs[locale] ?? {}, { terms: terminology[locale] ?? {} }),
    currentLocalePatches[locale] ?? {},
  ));
  const missing = Object.entries(reference)
    .filter(([key, value]) => !(key in existing)
      || placeholderSignature(value) !== placeholderSignature(existing[key]))
    .map(([key, value]) => ({ key, value }));
  const batches = [];
  let batch = [];
  let characters = 0;
  for (const entry of missing) {
    if (batch.length >= 20 || characters + entry.value.length > 3000) {
      batches.push(batch);
      batch = [];
      characters = 0;
    }
    batch.push(entry);
    characters += entry.value.length;
  }
  if (batch.length) batches.push(batch);
  console.log(`${locale}: translating ${missing.length} missing strings in ${batches.length} batches`);
  const translatedBatches = await mapConcurrent(batches, 6, (items) => translateBatch(locale, items));
  const patch = {};
  for (const entry of translatedBatches.flat()) setPath(patch, entry.key, entry.value);
  patches[locale] = patch;
}

const banner = `// Machine-assisted completion generated from src/locales/en.js.\n// Existing translated strings take precedence; community language review is welcome.\n`;
fs.writeFileSync(
  new URL("../src/locales/generated-patches.js", import.meta.url),
  `${banner}export const generatedLocalePatches = ${JSON.stringify(patches, null, 2)};\n`,
  "utf8",
);
console.log(`Generated complete patches for ${targets.length} locales.`);
