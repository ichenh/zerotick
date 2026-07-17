import en from "./en.js";
import zhCN from "./zh-CN.js";
import { packs, terminology } from "./packs.js";
import { currentLocalePatches } from "./current-patches.js";
import { generatedLocalePatches } from "./generated-patches.js";
import { OPTIONAL_LANGUAGE_OPTIONS } from "./catalog.js";
import { officialWebsiteLabels } from "./shared-labels.js";

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

export function assembleLocaleBundle(definition) {
  const source = definition.source ?? definition.base;
  const base = structuredClone(sourceBundles[source]);
  if (!base) throw new Error(`Missing source locale: ${source}`);
  // Newly introduced status labels fall back to user-facing English text until
  // that locale supplies its own translation. Never expose internal state IDs.
  for (const key of ["batteryRefreshing", "batteryUnavailable"]) {
    base.toolkit.bluetooth[key] ??= en.toolkit.bluetooth[key];
  }
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

export function assembleOptionalBundles() {
  return Object.fromEntries(
    OPTIONAL_LANGUAGE_OPTIONS.map((definition) => [
      definition.code,
      assembleLocaleBundle(definition),
    ]),
  );
}
