import en from "./locales/en.js";
import { assembleOptionalBundles } from "./locales/assemble.js";
import { ALL_LANGUAGE_OPTIONS } from "./locales/catalog.js";

export const bundles = {
  en,
  ...assembleOptionalBundles(),
};

const LEGACY_KEY_ALIASES = {
  "toolkit.ok": "diag.bluetooth.ok",
  "toolkit.warn": "diag.bluetooth.warn",
  "toolkit.unknown": "diag.bluetooth.unknown",
  "toolkit.bluetooth.idle": "diag.bluetooth.idle",
  "toolkit.bluetooth.radioCount": "diag.bluetooth.radioCount",
  "toolkit.bluetooth.issues": "diag.bluetooth.issues",
  "toolkit.bluetooth.noIssues": "diag.bluetooth.noIssues",
};

export const LOCALE_OPTIONS = ALL_LANGUAGE_OPTIONS;
export const LANGUAGE_CATALOG = ALL_LANGUAGE_OPTIONS;

export function isLocaleInstalled(code) {
  return Boolean(bundles[code]);
}

let currentLocale = "zh-CN";
let dict = bundles["zh-CN"] ?? en;

function getPath(obj, path) {
  return path.split(".").reduce((o, k) => (o && o[k] !== undefined ? o[k] : undefined), obj);
}

function interpolate(str, params) {
  if (!params) return str;
  return String(str).replace(/\{(\w+)\}/g, (_, k) =>
    params[k] !== undefined && params[k] !== null ? String(params[k]) : `{${k}}`,
  );
}

export function normalizeLocale(code) {
  const supported = matchSupportedLocale(code);
  if (bundles[supported]) return supported;
  return "en";
}

export function matchSupportedLocale(code) {
  if (!code) return "en";
  const map = {
    zh: "zh-CN",
    "zh-Hans": "zh-CN",
    "zh-Hant": "zh-TW",
    pt: "pt-BR",
    no: "nb",
  };
  const mapped = map[code] ?? code;
  if (LANGUAGE_CATALOG.some((locale) => locale.code === mapped)) return mapped;
  const base = mapped.split("-")[0];
  if (LANGUAGE_CATALOG.some((locale) => locale.code === base)) return base;
  return "en";
}

export function getLocale() {
  return currentLocale;
}

export function t(key, params) {
  const alias = LEGACY_KEY_ALIASES[key];
  const val = getPath(dict, key)
    ?? (alias ? getPath(dict, alias) : undefined)
    ?? getPath(en, key)
    ?? key;
  if (typeof val !== "string" && typeof val !== "number") return String(key);
  return interpolate(String(val), params);
}

function applyToElement(el) {
  const key = el.getAttribute("data-i18n");
  if (!key) return;
  const attr = el.getAttribute("data-i18n-attr");
  const text = t(key);
  if (attr) {
    el.setAttribute(attr, text);
  } else {
    el.textContent = text;
  }
}

export function applyDom(root = document) {
  root.querySelectorAll("[data-i18n]").forEach(applyToElement);
  document.title = t("app.title");
}

export function fillLocaleSelect(select) {
  if (!select) return;
  const prev = select.value;
  select.replaceChildren();
  for (const locale of LOCALE_OPTIONS) {
    const option = document.createElement("option");
    option.value = locale.code;
    option.textContent = locale.label;
    select.append(option);
  }
  select.value = bundles[prev] ? prev : currentLocale;
}

export function setLocale(code) {
  const normalized = normalizeLocale(code);
  currentLocale = bundles[normalized] ? normalized : "en";
  dict = bundles[currentLocale] ?? en;
  const dir = dict.meta?.dir === "rtl" ? "rtl" : "ltr";
  document.documentElement.lang = currentLocale;
  document.documentElement.dir = dir;
  applyDom();
  return currentLocale;
}

export function getPageMeta(pageId) {
  return {
    title: t(`pages.${pageId}.title`),
    desc: t(`pages.${pageId}.desc`),
  };
}

export function formatTime(iso) {
  try {
    return new Date(iso).toLocaleTimeString(currentLocale, { hour12: false });
  } catch {
    return iso;
  }
}

/** Format an integer-millisecond measurement without losing available precision. */
export function formatDuration(durationMs) {
  const milliseconds = Math.max(0, Math.trunc(Number(durationMs) || 0));
  if (milliseconds < 1000) return `${milliseconds} ${t("units.ms")}`;
  const seconds = new Intl.NumberFormat(currentLocale, {
    useGrouping: false,
    maximumFractionDigits: 3,
  }).format(milliseconds / 1000);
  return `${seconds} ${t("units.sec")}`;
}
