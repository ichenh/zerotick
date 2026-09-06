import en from "./en.js";
import zhCN from "./zh-CN.js";
import { packs, terminology } from "./packs.js";
import { currentLocalePatches } from "./current-patches.js";
import { generatedLocalePatches } from "./generated-patches.js";
import {
  releaseLocalePatches,
  releaseSupplementLocalePatches,
} from "./release-patches.js";
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
        deepMerge(
          deepMerge(packs[locale] ?? {}, { terms: terminology[locale] ?? {} }),
          currentLocalePatches[locale] ?? {},
        ),
        generatedLocalePatches[locale] ?? {},
      ),
      deepMerge(
        releaseLocalePatches[locale] ?? {},
        releaseSupplementLocalePatches[locale] ?? {},
      ),
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
  // These audio repair messages use explicit English fallback until a locale
  // supplies translations; do not present missing keys as successful recovery.
  base.toolkit.audio.repair = deepMerge(en.toolkit.audio.repair, base.toolkit.audio.repair);
  base.toolkit.audio.muteUnavailable ??= en.toolkit.audio.muteUnavailable;
  for (const key of ["scopeNotice", "noChanges"]) {
    base.diag.repair[key] ??= en.diag.repair[key];
  }
  base.errors.serviceRepairBusy ??= en.errors.serviceRepairBusy;
  // New network evidence and target tests use English fallback until the
  // corresponding locale provides reviewed translations.
  for (const key of ["evidence", "target"]) {
    base.toolkit.network[key] = deepMerge(en.toolkit.network[key], base.toolkit.network[key]);
  }
  // Connectivity verdicts must remain readable in every bundled locale while
  // translation packs catch up with the new live Network List Manager evidence.
  for (const key of [
    "internetAccess", "networkConnection", "available", "unavailable", "connected", "disconnected",
    "adaptersPresent",
    "diagnosisTitle",
  ]) {
    base.toolkit.network[key] ??= en.toolkit.network[key];
  }
  base.toolkit.network.diagnosis ??= {};
  for (const key of [
    "snapshotHealthy", "noAdapter", "linkDisconnected", "gatewayUnreachable",
    "upstreamUnavailable", "vpnOrProxy", "serviceIssue", "statusUnknown",
  ]) {
    base.toolkit.network.diagnosis[key] ??= en.toolkit.network.diagnosis[key];
  }
  for (const key of [
    "disconnectedTitle", "disconnectedSummary", "disconnectedStep1", "disconnectedStep2",
    "noInternetTitle", "noInternetSummary", "noInternetStep1", "noInternetStep2", "noInternetStep3",
    "connectivityUnknownTitle", "connectivityUnknownSummary", "connectivityUnknownStep1",
  ]) {
    base.toolkit.network.guide[key] ??= en.toolkit.network.guide[key];
  }
  for (const key of ["networkDisconnected", "internetUnavailable", "connectivityUnknown"]) {
    base.overview.health[key] ??= en.overview.health[key];
  }
  // Driver repair is evidence-driven and must never expose internal action IDs.
  // Until each translation pack is refreshed, use complete user-facing English
  // wording instead of presenting missing keys as if they were real operations.
  for (const key of [
    "driverEvidenceUnavailable",
    "repairing",
    "infFilter",
    "confirmTitle",
    "confirmMessage",
    "confirmRun",
    "cancel",
    "action",
    "risk",
    "result",
  ]) {
    base.toolkit.devices[key] ??= structuredClone(en.toolkit.devices[key]);
  }
  // Manual port termination must always show a readable confirmation and risk
  // warning, even before every translation pack has been refreshed.
  for (const key of ["confirmTitle", "confirmMessage", "confirmWarning", "confirmRun", "cancel"]) {
    base.ports[key] ??= en.ports[key];
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
