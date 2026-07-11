import en from "./locales/en.js";
import zhCN from "./locales/zh-CN.js";
import { packs } from "./locales/packs.js";

/** @typedef {Record<string, unknown>} LocaleBundle */

export function deepMerge(base, patch) {
  const out = { ...base };
  for (const [key, val] of Object.entries(patch ?? {})) {
    if (
      val &&
      typeof val === "object" &&
      !Array.isArray(val) &&
      base[key] &&
      typeof base[key] === "object" &&
      !Array.isArray(base[key])
    ) {
      out[key] = deepMerge(/** @type {LocaleBundle} */ (base[key]), /** @type {LocaleBundle} */ (val));
    } else {
      out[key] = val;
    }
  }
  return out;
}

const bundles = {
  en,
  "zh-CN": zhCN,
  ...packs,
};

export const LOCALE_OPTIONS = [
  { code: "en", label: "English" },
  { code: "zh-CN", label: "简体中文" },
  { code: "zh-TW", label: "繁體中文" },
  { code: "ja", label: "日本語" },
  { code: "ko", label: "한국어" },
  { code: "de", label: "Deutsch" },
  { code: "fr", label: "Français" },
  { code: "es", label: "Español" },
  { code: "pt-BR", label: "Português (Brasil)" },
  { code: "ru", label: "Русский" },
  { code: "ar", label: "العربية" },
  { code: "hi", label: "हिन्दी" },
  { code: "it", label: "Italiano" },
  { code: "nl", label: "Nederlands" },
  { code: "pl", label: "Polski" },
  { code: "tr", label: "Türkçe" },
  { code: "vi", label: "Tiếng Việt" },
  { code: "th", label: "ไทย" },
  { code: "id", label: "Bahasa Indonesia" },
  { code: "cs", label: "Čeština" },
  { code: "da", label: "Dansk" },
  { code: "fi", label: "Suomi" },
  { code: "nb", label: "Norsk Bokmål" },
  { code: "sv", label: "Svenska" },
  { code: "uk", label: "Українська" },
  { code: "he", label: "עברית" },
  { code: "ms", label: "Bahasa Melayu" },
  { code: "ro", label: "Română" },
  { code: "hu", label: "Magyar" },
];

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
  if (!code) return "en";
  const map = {
    zh: "zh-CN",
    "zh-Hans": "zh-CN",
    "zh-Hant": "zh-TW",
    pt: "pt-BR",
    no: "nb",
  };
  const mapped = map[code] ?? code;
  if (bundles[mapped]) return mapped;
  const base = mapped.split("-")[0];
  if (bundles[base]) return base;
  return "en";
}

export function getLocale() {
  return currentLocale;
}

export function t(key, params) {
  const val = getPath(dict, key) ?? getPath(en, key) ?? key;
  return interpolate(val, params);
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
  select.innerHTML = LOCALE_OPTIONS.map(
    (o) => `<option value="${o.code}">${o.label}</option>`,
  ).join("");
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
