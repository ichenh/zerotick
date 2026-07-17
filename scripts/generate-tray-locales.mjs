import { spawn } from "node:child_process";
import fs from "node:fs";

const path = new URL("../src-tauri/locales/tray.json", import.meta.url);
const locales = JSON.parse(fs.readFileSync(path, "utf8"));
const reference = locales.en;
const languageMap = { "pt-BR": "pt" };

function placeholders(text) {
  const values = [];
  const protectedText = text.replace(/\{\w+\}/g, (value) => {
    const token = `__ZTPH_${values.length}__`;
    values.push([token, value]);
    return token;
  });
  return { protectedText, values };
}

function curl(args) {
  return new Promise((resolve, reject) => {
    const child = spawn("curl.exe", args, { windowsHide: true });
    let output = "";
    let error = "";
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk) => { output += chunk; });
    child.stderr.on("data", (chunk) => { error += chunk; });
    child.on("error", reject);
    child.on("close", (code) => code === 0 ? resolve(output) : reject(new Error(error)));
  });
}

async function translate(locale, entries) {
  const prepared = entries.map(([key, value], index) => ({
    key,
    marker: `__ZTSEP_${String(index).padStart(3, "0")}__`,
    ...placeholders(value),
  }));
  const text = prepared.map((entry, index) => index === prepared.length - 1
    ? entry.protectedText
    : `${entry.protectedText}\n${entry.marker}\n`).join("");
  const raw = await curl([
    "-sS", "--fail", "--retry", "3", "--get",
    "--data-urlencode", "client=gtx",
    "--data-urlencode", "sl=en",
    "--data-urlencode", `tl=${languageMap[locale] ?? locale}`,
    "--data-urlencode", "dt=t",
    "--data-urlencode", `q=${text}`,
    "https://translate.googleapis.com/translate_a/single",
  ]);
  const joined = JSON.parse(raw)[0].map((segment) => segment[0]).join("");
  const parts = joined.split(/\n?__ZTSEP_\d{3}__\n?/g);
  if (parts.length !== prepared.length) throw new Error(`${locale}: boundary mismatch`);
  return Object.fromEntries(prepared.map((entry, index) => {
    const restored = entry.values.reduce(
      (value, [token, original]) => value.replaceAll(token, original),
      parts[index].trim(),
    );
    return [entry.key, restored];
  }));
}

for (const [locale, existing] of Object.entries(locales)) {
  if (locale === "en" || locale === "zh-CN") continue;
  const missing = Object.entries(reference).filter(([key]) => !(key in existing));
  const batches = [];
  for (let index = 0; index < missing.length; index += 18) {
    batches.push(missing.slice(index, index + 18));
  }
  const additions = await Promise.all(batches.map((batch) => translate(locale, batch)));
  locales[locale] = Object.assign({}, existing, ...additions);
  console.log(`${locale}: completed ${missing.length} tray and notification strings`);
}

fs.writeFileSync(path, `${JSON.stringify(locales, null, 2)}\n`, "utf8");
