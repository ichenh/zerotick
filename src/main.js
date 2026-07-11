import "./styles.css";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import {
  t,
  setLocale,
  applyDom,
  getPageMeta,
  formatTime,
  fillLocaleSelect,
  normalizeLocale,
} from "./i18n.js";

const $ = (id) => document.getElementById(id);
let toastTimer = null;
let knownDevPort = null;
let appSettings = {
  transient_threshold_ms: 500,
  tray_recovery_secs: 45,
  max_history_entries: 500,
  timeline_display_max: 80,
  native_notifications: true,
  launch_at_startup: false,
  bluetooth_poll_secs: 60,
  locale: "zh-CN",
};

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
  document.querySelectorAll(".nav-item").forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.page === pageId);
  });
  document.querySelectorAll(".page").forEach((page) => {
    page.classList.toggle("active", page.id === `page-${pageId}`);
  });
  const meta = getPageMeta(pageId);
  if (meta) {
    $("page-title").textContent = meta.title;
    $("page-desc").textContent = meta.desc;
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
  const labelKey = level === "critical" ? "tray.critical" : level === "warning" ? "tray.warning" : "tray.normal";
  const reasonKey = ev.reason_id ? `tray.reason.${ev.reason_id}` : null;
  const reason = reasonKey ? t(reasonKey) : ev.message ?? "";
  setStatus(`${t(labelKey)} · ${reason}`, cls);
}

function categoryLabel(code) {
  const key = `events.category.${code}`;
  const label = t(key);
  return label !== key ? label : code;
}

function formatEventMessage(ev) {
  const name = ev.friendly_name || ev.vid_pid || t("events.unknownDevice");
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
  return ev.message;
}

function formatBluetoothIssue(issue) {
  if (typeof issue === "string") return issue;
  if (!issue?.id) return "";
  const key = `diag.bluetooth.issue.${issue.id}`;
  const text = t(key, {
    name: issue.name ?? "—",
    state: issue.state ?? "—",
    code: issue.code ?? "—",
  });
  return text !== key ? text : issue.id;
}

function formatRepairSummary(ev) {
  if (ev.summary_id) {
    const key = `diag.repair.summary.${ev.summary_id}`;
    const text = t(key, { n: ev.summary_count ?? 0 });
    if (text !== key) return text;
  }
  return ev.summary ?? "";
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

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
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
    ["bluetooth-result", "diag.bluetooth.idle"],
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

  const active = document.querySelector(".nav-item.active");
  if (active?.dataset.page) {
    switchPage(active.dataset.page);
  }
}

function updatePortsHint(port) {
  if (port != null) knownDevPort = port;
  const p = port ?? knownDevPort;
  const hint = $("ports-hint");
  if (hint && p != null) {
    hint.textContent = t("ports.hint", { port: p });
  }
}

function renderBluetoothCard(ev) {
  const el = $("bluetooth-result");
  const state = ev.healthy ? "ok" : "warn";
  const statusText = ev.healthy ? t("diag.bluetooth.ok") : t("diag.bluetooth.warn");
  const issues = renderList(
    ev.issues.map((i) => formatBluetoothIssue(i)),
    t("diag.bluetooth.noIssues"),
  );

  setResultPanel(
    el,
    state,
    `
    <div class="result-head">
      <span class="result-badge result-badge-${state}">${statusText}</span>
      <span class="result-time">${formatTime(ev.timestamp)}</span>
    </div>
    <div class="result-row"><span class="result-label">bthserv</span><span class="result-value">${escapeHtml(ev.bthserv_state ?? t("diag.bluetooth.unknown"))}</span></div>
    <div class="result-row"><span class="result-label">${escapeHtml(t("diag.bluetooth.radio"))}</span><span class="result-value">${escapeHtml(t("diag.bluetooth.radioCount", { n: ev.radio_count }))}</span></div>
    <div class="result-section">
      <div class="result-section-title">${escapeHtml(t("diag.bluetooth.issues"))}</div>
      ${issues}
    </div>
  `,
  );
}

function renderBsodCard(ev) {
  const el = $("bsod-result");
  if (!ev) {
    setResultPanel(el, "ok", `<p class="result-empty">${escapeHtml(t("diag.bsod.none"))}</p>`);
    return;
  }

  const state = ev.is_recent ? "crit" : "ok";
  const recent = ev.is_recent ? t("diag.bsod.recent") : t("diag.bsod.history");

  setResultPanel(
    el,
    state,
    `
    <div class="result-head">
      <span class="result-badge result-badge-${state}">${recent}</span>
      <span class="result-time">${formatTime(ev.timestamp)}</span>
    </div>
    <div class="result-row"><span class="result-label">${escapeHtml(t("diag.bsod.bugcheck"))}</span><span class="result-value mono">${escapeHtml(ev.bugcheck_code ?? "—")}</span></div>
    <div class="result-row"><span class="result-label">${escapeHtml(t("diag.bsod.driver"))}</span><span class="result-value mono">${escapeHtml(ev.faulting_driver ?? "—")}</span></div>
    <div class="result-row"><span class="result-label">${escapeHtml(t("diag.bsod.dumpPath"))}</span><span class="result-value mono" title="${escapeHtml(ev.dump_path)}">${escapeHtml(ev.dump_path)}</span></div>
    ${ev.message ? `<p class="result-msg">${escapeHtml(ev.message)}</p>` : ""}
  `,
  );
}

function renderRepairCard(ev) {
  const el = $("repair-result");
  let state = "ok";
  if (!ev.success) state = ev.needs_admin ? "warn" : "crit";
  else if (ev.usb_power_warnings?.length) state = "warn";

  const restarted = renderList(ev.services_restarted, t("diag.repair.noneRestarted"));
  const errors = ev.service_errors?.length ? renderList(ev.service_errors, "") : "";
  const usb = ev.usb_power_warnings?.length
    ? renderList(ev.usb_power_warnings, "")
    : `<p class="result-muted">${escapeHtml(t("diag.repair.noUsbWarn"))}</p>`;

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
      <div class="result-section-title">${escapeHtml(t("diag.repair.restarted"))}</div>
      ${restarted}
    </div>
    ${errors ? `<div class="result-section"><div class="result-section-title">${escapeHtml(t("diag.repair.failed"))}</div>${errors}</div>` : ""}
    <div class="result-section">
      <div class="result-section-title">${escapeHtml(t("diag.repair.usbScan"))}</div>
      ${usb}
      ${ev.power_scan_error ? `<p class="result-msg warn">${escapeHtml(ev.power_scan_error)}</p>` : ""}
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

  const name = ev.friendly_name || ev.vid_pid || t("events.unknownDevice");
  const metaParts = [categoryLabel(ev.category)];
  if (ev.vid_pid) metaParts.push(ev.vid_pid);
  if (ev.disconnect_ms != null) metaParts.push(`${ev.disconnect_ms}ms`);

  li.innerHTML = `
    <span class="tl-time">${formatTime(ev.timestamp)}</span>
    <span class="tl-badge tl-badge-${type}">${eventBadge(ev.event_type)}</span>
    <div class="tl-body">
      <div class="tl-name">${escapeHtml(name)}</div>
      <div class="tl-meta">${escapeHtml(metaParts.join(" · "))}</div>
      <div class="tl-msg">${escapeHtml(formatEventMessage(ev))}</div>
    </div>
  `;
  li.title = ev.device_path || "";

  const list = $("timeline");
  list.prepend(li);
  if (animate) {
    requestAnimationFrame(() => li.classList.add("visible"));
  } else {
    li.classList.add("visible");
  }

  while (list.children.length > appSettings.timeline_display_max) {
    list.removeChild(list.lastChild);
  }
  updateTimelineEmpty();
}

function trimTimeline() {
  const list = $("timeline");
  while (list.children.length > appSettings.timeline_display_max) {
    list.removeChild(list.lastChild);
  }
  updateTimelineEmpty();
}

function eventBadge(type) {
  if (type === "transient_reconnect") return t("events.transient");
  if (type === "remove") return t("events.remove");
  return t("events.arrival");
}

async function onTransientEvent(ev) {
  const name = ev.friendly_name || ev.vid_pid || t("events.device");
  showToast(t("toast.transient", { name, ms: ev.disconnect_ms ?? "?" }), true);
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
    for (let i = items.length - 1; i >= 0; i -= 1) {
      appendTimeline(items[i], { animate: false });
    }
  } catch {
    /* 历史加载失败时忽略 */
  }
}

async function loadSettings() {
  try {
    const s = await invoke("get_settings");
    appSettings = { ...appSettings, ...s };
    $("set-threshold").value = s.transient_threshold_ms;
    $("set-tray-recovery").value = s.tray_recovery_secs;
    $("set-history-max").value = s.max_history_entries;
    $("set-timeline-max").value = s.timeline_display_max;
    $("set-bluetooth-poll").value = s.bluetooth_poll_secs;
    $("set-native-notify").checked = Boolean(s.native_notifications);
    $("set-launch-startup").checked = Boolean(s.launch_at_startup);
    const localeEl = $("set-locale");
    if (s.locale) localeEl.value = s.locale;
    setLocale(s.locale);
    fillLocaleSelect(localeEl);
    if (s.locale) localeEl.value = normalizeLocale(s.locale);
  } catch {
    /* 使用默认值 */
  }
}

async function refreshAdminHint() {
  const hint = $("repair-admin-hint");
  try {
    const elevated = await invoke("is_elevated");
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

  await listen("device-event", ({ payload: ev }) => {
    appendTimeline(ev);
    if (ev.event_type === "transient_reconnect") {
      onTransientEvent(ev);
    } else if (ev.event_type === "remove") {
      const name = ev.friendly_name || ev.vid_pid || t("events.device");
      showToast(t("toast.disconnected", { name }), false);
    }
  });

  await listen("bluetooth-status", ({ payload: ev }) => {
    renderBluetoothCard(ev);
    if (!ev.healthy) {
      const msg = ev.issues?.[0] ? formatBluetoothIssue(ev.issues[0]) : t("diag.bluetooth.unknown");
      showToast(t("toast.bluetooth", { msg }), true);
    }
  });

  await listen("bsod-alert", ({ payload: ev }) => {
    renderBsodCard(ev);
    if (ev.is_recent) {
      showToast(t("toast.bsod", { code: ev.bugcheck_code ?? t("diag.bluetooth.unknown") }), true);
    }
  });

  await listen("repair-complete", ({ payload: ev }) => {
    renderRepairCard(ev);
    showToast(formatRepairSummary(ev), !ev.success);
  });

  setStatus(t("status.running"), "ok");
}

function renderPortScan(report) {
  updatePortsHint(report.dev_server_port);

  const rangesEl = $("excluded-ranges");
  if (report.excluded_ranges?.length) {
    const chips = report.excluded_ranges
      .map(
        (r) =>
          `<span class="reserved-chip${r.contains_dev_port ? " warn" : ""}">${r.start}–${r.end}</span>`,
      )
      .join("");
    rangesEl.innerHTML = `<div class="reserved-title">${escapeHtml(t("ports.reservedTitle"))}</div><div class="reserved-chips">${chips}</div>`;
  } else {
    rangesEl.innerHTML = "";
  }

  const list = $("port-list");
  if (!report.entries?.length) {
    list.innerHTML = `<li class="empty-state">${escapeHtml(t("ports.noListeners"))}</li>`;
  } else {
    list.innerHTML = report.entries.map((e) => renderPortRow(e)).join("");
  }

  const btn = $("btn-release-all-ports");
  const n = report.releasable_count ?? 0;
  btn.disabled = n === 0;
  btn.textContent = n > 0 ? t("ports.releaseAllN", { n }) : t("ports.releaseAll");
}

function renderPortRow(entry) {
  const pid = entry.pid ?? "—";
  const proc = entry.process_name ?? "—";
  const categoryLabel = t(`ports.category.${entry.category}`);
  const messageText = t(`ports.message.${entry.message_id}`, { state: entry.message });
  const releaseBtn =
    entry.can_release && entry.pid
      ? `<button class="btn btn-subtle btn-sm btn-port-release" data-pid="${entry.pid}" type="button">${escapeHtml(t("ports.releaseOne"))}</button>`
      : "";
  return `
    <li class="port-row">
      <span class="port-num">${entry.port}</span>
      <div class="port-info">
        <span class="port-badge port-badge-${entry.category}">${escapeHtml(categoryLabel)}</span>
        <div class="port-meta">${escapeHtml(entry.state)} · ${escapeHtml(proc)} · PID ${pid}</div>
        <div class="port-msg">${escapeHtml(messageText)}</div>
      </div>
      <div class="port-actions">${releaseBtn}</div>
    </li>`;
}

async function scanPorts() {
  $("port-list").innerHTML = `<li class="empty-state">${escapeHtml(t("ports.scanning"))}</li>`;
  try {
    const report = await invoke("scan_ports");
    renderPortScan(report);
  } catch (e) {
    $("port-list").innerHTML = `<li class="empty-state">${escapeHtml(String(e))}</li>`;
  }
}

async function releasePort(pid) {
  try {
    await invoke("release_port", { pid });
    showToast(t("toast.pidKilled", { pid }), false);
    await scanPorts();
  } catch (e) {
    showToast(String(e), true);
  }
}

async function releaseAllPorts() {
  try {
    const r = await invoke("release_releasable_ports");
    const n = r.released_pids?.length ?? 0;
    if (n > 0) {
      showToast(t("toast.releasedN", { n }), false);
    } else if (r.errors?.length) {
      showToast(r.errors[0], true);
    } else {
      showToast(t("toast.nothingToRelease"), false);
    }
    await scanPorts();
  } catch (e) {
    showToast(String(e), true);
  }
}

function bindNavigation() {
  document.querySelectorAll(".nav-item").forEach((btn) => {
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

/** 滚动时短暂显示滚动条 */
function initAutoHideScrollbar(el) {
  if (!el) return;
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

async function init() {
  bindNavigation();
  fillLocaleSelect($("set-locale"));
  setLocale(normalizeLocale(navigator.language));
  initNumberInputs($("settings-form"));
  initAutoHideScrollbar($("settings-form"));
  updateTimelineEmpty();

  $("set-locale").addEventListener("change", async () => {
    setLocale($("set-locale").value);
    applyDom();
    await syncWindowTitle();
    refreshStaticPanels();
    await reloadTimeline();
    if (knownDevPort != null) updatePortsHint(knownDevPort);
  });

  try {
    await waitForTauri();
    await bindEvents();
    const version = await invoke("get_app_version");
    $("app-version").textContent = `v${version}`;
    await loadSettings();
    await syncWindowTitle();
    await loadHistory();
    trimTimeline();
    await refreshAdminHint();
    await scanPorts();
  } catch (e) {
    setStatus(t("status.failed", { err: e }), "crit");
  }

  $("settings-form").addEventListener("submit", async (e) => {
    e.preventDefault();
    const payload = {
      transient_threshold_ms: Number($("set-threshold").value),
      tray_recovery_secs: Number($("set-tray-recovery").value),
      max_history_entries: Number($("set-history-max").value),
      timeline_display_max: Number($("set-timeline-max").value),
      bluetooth_poll_secs: Number($("set-bluetooth-poll").value),
      native_notifications: $("set-native-notify").checked,
      launch_at_startup: $("set-launch-startup").checked,
      locale: $("set-locale").value,
    };
    try {
      const saved = await invoke("save_settings", { settings: payload });
      appSettings = { ...appSettings, ...saved };
      setLocale(saved.locale);
      await syncWindowTitle();
      await reloadTimeline();
      showToast(t("toast.saved"), false);
    } catch (err) {
      showToast(t("toast.saveFailed", { err }), true);
    }
  });

  $("btn-clear-history").addEventListener("click", async () => {
    try {
      await invoke("clear_device_history");
      $("timeline").innerHTML = "";
      updateTimelineEmpty();
      showToast(t("toast.historyCleared"), false);
    } catch (e) {
      showToast(t("toast.clearFailed", { err: e }), true);
    }
  });

  async function exportHistory(format) {
    try {
      const path = await invoke("export_device_history", { format });
      showToast(t("toast.exported", { path }), false);
    } catch (e) {
      const msg = String(e);
      if (msg !== t("errors.exportCancelled")) {
        showToast(msg, true);
      }
    }
  }

  $("btn-export-json").addEventListener("click", () => exportHistory("json"));
  $("btn-export-csv").addEventListener("click", () => exportHistory("csv"));

  $("btn-scan-ports").addEventListener("click", () => scanPorts());
  $("btn-release-all-ports").addEventListener("click", () => releaseAllPorts());
  $("port-list").addEventListener("click", (e) => {
    const btn = e.target.closest(".btn-port-release");
    if (!btn) return;
    const pid = Number(btn.dataset.pid);
    if (pid) releasePort(pid);
  });

  $("btn-bluetooth").addEventListener("click", async () => {
    setResultPanel($("bluetooth-result"), "loading", `<p class="result-empty">${escapeHtml(t("diag.bluetooth.loading"))}</p>`);
    try {
      const r = await invoke("check_bluetooth");
      renderBluetoothCard(r);
    } catch (e) {
      setResultPanel($("bluetooth-result"), "crit", `<p class="result-msg warn">${escapeHtml(String(e))}</p>`);
    }
  });

  $("btn-bsod").addEventListener("click", async () => {
    setResultPanel($("bsod-result"), "loading", `<p class="result-empty">${escapeHtml(t("diag.bsod.loading"))}</p>`);
    try {
      const r = await invoke("scan_bsod");
      renderBsodCard(r);
    } catch (e) {
      setResultPanel($("bsod-result"), "crit", `<p class="result-msg warn">${escapeHtml(String(e))}</p>`);
    }
  });

  $("btn-repair").addEventListener("click", async () => {
    setResultPanel($("repair-result"), "loading", `<p class="result-empty">${escapeHtml(t("diag.repair.loading"))}</p>`);
    try {
      const r = await invoke("run_repair");
      renderRepairCard({ ...r, timestamp: new Date().toISOString() });
    } catch (e) {
      setResultPanel($("repair-result"), "crit", `<p class="result-msg warn">${escapeHtml(String(e))}</p>`);
      showToast(t("toast.repairFailed", { err: e }), true);
    }
  });
}

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", () => init());
} else {
  init();
}
