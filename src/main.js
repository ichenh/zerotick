import "./styles.css";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
  t,
  setLocale,
  getPageMeta,
  formatDuration,
  formatTime,
  getLocale,
  isLocaleInstalled,
  LANGUAGE_CATALOG,
  matchSupportedLocale,
  normalizeLocale,
} from "./i18n.js";
import {
  escapeHtml,
  renderBluetoothReport,
  formatBluetoothIssue,
  renderPortScan,
  bindToolkitHandlers,
  applyFullScanProgress,
} from "./panels.js";
import { enhanceSelectMenus, refreshSelectMenus } from "./select-menu.js";

const $ = (id) => document.getElementById(id);
let toastTimer = null;
let settingsLoaded = false;
let settingsSaveTimer = null;
let settingsSaveChain = Promise.resolve();
let portsLoaded = false;
let portsScanPromise = null;
let navItems = [];
let pages = [];
let activePageId = null;
let elevatedStatePromise = null;
let appVersion = "";
let updateInfo = null;
let updateCheckState = "idle";
let appSettings = {
  transient_threshold_ms: 500,
  tray_recovery_secs: 45,
  max_history_entries: 500,
  timeline_display_max: 80,
  timeline_order: "desc",
  native_notifications: true,
  launch_at_startup: false,
  close_to_tray: true,
  run_as_admin: false,
  advanced_display: false,
  bluetooth_poll_secs: 60,
  full_scan_timeout_secs: 25,
  system_query_timeout_secs: 20,
  network_test_timeout_secs: 20,
  bsod_debugger_timeout_secs: 90,
  locale: "en",
  locale_auto_configured: false,
};
window.__zerotickSettings = appSettings;

const TRAY_CLS = {
  normal: "ok",
  warning: "warn",
  critical: "crit",
};

function waitForTauri(maxMs = 8000) {
  return new Promise((resolve, reject) => {
    if (window.__TAURI_INTERNALS__) {
      resolve();
      return;
    }
    const deadline = Date.now() + maxMs;
    const tick = () => {
      if (window.__TAURI_INTERNALS__) {
        resolve();
      } else if (Date.now() > deadline) {
        const hint = isTauri() ? t("errors.tauriTimeout") : t("errors.notTauri");
        reject(new Error(hint));
      } else {
        requestAnimationFrame(tick);
      }
    };
    tick();
  });
}

function switchPage(pageId) {
  const pageChanged = activePageId !== pageId;
  if (pageChanged) {
    navItems.forEach((btn) => {
      btn.classList.toggle("active", btn.dataset.page === pageId);
    });
    pages.forEach((page) => {
      page.classList.toggle("active", page.id === `page-${pageId}`);
    });
    activePageId = pageId;
  }
  const meta = getPageMeta(pageId);
  if (meta) {
    $("page-title").textContent = meta.title;
    $("page-desc").textContent = meta.desc;
  }
  if (pageChanged && pageId === "repair") {
    void refreshAdminHint();
  }
  if (pageId === "ports" && !portsLoaded) {
    void scanPorts();
  }
}

function setStatus(text, level = "ok") {
  const el = $("engine-status");
  el.className = `status-chip ${level}`;
  const textEl = el.querySelector(".status-text");
  if (textEl) textEl.textContent = text;
}

function applyTrayStatus(ev) {
  const level = ev.level ?? "normal";
  const cls = TRAY_CLS[level] ?? TRAY_CLS.normal;
  const reasonKey = ev.reason_id ? `tray.reason.${ev.reason_id}` : null;
  const localizedReason = reasonKey ? t(reasonKey) : "";
  const reason = localizedReason && localizedReason !== reasonKey
    ? localizedReason
    : document.documentElement.classList.contains("show-advanced")
      ? (ev.message || ev.reason_id || t("tray.reason.unknown"))
      : t("tray.reason.unknown");
  if (level === "normal") {
    setStatus(reason || t("tray.normal"), cls);
    return;
  }
  const labelKey = level === "critical" ? "tray.critical" : "tray.warning";
  setStatus(`${t(labelKey)} · ${reason}`, cls);
}

function categoryLabel(code) {
  const key = `events.category.${code}`;
  const label = t(key);
  return label !== key ? label : t("events.category.unknown");
}

function isGenericDeviceName(name) {
  return [
    "usb composite device",
    "usb device",
    "unknown usb device",
    "bluetooth device",
    "unknown device",
  ].includes(String(name ?? "").trim().toLowerCase());
}

function userDeviceName(ev) {
  const friendly = String(ev?.friendly_name ?? "").trim();
  return friendly && !isGenericDeviceName(friendly)
    ? friendly
    : t("events.unknownDevice");
}

function advancedDeviceName(ev) {
  const name = userDeviceName(ev);
  return name === t("events.unknownDevice") && ev?.vid_pid
    ? `${name} (${ev.vid_pid})`
    : name;
}

function formatEventMessage(ev) {
  const name = userDeviceName(ev);
  const category = categoryLabel(ev.category);
  const ms = ev.disconnect_ms ?? 0;
  if (ev.event_type === "transient_reconnect") {
    return t("events.msg.transient", { category, name, ms });
  }
  if (ev.event_type === "arrival" && ev.disconnect_ms != null) {
    return t("events.msg.reconnect", { category, name, ms });
  }
  if (ev.event_type === "arrival") {
    return t("events.msg.arrival", { category, name });
  }
  if (ev.event_type === "remove") {
    return t("events.msg.remove", { category, name });
  }
  return t("events.msg.unknown");
}

function formatRepairSummary(ev) {
  if (ev.summary_id) {
    const key = `diag.repair.summary.${ev.summary_id}`;
    const text = t(key, { n: ev.summary_count ?? 0 });
    if (text !== key) return text;
  }
  if (ev.summary && document.documentElement.classList.contains("show-advanced")) {
    return ev.summary;
  }
  return t("diag.repair.summary.unknown");
}

async function syncWindowTitle() {
  const title = t("app.title");
  document.title = title;
  try {
    await getCurrentWebviewWindow().setTitle(title);
  } catch {
    /* 非 Tauri 环境 */
  }
}

async function reloadTimeline() {
  $("timeline").innerHTML = "";
  await loadHistory();
  updateTimelineEmpty();
}

function showToast(message, critical = false) {
  const toast = $("toast");
  toast.textContent = message;
  toast.classList.remove("hidden", "warn", "crit");
  toast.classList.add(critical ? "crit" : "warn");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => toast.classList.add("hidden"), critical ? 8000 : 4000);
}

function renderList(items, emptyText) {
  if (!items?.length) {
    return `<p class="result-muted">${escapeHtml(emptyText)}</p>`;
  }
  return `<ul class="result-list">${items.map((i) => `<li>${escapeHtml(i)}</li>`).join("")}</ul>`;
}

function setResultPanel(el, state, html) {
  el.className = `result-panel result-${state}`;
  el.innerHTML = html;
}

function updateTimelineEmpty() {
  const list = $("timeline");
  const empty = $("timeline-empty");
  const hasItems = list.children.length > 0;
  empty.classList.toggle("hidden", hasItems);
}

function refreshStaticPanels() {
  const idlePanels = [
    ["network-result", "toolkit.network.idle"],
    ["audio-result", "toolkit.audio.idle"],
    ["usb-result", "toolkit.usb.idle"],
    ["bluetooth-result", "toolkit.bluetooth.idle"],
    ["bsod-result", "diag.bsod.idle"],
    ["repair-result", "diag.repair.idle"],
  ];
  for (const [id, key] of idlePanels) {
    const el = $(id);
    if (el.classList.contains("result-idle")) {
      const p = el.querySelector(".result-empty");
      if (p) p.textContent = t(key);
    }
  }

  const list = $("port-list");
  const empty = list.querySelector(":scope > .empty-state");
  if (empty && !list.querySelector(".port-row")) {
    empty.textContent = t("ports.empty");
  }

  const releaseAllBtn = $("btn-release-all-ports");
  if (releaseAllBtn?.disabled) {
    releaseAllBtn.textContent = t("ports.releaseAll");
  }

  const adminHint = $("repair-admin-hint");
  if (adminHint && !adminHint.classList.contains("hidden")) {
    adminHint.textContent = t("diag.repair.adminHint");
  }

  refreshSpinLabels();
  renderUpdateStatus();
  renderLanguageList();
  refreshSelectMenus();

  const active = document.querySelector(".nav-item.active");
  if (active?.dataset.page) {
    switchPage(active.dataset.page);
  }
}

function renderBsodCard(ev) {
  const el = $("bsod-result");
  if (!ev) {
    setResultPanel(el, "ok", `<p class="result-empty">${escapeHtml(t("diag.bsod.none"))}</p>`);
    return;
  }

  const state = ev.is_recent ? "crit" : "ok";
  const recent = ev.is_recent ? t("diag.bsod.recent") : t("diag.bsod.history");
  const code = [ev.bugcheck_code, ev.code_name].filter(Boolean).join(" · ") || "—";
  const fixIds = (ev.fixes ?? []).filter((fix) => fix.automatic).map((fix) => fix.id);
  const fixes = (ev.fixes ?? []).length
    ? `<ul class="result-list bsod-fix-list">${ev.fixes
        .map((fix) => `<li>${escapeHtml(t(`diag.bsod.fix.${fix.id}`))}</li>`)
        .join("")}</ul>`
    : `<p class="result-muted">${escapeHtml(t("diag.bsod.noFixes"))}</p>`;
  const confidence = ev.analysis_kind === "root_cause"
    ? t("diag.bsod.confidence.module")
    : ev.bugcheck_code
      ? t("diag.bsod.confidence.code")
      : t("diag.bsod.confidence.limited");
  const analysisTitle = ev.analysis_kind === "root_cause"
    ? t("diag.bsod.rootCause")
    : t("diag.bsod.errorTypeAnalysis");
  const errorType = t(`diag.bsod.analysis.${ev.analysis_id ?? "unknown"}`);
  const analysisText = ev.analysis_kind === "root_cause" && ev.faulting_module
    ? t("diag.bsod.moduleEvidenceAnalysis", { module: ev.faulting_module, analysis: errorType })
    : errorType;

  setResultPanel(
    el,
    state,
    `
    <div class="result-head">
      <span class="result-badge result-badge-${state}">${recent}</span>
      <span class="result-time">${formatTime(ev.timestamp)}</span>
    </div>
    <div class="result-section">
      <div class="result-section-title">${escapeHtml(analysisTitle)}</div>
      <p class="bsod-root-cause">${escapeHtml(analysisText)}</p>
      <p class="result-muted bsod-confidence">${escapeHtml(confidence)}</p>
    </div>
    <details class="technical-details advanced-only">
      <summary>${escapeHtml(t("toolkit.technicalDetails"))}</summary>
      <div class="result-row"><span class="result-label">${escapeHtml(t("diag.bsod.bugcheck"))}</span><span class="result-value mono">${escapeHtml(code)}</span></div>
      <div class="result-row"><span class="result-label">${escapeHtml(t("diag.bsod.faultingModule"))}</span><span class="result-value mono">${escapeHtml(ev.faulting_module ?? "—")}</span></div>
      ${ev.debugger ? `<div class="result-row"><span class="result-label">${escapeHtml(t("diag.bsod.debugger"))}</span><span class="result-value mono">${escapeHtml(ev.debugger)}</span></div>` : ""}
      ${ev.failure_bucket ? `<div class="result-row"><span class="result-label">${escapeHtml(t("diag.bsod.failureBucket"))}</span><span class="result-value mono">${escapeHtml(ev.failure_bucket)}</span></div>` : ""}
      ${ev.dump_time ? `<div class="result-row"><span class="result-label">${escapeHtml(t("diag.bsod.dumpTime"))}</span><span class="result-value mono">${escapeHtml(ev.dump_time)}</span></div>` : ""}
      <div class="result-row"><span class="result-label">${escapeHtml(t("diag.bsod.dumpPath"))}</span><span class="result-value mono" title="${escapeHtml(ev.dump_path)}">${escapeHtml(ev.dump_path)}</span></div>
      ${ev.stack_summary ? `<div class="result-section"><div class="result-section-title">${escapeHtml(t("diag.bsod.stackSummary"))}</div><p class="result-value mono">${escapeHtml(ev.stack_summary)}</p></div>` : ""}
    </details>
    <div class="result-section">
      <div class="result-section-title">${escapeHtml(t("diag.bsod.repairs"))}</div>
      ${fixes}
      ${!ev.is_recent ? `<p class="result-muted">${escapeHtml(t("diag.bsod.historicalRepairHint"))}</p>` : ""}
      ${fixIds.length ? `<button type="button" class="btn btn-accent btn-bsod-repair" data-fixes="${fixIds.join(",")}">${escapeHtml(t("diag.bsod.runRepairs"))}</button>` : ""}
      <div class="bsod-repair-results" aria-live="polite"></div>
    </div>
  `,
  );
}

function renderRepairCard(ev) {
  const el = $("repair-result");
  let state = "ok";
  if (!ev.success) state = ev.needs_admin ? "warn" : "crit";

  const repaired = renderList(ev.services_restarted, t("diag.repair.noneRepaired"));
  const healthy = renderList(ev.services_healthy, t("diag.repair.noneHealthy"));
  const errors = ev.service_errors?.length ? renderList(ev.service_errors, "") : "";
  const usbConfigs = (ev.usb_power_configs ?? []).map((config) =>
    t("diag.repair.usbConfigItem", {
      device: config.device_id,
      instance: config.instance_group,
      count: config.interface_count,
    }),
  );
  const usb = renderList(usbConfigs, t("diag.repair.noUsbConfig"));

  setResultPanel(
    el,
    state,
    `
    <div class="result-head">
      <span class="result-badge result-badge-${state}">${escapeHtml(formatRepairSummary(ev))}</span>
      <span class="result-time">${formatTime(ev.timestamp ?? new Date().toISOString())}</span>
    </div>
    ${ev.needs_admin ? `<p class="info-banner">${escapeHtml(t("diag.repair.adminBanner"))}</p>` : ""}
    <div class="result-section">
      <div class="result-section-title">${escapeHtml(t("diag.repair.repaired"))}</div>
      ${repaired}
    </div>
    <div class="result-section">
      <div class="result-section-title">${escapeHtml(t("diag.repair.healthy"))}</div>
      ${healthy}
    </div>
    ${errors ? `<div class="result-section advanced-only"><div class="result-section-title">${escapeHtml(t("diag.repair.failed"))}</div>${errors}</div>` : ""}
    <div class="result-section advanced-only">
      <div class="result-section-title">${escapeHtml(t("diag.repair.usbConfig"))}</div>
      ${usb}
      ${ev.power_scan_error ? `<p class="result-msg warn mono">${escapeHtml(ev.power_scan_error)}</p>` : ""}
    </div>
  `,
  );
}

function appendTimeline(ev, { animate = true } = {}) {
  const li = document.createElement("li");
  const type =
    ev.event_type === "transient_reconnect"
      ? "transient"
      : ev.event_type === "remove"
        ? "remove"
        : "arrival";

  li.className = type;
  if (!animate) li.classList.add("no-animate");
  if (type === "transient" && animate) li.classList.add("pulse");

  const name = userDeviceName(ev);
  const technicalParts = [];
  if (ev.vid_pid) technicalParts.push(ev.vid_pid);
  if (ev.disconnect_ms != null) technicalParts.push(formatDuration(ev.disconnect_ms));
  if (ev.device_path) technicalParts.push(ev.device_path);
  if (ev.message) technicalParts.push(ev.message);

  li.innerHTML = `
    <span class="tl-time">${formatTime(ev.timestamp)}</span>
    <span class="tl-badge tl-badge-${type}">${eventBadge(ev.event_type)}</span>
    <div class="tl-body">
      <div class="tl-name">${escapeHtml(name)}</div>
      <div class="tl-meta">${escapeHtml(categoryLabel(ev.category))}${technicalParts.length ? `<span class="advanced-only advanced-inline"> · ${escapeHtml(technicalParts.join(" · "))}</span>` : ""}</div>
      <div class="tl-msg">${escapeHtml(formatEventMessage(ev))}</div>
    </div>
  `;
  const list = $("timeline");
  const eventTime = Date.parse(ev.timestamp);
  li.dataset.timestamp = String(Number.isFinite(eventTime) ? eventTime : Date.now());
  const descending = appSettings.timeline_order !== "asc";
  const insertBefore = [...list.children].find((child) => {
    const childTime = Number(child.dataset.timestamp);
    return descending ? childTime < Number(li.dataset.timestamp) : childTime > Number(li.dataset.timestamp);
  });
  if (insertBefore) list.insertBefore(li, insertBefore);
  else list.append(li);
  if (animate) {
    requestAnimationFrame(() => li.classList.add("visible"));
  } else {
    li.classList.add("visible");
  }

  while (list.children.length > appSettings.timeline_display_max) {
    list.removeChild(descending ? list.lastChild : list.firstChild);
  }
  updateTimelineEmpty();
}

function trimTimeline() {
  const list = $("timeline");
  const descending = appSettings.timeline_order !== "asc";
  while (list.children.length > appSettings.timeline_display_max) {
    list.removeChild(descending ? list.lastChild : list.firstChild);
  }
  updateTimelineEmpty();
}

function eventBadge(type) {
  if (type === "transient_reconnect") return t("events.transient");
  if (type === "remove") return t("events.remove");
  return t("events.arrival");
}

async function onTransientEvent(ev) {
  const name = document.documentElement.classList.contains("show-advanced")
    ? advancedDeviceName(ev)
    : userDeviceName(ev);
  showToast(
    document.documentElement.classList.contains("show-advanced")
      ? `${t("toast.transientSimple", { name })} · ${formatDuration(ev.disconnect_ms)}`
      : t("toast.transientSimple", { name }),
    true,
  );
  switchPage("overview");
  try {
    const win = getCurrentWebviewWindow();
    await win.show();
    await win.unminimize();
    await win.setFocus();
  } catch {
    /* 非 Tauri 环境 */
  }
}

async function loadHistory() {
  try {
    const items = await invoke("get_device_history");
    if (!Array.isArray(items) || items.length === 0) return;
    for (const item of items) appendTimeline(item, { animate: false });
  } catch {
    /* 历史加载失败时忽略 */
  }
}

async function loadSettings() {
  try {
    const s = await invoke("get_settings");
    appSettings = { ...appSettings, ...s };
    window.__zerotickSettings = appSettings;
    $("set-threshold").value = s.transient_threshold_ms;
    $("set-tray-recovery").value = s.tray_recovery_secs;
    $("set-history-max").value = s.max_history_entries;
    $("set-timeline-max").value = s.timeline_display_max;
    $("timeline-order").value = s.timeline_order === "asc" ? "asc" : "desc";
    $("set-bluetooth-poll").value = s.bluetooth_poll_secs;
    $("set-full-scan-timeout").value = s.full_scan_timeout_secs;
    $("set-system-query-timeout").value = s.system_query_timeout_secs;
    $("set-network-test-timeout").value = s.network_test_timeout_secs;
    $("set-bsod-debugger-timeout").value = s.bsod_debugger_timeout_secs;
    $("set-native-notify").checked = Boolean(s.native_notifications);
    $("set-launch-startup").checked = Boolean(s.launch_at_startup);
    $("set-run-as-admin").checked = Boolean(s.run_as_admin);
    $("set-advanced-display").checked = Boolean(s.advanced_display);
    document.documentElement.classList.toggle("show-advanced", Boolean(s.advanced_display));
    $("set-close-tray").value = s.close_to_tray === false ? "0" : "1";
    const localeEl = $("set-locale");
    if (s.locale) localeEl.value = s.locale;
    setLocale(s.locale);
    if (s.locale) localeEl.value = normalizeLocale(s.locale);
    renderLanguageList();
    refreshSelectMenus();
  } catch {
    /* 使用默认值 */
  } finally {
    settingsLoaded = true;
  }
}

function rawErrorText(error) {
  if (error instanceof Error && error.message) return error.message;
  return String(error ?? "").trim() || "Unknown error";
}

function userErrorMessage(error, fallbackKey = "errors.operationFailed") {
  const raw = rawErrorText(error).toLowerCase();
  if (
    raw.includes("admin_required") ||
    raw.includes("permissiondenied") ||
    raw.includes("access is denied") ||
    raw.includes("access denied") ||
    raw.includes("requested registry access is not allowed") ||
    raw.includes("securityexception")
  ) {
    return t("errors.permissionDenied");
  }
  if (raw.includes("timeout") || raw.includes("timed out") || raw.includes("超时")) {
    return t("errors.timeout");
  }
  return t(fallbackKey);
}

function renderOperationError(error, context, fallbackKey = "errors.detectionFailed") {
  return `<p class="result-msg warn">${escapeHtml(userErrorMessage(error, fallbackKey))}</p>
    <details class="technical-details advanced-only">
      <summary>${escapeHtml(t("toolkit.technicalDetails"))}</summary>
      <div class="result-section">
        <div class="result-section-title">${escapeHtml(t("errors.technicalContext", { context }))}</div>
        <pre class="technical-error mono">${escapeHtml(rawErrorText(error))}</pre>
      </div>
    </details>`;
}

function showOperationError(error, context, fallbackKey = "errors.operationFailed") {
  const friendly = userErrorMessage(error, fallbackKey);
  const message = document.documentElement.classList.contains("show-advanced")
    ? `${friendly} ${t("errors.technicalContext", { context })} · ${t("errors.rawDetail")}: ${rawErrorText(error).slice(0, 280)}`
    : friendly;
  console.error(`[ZeroTick:${context}]`, error);
  showToast(message, true);
}

function collectSettingsPayload() {
  return {
    transient_threshold_ms: Number($("set-threshold").value),
    tray_recovery_secs: Number($("set-tray-recovery").value),
    max_history_entries: Number($("set-history-max").value),
    timeline_display_max: Number($("set-timeline-max").value),
    timeline_order: $("timeline-order").value === "asc" ? "asc" : "desc",
    bluetooth_poll_secs: Number($("set-bluetooth-poll").value),
    full_scan_timeout_secs: Number($("set-full-scan-timeout").value),
    system_query_timeout_secs: Number($("set-system-query-timeout").value),
    network_test_timeout_secs: Number($("set-network-test-timeout").value),
    bsod_debugger_timeout_secs: Number($("set-bsod-debugger-timeout").value),
    native_notifications: $("set-native-notify").checked,
    launch_at_startup: $("set-launch-startup").checked,
    run_as_admin: $("set-run-as-admin").checked,
    advanced_display: $("set-advanced-display").checked,
    close_to_tray: $("set-close-tray").value === "1",
    locale: $("set-locale").value,
    locale_auto_configured: Boolean(appSettings.locale_auto_configured),
  };
}

function saveSettingsImmediately() {
  if (!settingsLoaded || !$("settings-form").checkValidity()) return;
  const payload = collectSettingsPayload();
  settingsSaveChain = settingsSaveChain
    .catch(() => {})
    .then(async () => {
      const previous = appSettings;
      const shouldElevate = !previous.run_as_admin && payload.run_as_admin;
      try {
        const saved = await invoke("save_settings", { settings: payload });
        const localeChanged = normalizeLocale(previous.locale) !== normalizeLocale(saved.locale);
        const timelineLimitChanged = previous.timeline_display_max !== saved.timeline_display_max;
        const timelineOrderChanged = previous.timeline_order !== saved.timeline_order;
        const advancedDisplayChanged = previous.advanced_display !== saved.advanced_display;
        appSettings = { ...appSettings, ...saved };
        window.__zerotickSettings = appSettings;
        if (advancedDisplayChanged) {
          document.documentElement.classList.toggle("show-advanced", Boolean(saved.advanced_display));
        }
        if (localeChanged || timelineOrderChanged) {
          setLocale(saved.locale);
          await Promise.all([syncWindowTitle(), reloadTimeline()]);
        } else if (timelineLimitChanged) {
          trimTimeline();
        }
        if (shouldElevate) {
          const elevated = await getElevatedState();
          if (!elevated) {
            try {
              await invoke("restart_elevated");
            } catch (err) {
              showOperationError(err, "restart_elevated", "toast.elevateFailed");
            }
          }
        }
      } catch (err) {
        showOperationError(err, "save_settings", "toast.saveFailed");
      }
    });
}

function scheduleSettingsSave(immediate = false) {
  if (!settingsLoaded) return;
  clearTimeout(settingsSaveTimer);
  if (immediate) {
    saveSettingsImmediately();
  } else {
    settingsSaveTimer = setTimeout(saveSettingsImmediately, 450);
  }
}

function getElevatedState() {
  elevatedStatePromise ??= invoke("is_elevated").catch((error) => {
    elevatedStatePromise = null;
    throw error;
  });
  return elevatedStatePromise;
}

function renderLanguageList() {
  const list = $("language-list");
  if (!list) return;
  const current = getLocale();
  const currentOption = LANGUAGE_CATALOG.find((locale) => locale.code === current);
  const currentLabel = $("current-language-label");
  if (currentLabel) currentLabel.textContent = currentOption?.label ?? current;
  list.innerHTML = LANGUAGE_CATALOG.map((locale) => {
    const active = locale.code === current;
    return `<div class="language-row${active ? " is-active" : ""}">
      <button class="language-choice" type="button" data-code="${escapeHtml(locale.code)}" aria-pressed="${active}">
        <span>${escapeHtml(locale.label)}</span>
      </button>
      <span class="language-installed" aria-hidden="true">✓</span>
    </div>`;
  }).join("");
}

function setLanguagePopover(open) {
  const popover = $("language-popover");
  const trigger = $("language-picker-trigger");
  if (!popover || !trigger) return;
  popover.classList.toggle("hidden", !open);
  trigger.setAttribute("aria-expanded", String(open));
}

function chooseInstalledLanguage(code) {
  if (!isLocaleInstalled(code)) return;
  appSettings.locale_auto_configured = true;
  $("set-locale").value = code;
  setLocale(code);
  refreshStaticPanels();
  renderLanguageList();
  setLanguagePopover(false);
  saveSettingsImmediately();
}

function preferredSystemLocale() {
  for (const locale of navigator.languages ?? [navigator.language]) {
    const matched = matchSupportedLocale(locale);
    if (matched !== "en" || String(locale).toLowerCase().startsWith("en")) return matched;
  }
  return "en";
}

function configureInitialLanguage() {
  if (appSettings.locale_auto_configured) return;
  const saved = matchSupportedLocale(appSettings.locale);
  const target = saved !== "en" ? saved : preferredSystemLocale();
  appSettings.locale_auto_configured = true;
  $("set-locale").value = target;
  setLocale(target);
  refreshStaticPanels();
  saveSettingsImmediately();
}

function bindLanguagePicker() {
  $("language-picker-trigger").addEventListener("click", () => {
    const open = $("language-picker-trigger").getAttribute("aria-expanded") !== "true";
    setLanguagePopover(open);
  });
  $("language-list").addEventListener("click", (event) => {
    const choice = event.target.closest(".language-choice");
    if (choice) chooseInstalledLanguage(choice.dataset.code);
  });
  document.addEventListener("click", (event) => {
    if (!event.target.closest(".language-picker-control")) setLanguagePopover(false);
  });
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape" || $("language-popover").classList.contains("hidden")) return;
    setLanguagePopover(false);
    $("language-picker-trigger").focus();
  });
}

function renderUpdateStatus() {
  const status = $("update-status");
  if (!status) return;
  if (updateCheckState === "checking") {
    status.textContent = t("updates.checking");
    return;
  }
  if (updateCheckState === "failed") {
    status.textContent = t("updates.failed");
    return;
  }
  if (!updateInfo) {
    status.textContent = t("updates.notChecked");
    return;
  }
  status.textContent = updateInfo.update_available
    ? t("updates.available", { version: updateInfo.latest_version })
    : t("updates.current", { version: updateInfo.current_version });
}

async function openProjectUrl(url) {
  try {
    await invoke("open_project_url", { url });
  } catch (error) {
    showOperationError(error, "open_project_url");
  }
}

async function checkForUpdates({ force = false, reveal = false } = {}) {
  const button = $("btn-check-update");
  updateCheckState = "checking";
  button.disabled = true;
  button.setAttribute("aria-busy", "true");
  renderUpdateStatus();
  try {
    updateInfo = await invoke("check_for_updates", { force });
    updateCheckState = "ready";
    $("btn-release-page").classList.remove("hidden");
    $("btn-download-update").classList.toggle(
      "hidden",
      !updateInfo.update_available || !updateInfo.download_url,
    );
    renderUpdateStatus();
    const message = updateInfo.update_available
      ? t("updates.available", { version: updateInfo.latest_version })
      : t("updates.current", { version: updateInfo.current_version });
    showToast(message, false);
    if (reveal && updateInfo.update_available) switchPage("settings");
    return updateInfo;
  } catch (error) {
    updateCheckState = "failed";
    renderUpdateStatus();
    showOperationError(error, "check_for_updates", "updates.failed");
    return null;
  } finally {
    button.disabled = false;
    button.removeAttribute("aria-busy");
  }
}

async function refreshAdminHint() {
  const hint = $("repair-admin-hint");
  try {
    const elevated = await getElevatedState();
    if (elevated) {
      hint.classList.add("hidden");
      hint.textContent = "";
    } else {
      hint.classList.remove("hidden");
      hint.textContent = t("diag.repair.adminHint");
    }
  } catch {
    hint.classList.add("hidden");
  }
}

async function bindEvents() {
  await listen("tray-status", ({ payload: ev }) => applyTrayStatus(ev));
  applyTrayStatus({ level: "normal", reason_id: "normal" });

  await listen("device-event", ({ payload: ev }) => {
    appendTimeline(ev);
    if (ev.event_type === "transient_reconnect") {
      onTransientEvent(ev);
    } else if (ev.event_type === "remove") {
      const name = document.documentElement.classList.contains("show-advanced")
        ? advancedDeviceName(ev)
        : userDeviceName(ev);
      showToast(t("toast.disconnected", { name }), false);
    }
  });

  await listen("bluetooth-status", ({ payload: ev }) => {
    renderBluetoothReport(ev);
    if (!ev.healthy) {
      const msg = ev.issues?.[0] ? formatBluetoothIssue(ev.issues[0]) : t("toolkit.unknown");
      showToast(t("toast.bluetooth", { msg }), true);
    }
  });

  await listen("bsod-alert", ({ payload: ev }) => {
    renderBsodCard(ev);
    if (ev.is_recent) {
      showToast(
        document.documentElement.classList.contains("show-advanced")
          ? t("toast.bsod", { code: ev.bugcheck_code ?? t("diag.bluetooth.unknown") })
          : t("toast.bsodSimple"),
        true,
      );
    }
  });

  await listen("full-scan-progress", ({ payload }) => {
    applyFullScanProgress(payload);
  });

}

async function scanPorts(force = false) {
  if (portsScanPromise) return portsScanPromise;
  if (portsLoaded && !force) return;
  $("port-list").innerHTML = `<li class="empty-state">${escapeHtml(t("ports.scanning"))}</li>`;
  portsScanPromise = (async () => {
    try {
      renderPortScan(await invoke("scan_ports"));
      portsLoaded = true;
    } catch (e) {
      $("port-list").innerHTML = `<li class="empty-state">${renderOperationError(e, "scan_ports")}</li>`;
    } finally {
      portsScanPromise = null;
    }
  })();
  return portsScanPromise;
}

async function releasePort(pid) {
  try {
    await invoke("release_port", { pid });
    showToast(
      document.documentElement.classList.contains("show-advanced")
        ? t("toast.pidKilled", { pid })
        : t("toast.processEnded"),
      false,
    );
    await scanPorts(true);
  } catch (e) {
    showOperationError(e, "release_port");
  }
}

async function releaseAllPorts() {
  try {
    const r = await invoke("release_releasable_ports");
    const n = r.released_pids?.length ?? 0;
    if (n > 0) {
      showToast(t("toast.releasedN", { n }), false);
    } else if (r.errors?.length) {
      showOperationError(r.errors[0], "release_releasable_ports");
    } else {
      showToast(t("toast.nothingToRelease"), false);
    }
    await scanPorts(true);
  } catch (e) {
    showOperationError(e, "release_releasable_ports");
  }
}

function bindNavigation() {
  navItems = [...document.querySelectorAll(".nav-item")];
  pages = [...document.querySelectorAll(".page")];
  activePageId = document.querySelector(".nav-item.active")?.dataset.page ?? null;
  navItems.forEach((btn) => {
    btn.addEventListener("click", () => switchPage(btn.dataset.page));
  });
}

const SPIN_SVG_UP =
  '<svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path d="M2 6.5 5 3.5 8 6.5" stroke-linecap="round" stroke-linejoin="round"/></svg>';
const SPIN_SVG_DOWN =
  '<svg viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path d="M2 3.5 5 6.5 8 3.5" stroke-linecap="round" stroke-linejoin="round"/></svg>';

function clampNumberInput(input, value) {
  const min = input.min !== "" ? Number(input.min) : -Infinity;
  const max = input.max !== "" ? Number(input.max) : Infinity;
  return Math.min(max, Math.max(min, value));
}

function adjustNumberInput(input, delta) {
  const current = Number(input.value);
  const base = Number.isFinite(current) ? current : 0;
  input.value = String(clampNumberInput(input, base + delta));
  input.dispatchEvent(new Event("input", { bubbles: true }));
}

function refreshSpinLabels(root = document) {
  root.querySelectorAll('.number-spin-btn[data-delta="1"]').forEach((btn) => {
    btn.setAttribute("aria-label", t("spin.increase"));
  });
  root.querySelectorAll('.number-spin-btn[data-delta="-1"]').forEach((btn) => {
    btn.setAttribute("aria-label", t("spin.decrease"));
  });
}

/** 聚焦时显示叠加微调按钮；Ctrl+↑/↓ 以 1 为单位增减 */
function initNumberInputs(root = document) {
  root.querySelectorAll('input[type="number"]').forEach((input) => {
    if (input.closest(".number-input-wrap")) return;

    const wrap = document.createElement("div");
    wrap.className = "number-input-wrap";
    input.replaceWith(wrap);
    wrap.appendChild(input);

    const spin = document.createElement("div");
    spin.className = "number-spin";
    spin.innerHTML = `
      <button type="button" class="number-spin-btn" data-delta="1" aria-label="${escapeHtml(t("spin.increase"))}">${SPIN_SVG_UP}</button>
      <button type="button" class="number-spin-btn" data-delta="-1" aria-label="${escapeHtml(t("spin.decrease"))}">${SPIN_SVG_DOWN}</button>
    `;
    wrap.appendChild(spin);

    spin.querySelectorAll(".number-spin-btn").forEach((btn) => {
      btn.addEventListener("mousedown", (e) => e.preventDefault());
      btn.addEventListener("click", (e) => {
        const dir = Number(btn.dataset.delta);
        const step = e.ctrlKey ? 1 : Number(input.step) || 1;
        adjustNumberInput(input, dir * step);
        input.focus();
      });
    });

    input.addEventListener("keydown", (e) => {
      if (!e.ctrlKey || e.shiftKey || e.altKey) return;
      if (e.key === "ArrowUp") {
        e.preventDefault();
        adjustNumberInput(input, 1);
      } else if (e.key === "ArrowDown") {
        e.preventDefault();
        adjustNumberInput(input, -1);
      }
    });
  });
}

/** 滚动时短暂显示滚动条，停止后渐隐 */
function initAutoHideScrollbar(el) {
  if (!el || el.dataset.scrollInit === "1") return;
  el.dataset.scrollInit = "1";
  let timer = null;
  el.addEventListener(
    "scroll",
    () => {
      el.classList.add("is-scrolling");
      clearTimeout(timer);
      timer = setTimeout(() => el.classList.remove("is-scrolling"), 500);
    },
    { passive: true },
  );
}

function initScrollPanels() {
  const selector =
    ".settings-layout, .event-list, .port-table, .diag-grid, .card-fill > .result-panel";
  document.querySelectorAll(selector).forEach(initAutoHideScrollbar);
}

async function init() {
  bindLanguagePicker();
  bindNavigation();
  bindToolkitHandlers({ showToast });
  $("set-locale").value = "en";
  setLocale($("set-locale").value);
  renderLanguageList();
  enhanceSelectMenus();
  refreshSelectMenus();
  initNumberInputs($("settings-form"));
  initScrollPanels();
  updateTimelineEmpty();

  try {
    await waitForTauri();
    await bindEvents();
    const version = await invoke("get_app_version");
    appVersion = version;
    await loadSettings();
    await configureInitialLanguage();
    $("app-version").textContent = `v${version}`;
    $("settings-app-version").textContent = `v${version}`;
    await Promise.all([syncWindowTitle(), loadHistory()]);
    trimTimeline();
  } catch (e) {
    console.error("[ZeroTick:init]", e);
    const message = userErrorMessage(e, "errors.operationFailed");
    setStatus(
      document.documentElement.classList.contains("show-advanced")
        ? `${message} ${t("errors.rawDetail")}: ${rawErrorText(e).slice(0, 180)}`
        : message,
      "crit",
    );
  }

  $("settings-form").addEventListener("submit", (e) => {
    e.preventDefault();
  });
  $("set-advanced-display").addEventListener("change", (event) => {
    document.documentElement.classList.toggle("show-advanced", event.currentTarget.checked);
  });
  $("settings-form").addEventListener("input", (event) => {
    if (event.target.matches('input[type="number"]')) scheduleSettingsSave(false);
  });
  $("settings-form").addEventListener("change", () => scheduleSettingsSave(true));
  $("timeline-order").addEventListener("change", saveSettingsImmediately);

  $("btn-check-update").addEventListener("click", () => checkForUpdates({ force: true }));
  $("app-version").addEventListener("click", () => checkForUpdates({ force: false, reveal: true }));
  $("btn-download-update").addEventListener("click", () => {
    if (updateInfo?.download_url) openProjectUrl(updateInfo.download_url);
  });
  $("btn-release-page").addEventListener("click", () => {
    if (updateInfo?.release_url) openProjectUrl(updateInfo.release_url);
  });
  document.querySelectorAll(".btn-project-link").forEach((button) => {
    button.addEventListener("click", () => openProjectUrl(button.dataset.url));
  });
  $("btn-clear-history").addEventListener("click", async () => {
    try {
      await invoke("clear_device_history");
      $("timeline").innerHTML = "";
      updateTimelineEmpty();
      showToast(t("toast.historyCleared"), false);
    } catch (e) {
      showOperationError(e, "clear_device_history", "toast.clearFailed");
    }
  });

  async function exportHistory(format) {
    try {
      const path = await invoke("export_device_history", { format });
      showToast(
        document.documentElement.classList.contains("show-advanced")
          ? t("toast.exported", { path })
          : t("toast.exportComplete"),
        false,
      );
    } catch (e) {
      const msg = String(e);
      if (msg !== t("errors.exportCancelled")) {
        showOperationError(e, "export_device_history");
      }
    }
  }

  $("btn-export-json").addEventListener("click", () => exportHistory("json"));
  $("btn-export-csv").addEventListener("click", () => exportHistory("csv"));

  $("btn-scan-ports").addEventListener("click", () => scanPorts(true));
  $("btn-release-all-ports").addEventListener("click", () => releaseAllPorts());
  $("port-list").addEventListener("click", (e) => {
    const btn = e.target.closest(".btn-port-release");
    if (!btn) return;
    const pid = Number(btn.dataset.pid);
    if (pid) releasePort(pid);
  });

  $("btn-bsod")?.addEventListener("click", async () => {
    setResultPanel($("bsod-result"), "loading", `<p class="result-empty">${escapeHtml(t("diag.bsod.loading"))}</p>`);
    try {
      const r = await invoke("scan_bsod");
      renderBsodCard(r);
    } catch (e) {
      setResultPanel($("bsod-result"), "crit", renderOperationError(e, "scan_bsod"));
    }
  });

  $("bsod-result")?.addEventListener("click", async (event) => {
    const button = event.target.closest(".btn-bsod-repair");
    if (!button) return;
    const fixIds = (button.dataset.fixes ?? "").split(",").filter(Boolean);
    const results = button.parentElement?.querySelector(".bsod-repair-results");
    button.disabled = true;
    button.textContent = t("diag.bsod.repairing");
    if (results) results.innerHTML = `<p class="result-muted">${escapeHtml(t("diag.bsod.repairingHint"))}</p>`;
    try {
      const messages = await invoke("apply_bsod_repairs", { fixIds });
      const failed = messages.some((message) => String(message).trim().startsWith("✗"));
      if (results) results.innerHTML = `<p class="repair-verdict ${failed ? "warn" : "ok"}">${escapeHtml(t(failed ? "toolkit.repairResult.needsAttention" : "diag.bsod.repairComplete"))}</p><div class="advanced-only">${renderList(messages, t("diag.bsod.noRepairResults"))}</div>`;
      showToast(t("diag.bsod.repairComplete"), false);
    } catch (error) {
      const fallback = String(error) === "bsod_repair:admin_required"
        ? "diag.bsod.adminRequired"
        : "errors.operationFailed";
      if (results) results.innerHTML = renderOperationError(error, "apply_bsod_repairs", fallback);
      showOperationError(error, "apply_bsod_repairs", fallback);
    } finally {
      button.disabled = false;
      button.textContent = t("diag.bsod.runRepairs");
    }
  });

  $("btn-repair")?.addEventListener("click", async (event) => {
    const button = event.currentTarget;
    if (button.disabled) return;
    button.disabled = true;
    button.setAttribute("aria-busy", "true");
    setResultPanel($("repair-result"), "loading", `<p class="result-empty">${escapeHtml(t("diag.repair.loading"))}</p>`);
    await new Promise((resolve) => requestAnimationFrame(() => resolve()));
    try {
      const r = await invoke("run_repair");
      const result = { ...r, timestamp: new Date().toISOString() };
      renderRepairCard(result);
      showToast(formatRepairSummary(result), !result.success);
    } catch (e) {
      setResultPanel($("repair-result"), "crit", renderOperationError(e, "run_repair", "toast.repairFailed"));
      showOperationError(e, "run_repair", "toast.repairFailed");
    } finally {
      button.disabled = false;
      button.removeAttribute("aria-busy");
    }
  });
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => init());
} else {
  init();
}
