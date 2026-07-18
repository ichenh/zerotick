import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { t, formatDuration, formatTime } from "./i18n.js";
import { enhanceSelectMenus } from "./select-menu.js";

const $ = (id) => document.getElementById(id);

export function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
}

function isGenericHardwareName(name) {
  return /^(?:USB Composite Device|USB Mass Storage Device|USB Storage Device|USB device|Unknown USB Device|Unknown device|Generic(?:-\w+)?(?: USB)? Device|Disk drive|UAS Mass Storage Device)$/i.test(String(name ?? "").trim());
}

function setPanel(el, state, html) {
  el.className = `result-panel result-${state}`;
  el.innerHTML = html;
}

function renderList(items, emptyText) {
  if (!items?.length) {
    return `<p class="result-muted">${escapeHtml(emptyText)}</p>`;
  }
  return `<ul class="result-list">${items.map((i) => `<li>${escapeHtml(i)}</li>`).join("")}</ul>`;
}

function formatServiceLabel(labelId) {
  const key = `toolkit.services.label.${labelId}`;
  const text = t(key);
  return text !== key ? text : labelId;
}

function formatServiceIssue(issue) {
  const label = formatServiceLabel(issue.label_id);
  const key = `toolkit.services.issue.${issue.id}`;
  const text = t(key, {
    name: label,
    service: issue.service_name ?? "—",
    state: issue.state ?? "—",
  });
  return text !== key ? text : issue.id;
}

function formatServiceState(service) {
  if (!service) return t("toolkit.unknown");
  if (typeof service === "string") {
    if (service === "Running") return t("toolkit.ok");
    return service ? t("toolkit.warn") : t("toolkit.unknown");
  }
  if (service.expected_stopped) return t("toolkit.services.onDemand");
  if (service.state === "Running") return t("toolkit.ok");
  if (service.state) return t("toolkit.warn");
  return t("toolkit.unknown");
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

function renderOperationError(error, context, fallbackKey = "errors.detectionFailed", friendlyOverride = "") {
  const friendly = friendlyOverride || userErrorMessage(error, fallbackKey);
  return `<p class="result-msg warn">${escapeHtml(friendly)}</p>
    <details class="technical-details advanced-only">
      <summary>${escapeHtml(t("toolkit.technicalDetails"))}</summary>
      <div class="result-section">
        <div class="result-section-title">${escapeHtml(t("errors.technicalContext", { context }))}</div>
        <pre class="technical-error mono">${escapeHtml(rawErrorText(error))}</pre>
      </div>
    </details>`;
}

function notifyOperationError(showToast, error, context, fallbackKey = "errors.operationFailed") {
  const friendly = userErrorMessage(error, fallbackKey);
  const raw = rawErrorText(error);
  const message = document.documentElement.classList.contains("show-advanced")
    ? `${friendly} ${t("errors.technicalContext", { context })} · ${t("errors.rawDetail")}: ${raw.slice(0, 280)}`
    : friendly;
  console.error(`[ZeroTick:${context}]`, error);
  showToast(message, true);
}

async function runButtonTask(button, task) {
  if (!button || button.disabled) return;
  button.disabled = true;
  button.setAttribute("aria-busy", "true");
  await new Promise((resolve) => requestAnimationFrame(() => resolve()));
  try {
    await task();
  } finally {
    if (button.isConnected) {
      button.disabled = false;
      button.removeAttribute("aria-busy");
    }
  }
}

function renderServicesBlock(services, titleKey) {
  const rows = (services?.services ?? [])
    .map((svc) => {
      const badge = svc.state === "Running" || svc.expected_stopped ? "ok" : svc.state ? "crit" : "warn";
      return `<div class="result-row">
        <span class="result-label">${escapeHtml(formatServiceLabel(svc.label_id))}</span>
        <span class="result-value"><span class="result-badge result-badge-${badge}">${escapeHtml(formatServiceState(svc))}</span></span>
      </div>`;
    })
    .join("");
  const issues = renderList(
    (services?.issues ?? []).map(formatServiceIssue),
    t("toolkit.services.noIssues"),
  );
  return `<details class="technical-details advanced-only">
    <summary>${escapeHtml(t("toolkit.technicalDetails"))}</summary>
    <div class="result-section">
      <div class="result-section-title">${escapeHtml(t(titleKey))}</div>
      ${rows || `<p class="result-muted">${escapeHtml(t("toolkit.empty"))}</p>`}
    </div>
    <div class="result-section">
      <div class="result-section-title">${escapeHtml(t("toolkit.services.issues"))}</div>
      ${issues}
    </div></details>`;
}

function renderGuidance(level, titleKey, summaryKey, actionKeys = []) {
  const actions = actionKeys
    .map((key) => `<li>${escapeHtml(t(key))}</li>`)
    .join("");
  return `<section class="diagnosis-guide diagnosis-guide-${level}">
    <div class="diagnosis-guide-copy">
      <div class="diagnosis-guide-kicker">${escapeHtml(t(`toolkit.verdict.${level}`))}</div>
      <h3>${escapeHtml(t(titleKey))}</h3>
      <p>${escapeHtml(t(summaryKey))}</p>
    </div>
    ${actions ? `<div class="diagnosis-actions"><span>${escapeHtml(t("toolkit.recommendedActions"))}</span><ol>${actions}</ol></div>` : ""}
  </section>`;
}

function renderRepairBlock(result, remainingIssue = false) {
  if (!result) return "";
  const changed = (result.services_restarted?.length ?? 0) > 0;
  const restarted = changed ? renderList(result.services_restarted, "") : "";
  const errors = result.service_errors?.length ? renderList(result.service_errors, "") : "";
  const needsAttention = Boolean(errors || remainingIssue);
  const verdictKey = needsAttention
    ? "toolkit.repairResult.needsAttention"
    : changed
      ? "toolkit.repairResult.rechecked"
      : "toolkit.repairResult.noneRestarted";
  return `
    ${result.needs_admin ? `<p class="info-banner">${escapeHtml(t("toolkit.repairResult.adminBanner"))}</p>` : ""}
    <p class="repair-verdict ${needsAttention ? "warn" : "ok"}">${escapeHtml(t(verdictKey))}</p>
    ${changed ? `<div class="result-section">
      <div class="result-section-title">${escapeHtml(t("toolkit.repairResult.restarted"))}</div>
      ${restarted}
    </div>` : ""}
    ${errors ? `<div class="result-section advanced-only"><div class="result-section-title">${escapeHtml(t("toolkit.repairResult.failed"))}</div>${errors}</div>` : ""}`;
}

function showRepairOutcome(showToast, result, remainingIssue) {
  const hasErrors = (result.service_errors?.length ?? 0) > 0;
  const changed = (result.services_restarted?.length ?? 0) > 0;
  const failed = hasErrors || remainingIssue;
  const key = failed
    ? "toolkit.repairResult.needsAttention"
    : changed
      ? "toolkit.repairResult.rechecked"
      : "toolkit.repairResult.noneRestarted";
  showToast(t(key), failed);
}

function renderVpnBlock(vpn) {
  const active = Boolean(vpn?.active);
  const statusText = vpn?.tunnel_active
    ? t("toolkit.network.vpnActive")
    : vpn?.proxy?.active
      ? t("toolkit.network.proxyActive")
      : t("toolkit.network.vpnInactive");
  const badge = "ok";
  const connItems = (vpn?.connections ?? []).map(
    (c) => `${c.name}${c.server ? ` → ${c.server}` : ""} (${c.status})`,
  );
  const adapterItems = (vpn?.adapters ?? []).map((a) => `${a.name} — ${a.description}`);
  const proxyMode = vpn?.proxy?.active
    ? t(`toolkit.network.proxyMode.${vpn.proxy.mode || "combined"}`)
    : "";
  const proxySources = (vpn?.proxy?.sources ?? []).map((source) =>
    `${t(`toolkit.network.proxySource.${source.kind || "manual"}`)} · ${source.address}`,
  );
  const proxyProviders = (vpn?.proxy?.providers ?? []).map((provider) => {
    const evidence = t(`toolkit.network.proxyEvidence.${provider.evidence || "related_process"}`);
    const technical = [provider.pid ? `PID ${provider.pid}` : "", provider.path || ""].filter(Boolean).join(" · ");
    return `${escapeHtml(provider.name)} · ${escapeHtml(evidence)}${technical ? ` <span class="advanced-only advanced-inline mono">· ${escapeHtml(technical)}</span>` : ""}`;
  });
  const providerBlock = vpn?.proxy?.active
    ? proxyProviders.length
      ? `<div class="result-row"><span class="result-label">${escapeHtml(t("toolkit.network.proxyProvider"))}</span><span class="result-value"><ul class="result-list">${proxyProviders.map((item) => `<li>${item}</li>`).join("")}</ul></span></div>`
      : `<div class="result-row"><span class="result-label">${escapeHtml(t("toolkit.network.proxyProvider"))}</span><span class="result-value result-muted">${escapeHtml(t("toolkit.network.proxyProviderUnknown"))}</span></div>`
    : "";
  const hint = active
    ? `<p class="result-muted">${escapeHtml(t("toolkit.network.vpnActiveHint"))}</p>`
    : "";
  return `
    <div class="result-section">
      <div class="result-section-title">${escapeHtml(t("toolkit.network.vpnTitle"))}</div>
      <div class="result-row">
        <span class="result-label">${escapeHtml(t("toolkit.network.vpnStatus"))}</span>
        <span class="result-value"><span class="result-badge result-badge-${badge}">${escapeHtml(statusText)}</span></span>
      </div>
      ${hint}
      ${proxyMode ? `<div class="result-row"><span class="result-label">${escapeHtml(t("toolkit.network.proxy"))}</span><span class="result-value">${escapeHtml(proxyMode)}</span></div>` : ""}
      ${proxySources.length ? `<div class="result-row"><span class="result-label">${escapeHtml(t("toolkit.network.proxyAddress"))}</span><span class="result-value">${renderList(proxySources, "")}</span></div>` : ""}
      ${providerBlock}
      ${
        connItems.length
          ? `<div class="result-row"><span class="result-label">${escapeHtml(t("toolkit.network.vpnConnections"))}</span><span class="result-value">${renderList(connItems, "")}</span></div>`
          : ""
      }
      ${
        adapterItems.length
          ? `<div class="result-row"><span class="result-label">${escapeHtml(t("toolkit.network.vpnAdapters"))}</span><span class="result-value">${renderList(adapterItems, "")}</span></div>`
          : ""
      }
    </div>`;
}

function formatSpeedTestError(error) {
  const raw = String(error);
  if (!raw.startsWith("speed_test:")) return userErrorMessage(error, "toolkit.network.speedError.unknown");
  const code = raw.slice("speed_test:".length);
  const key = `toolkit.network.speedError.${code}`;
  const message = t(key);
  return message === key ? t("toolkit.network.speedError.unknown") : message;
}

function formatAudioModeError(error) {
  const raw = String(error);
  if (raw.includes("audio_mode:admin_required")) {
    return t("toolkit.audio.modeAdminRequired");
  }
  if (raw.includes("audio_mode:access_denied")) {
    return t("toolkit.audio.modeAccessDenied");
  }
  if (raw.includes("audio_mode:endpoint_not_found")) {
    return t("toolkit.audio.modeEndpointMissing");
  }
  if (raw.includes("audio_mode:verify_failed")) {
    return t("toolkit.audio.modeVerifyFailed");
  }
  if (raw.includes("audio_mode:property_store_unavailable") || raw.includes("audio_mode:interface_unavailable")) {
    return t("toolkit.audio.modeInterfaceUnavailable");
  }
  if (raw.includes("audio_mode:write_failed")) {
    return t("toolkit.audio.modeChangeFailed");
  }
  return userErrorMessage(error);
}

export function renderNetworkSpeedResult(result) {
  const el = $("network-result");
  const mbps = Number(result.speed_mbps ?? 0).toFixed(2);
  const kb = ((result.bytes ?? 0) / 1024).toFixed(1);
  const vpnNote = result.vpn_active
    ? `<p class="result-muted">${escapeHtml(t("toolkit.network.vpnActiveHint"))}</p>`
    : "";
  setPanel(
    el,
    "ok",
    `
    <div class="result-head">
      <span class="result-badge result-badge-ok">${escapeHtml(t("toolkit.network.speedDone"))}</span>
      <span class="result-time">${formatTime(new Date().toISOString())}</span>
    </div>
    <div class="result-row"><span class="result-label">${escapeHtml(t("toolkit.network.speedMbps"))}</span><span class="result-value mono">${escapeHtml(mbps)} Mbps</span></div>
    <div class="result-row advanced-only"><span class="result-label">${escapeHtml(t("toolkit.network.speedBytes"))}</span><span class="result-value mono">${escapeHtml(kb)} KB</span></div>
    <div class="result-row advanced-only"><span class="result-label">${escapeHtml(t("toolkit.network.speedDuration"))}</span><span class="result-value mono">${escapeHtml(formatDuration(result.duration_ms))}</span></div>
    ${vpnNote}
  `,
  );
}

export function renderNetworkReport(report) {
  const el = $("network-result");
  const healthy = !(report.services?.issues?.length);
  const gw = report.gateway ?? t("toolkit.unknown");
  const reach = report.gateway_reachable;
  const reachText =
    reach === true ? t("toolkit.network.reachable") : reach === false ? t("toolkit.network.unreachable") : "—";

  let guide = renderGuidance("ok", "toolkit.network.guide.okTitle", "toolkit.network.guide.okSummary");
  let state = "ok";
  if ((report.adapter_count ?? 0) === 0) {
    state = "crit";
    guide = renderGuidance("crit", "toolkit.network.guide.noAdapterTitle", "toolkit.network.guide.noAdapterSummary", ["toolkit.network.guide.noAdapterStep1", "toolkit.network.guide.noAdapterStep2", "toolkit.network.guide.noAdapterStep3"]);
  } else if (reach === false) {
    state = "warn";
    guide = renderGuidance("warn", "toolkit.network.guide.gatewayTitle", "toolkit.network.guide.gatewaySummary", ["toolkit.network.guide.gatewayStep1", "toolkit.network.guide.gatewayStep2", "toolkit.network.guide.gatewayStep3"]);
  } else if (!healthy) {
    state = "warn";
    guide = renderGuidance("warn", "toolkit.network.guide.serviceTitle", "toolkit.network.guide.serviceSummary", ["toolkit.network.guide.serviceStep1", "toolkit.network.guide.serviceStep2"]);
  }

  setPanel(
    el,
    state,
    `
    ${guide}
    <div class="result-head">
      <span class="result-badge result-badge-${state}">${escapeHtml(t(`toolkit.verdict.${state}`))}</span>
      <span class="result-time">${formatTime(new Date().toISOString())}</span>
    </div>
    <div class="result-row advanced-only"><span class="result-label">${escapeHtml(t("toolkit.network.gateway"))}</span><span class="result-value mono">${escapeHtml(gw)} · ${escapeHtml(reachText)}</span></div>
    <div class="result-row advanced-only"><span class="result-label">${escapeHtml(t("toolkit.network.adapters"))}</span><span class="result-value">${escapeHtml(String(report.adapter_count ?? 0))}</span></div>
    ${renderVpnBlock(report.vpn)}
    ${renderServicesBlock(report.services, "toolkit.network.servicesTitle")}
  `,
  );
}

const PLAYBACK_CATEGORY_ORDER = ["speakers", "headphones", "digital", "line", "other"];
const CAPTURE_CATEGORY_ORDER = ["microphone", "headset", "line", "other"];

function uniqueAudioDevices(devices) {
  const byId = new Map();
  for (const d of devices ?? []) {
    if (d?.id && !byId.has(d.id)) byId.set(d.id, d);
  }
  return [...byId.values()].sort(
    (a, b) =>
      Number(b.is_default) - Number(a.is_default)
      || String(a.name).localeCompare(String(b.name))
      || String(a.id).localeCompare(String(b.id)),
  );
}

function groupByCategory(devices, order) {
  const grouped = new Map();
  for (const d of uniqueAudioDevices(devices)) {
    const cat = d.category || "other";
    if (!grouped.has(cat)) grouped.set(cat, []);
    grouped.get(cat).push(d);
  }
  return order
    .filter((cat) => grouped.has(cat))
    .map((cat) => ({ category: cat, devices: grouped.get(cat) }));
}

function renderAudioDeviceRow(d) {
  const defaultMark = d.is_default
    ? `<span class="result-badge result-badge-ok">${escapeHtml(t("toolkit.audio.default"))}</span>`
    : "";
  const isPlayback = (d.kind || "playback") === "playback";
  const mode = d.mode || "shared";
  const allowsExclusive = mode === "exclusive" || mode === "exclusive_priority";
  const prioritizesExclusive = mode === "exclusive_priority";
  const volume = Number.isFinite(Number(d.volume_percent)) ? Math.min(100, Math.max(0, Number(d.volume_percent))) : null;
  const muted = Boolean(d.is_muted);
  const modeControl = isPlayback
    ? `<details class="audio-advanced">
        <summary>${escapeHtml(t("toolkit.audio.advanced"))}</summary>
        <div class="audio-mode-options">
          <label class="audio-option">
            <input class="audio-exclusive-toggle" type="checkbox" data-setting="allow" data-id="${escapeHtml(d.id)}" data-kind="${escapeHtml(d.kind || "playback")}"${allowsExclusive ? " checked" : ""} />
            <span><strong>${escapeHtml(t("toolkit.audio.allowExclusive"))}</strong><small>${escapeHtml(t("toolkit.audio.allowExclusiveHint"))}</small></span>
          </label>
          <label class="audio-option${allowsExclusive ? "" : " is-disabled"}">
            <input class="audio-exclusive-toggle" type="checkbox" data-setting="priority" data-id="${escapeHtml(d.id)}" data-kind="${escapeHtml(d.kind || "playback")}"${prioritizesExclusive ? " checked" : ""}${allowsExclusive ? "" : " disabled"} />
            <span><strong>${escapeHtml(t("toolkit.audio.exclusivePriority"))}</strong><small>${escapeHtml(t("toolkit.audio.exclusivePriorityHint"))}</small></span>
          </label>
          <p class="result-muted">${escapeHtml(t("toolkit.audio.modeAdminHint"))}</p>
        </div>
      </details>`
    : "";
  const volumeControl = volume === null
    ? `<span class="result-muted">${escapeHtml(t("toolkit.audio.volumeUnavailable"))}</span>`
    : `<div class="audio-volume-control">
        <span class="audio-control-label">${escapeHtml(t("toolkit.audio.volume"))}</span>
        <input class="audio-volume-slider" type="range" min="0" max="100" step="1" value="${volume}" data-id="${escapeHtml(d.id)}" aria-label="${escapeHtml(t("toolkit.audio.volume"))}" />
        <output class="audio-volume-value">${volume}%</output>
        <button type="button" class="btn btn-default btn-sm btn-audio-mute${muted ? " is-active" : ""}" data-id="${escapeHtml(d.id)}" data-muted="${muted}" aria-pressed="${muted}">${escapeHtml(t(muted ? "toolkit.audio.unmute" : "toolkit.audio.mute"))}</button>
      </div>`;
  return `
    <li class="device-row audio-device-row" data-device-id="${escapeHtml(d.id)}">
      <div class="audio-device-heading">
        <div class="audio-device-info">
          <div class="device-name">${escapeHtml(d.name)} ${defaultMark}</div>
        </div>
        <button type="button" class="btn btn-default btn-sm btn-audio-default" data-id="${escapeHtml(d.id)}" data-kind="${escapeHtml(d.kind || "playback")}"${d.is_default ? " disabled" : ""}>${escapeHtml(t("toolkit.audio.setDefault"))}</button>
      </div>
      <div class="result-muted mono advanced-only">Endpoint ID: ${escapeHtml(d.id)}</div>
      ${volumeControl}
      ${modeControl}
    </li>`;
}

function renderAudioCategoryGroups(groups, emptyKey) {
  if (!groups.length) {
    return `<p class="result-muted audio-empty">${escapeHtml(t(emptyKey))}</p>`;
  }
  return groups
    .map(
      (g) => `
    <div class="audio-category-group">
      <div class="audio-category-title">${escapeHtml(t(`toolkit.audio.category.${g.category}`))}</div>
      <ul class="device-list audio-category-list">${g.devices.map(renderAudioDeviceRow).join("")}</ul>
    </div>`,
    )
    .join("");
}

export function renderAudioReport(report) {
  const el = $("audio-result");
  const healthy = !(report.services?.issues?.length);
  const playbackGroups = groupByCategory(report.playback ?? report.devices?.filter((d) => d.kind !== "capture"), PLAYBACK_CATEGORY_ORDER);
  const captureGroups = groupByCategory(report.capture ?? report.devices?.filter((d) => d.kind === "capture"), CAPTURE_CATEGORY_ORDER);
  const deviceCount = (report.playback?.length ?? 0) + (report.capture?.length ?? 0);
  let state = healthy ? "ok" : "warn";
  let guide = healthy
    ? renderGuidance("ok", "toolkit.audio.guide.okTitle", "toolkit.audio.guide.okSummary")
    : renderGuidance("warn", "toolkit.audio.guide.serviceTitle", "toolkit.audio.guide.serviceSummary", ["toolkit.audio.guide.serviceStep1", "toolkit.audio.guide.serviceStep2"]);
  if (deviceCount === 0) {
    state = "crit";
    guide = renderGuidance("crit", "toolkit.audio.guide.noDeviceTitle", "toolkit.audio.guide.noDeviceSummary", ["toolkit.audio.guide.noDeviceStep1", "toolkit.audio.guide.noDeviceStep2", "toolkit.audio.guide.noDeviceStep3"]);
  }

  setPanel(
    el,
    state,
    `
    ${guide}
    <div class="result-head">
      <span class="result-badge result-badge-${state}">${escapeHtml(t(`toolkit.verdict.${state}`))}</span>
    </div>
    <div class="audio-io-grid">
      <section class="audio-io-section">
        <div class="audio-io-heading"><span>${escapeHtml(t("toolkit.audio.outputTitle"))}</span><span class="audio-device-count">${report.playback?.length ?? 0}</span></div>
        ${renderAudioCategoryGroups(playbackGroups, "toolkit.audio.noOutputDevices")}
      </section>
      <section class="audio-io-section">
        <div class="audio-io-heading"><span>${escapeHtml(t("toolkit.audio.inputTitle"))}</span><span class="audio-device-count">${report.capture?.length ?? 0}</span></div>
        ${renderAudioCategoryGroups(captureGroups, "toolkit.audio.noInputDevices")}
      </section>
    </div>
    <div class="audio-services">${renderServicesBlock(report.services, "toolkit.audio.servicesTitle")}</div>
  `,
  );
}

function usbSlotForVolume(device, volume) {
  const letter = volume.letter.replace(":", "").toUpperCase();
  return (device.slots ?? []).find((slot) =>
    (volume.disk_number != null && slot.disk_number === volume.disk_number)
      || (slot.volume_letters ?? []).some((slotLetter) => slotLetter.replace(":", "").toUpperCase() === letter),
  );
}

function normalizeUsbVolume(device, volume) {
  const slot = device.device_kind === "card_reader" ? usbSlotForVolume(device, volume) : null;
  return slot?.access_state === "no_media"
    ? { ...volume, access_state: "no_media" }
    : volume;
}

function renderUsbFormatButton(volume) {
  const letter = volume.letter.replace(":", "").toUpperCase();
  return `<button type="button" class="btn btn-danger-quiet btn-sm btn-usb-format" data-letter="${escapeHtml(letter)}" data-label="${escapeHtml(volume.label || "")}" data-filesystem="${escapeHtml(volume.filesystem || "")}" data-size="${escapeHtml(volume.size_gb?.toFixed(1) || "0.0")}">${escapeHtml(t("toolkit.usb.format", { letter }))}</button>`;
}

function usbVolumeDetail(volume) {
  if (volume.access_state !== "ready") return t(`toolkit.usb.access.${volume.access_state}`);
  const total = Math.max(0, Number(volume.size_gb) || 0);
  const free = Math.min(total, Math.max(0, Number(volume.free_gb) || 0));
  const used = Math.max(0, total - free);
  return t("toolkit.usb.capacitySummary", {
    label: volume.label,
    letter: volume.letter,
    filesystem: volume.filesystem,
    used: used.toFixed(1),
    total: total.toFixed(1),
    free: free.toFixed(1),
  });
}

function renderCardReaderSlots(device, drivesByLetter) {
  return (device.slots ?? []).map((slot, index) => {
    const letters = (slot.volume_letters ?? []).map((letter) => letter.replace(":", "").toUpperCase());
    const volumes = letters
      .map((letter) => drivesByLetter.get(letter))
      .filter(Boolean)
      .map((volume) => normalizeUsbVolume(device, volume));
    const readyVolume = volumes.find((volume) => volume.access_state === "ready");
    const blockedVolume = volumes.find((volume) => ["locked", "unavailable"].includes(volume.access_state));
    const state = slot.access_state === "no_media"
      ? "no_media"
      : blockedVolume?.access_state || (readyVolume ? "mounted" : slot.access_state || "unknown");
    const letter = letters[0] || "";
    const detail = state === "no_media"
      ? t("toolkit.usb.cardReader.emptySlotDetail", { letter: letter ? `${letter}:` : t("toolkit.usb.cardReader.noLetter") })
      : volumes.length
        ? volumes.map(usbVolumeDetail).join("\n")
        : t(`toolkit.usb.access.${state}Hint`) !== `toolkit.usb.access.${state}Hint`
          ? t(`toolkit.usb.access.${state}Hint`)
          : t("toolkit.usb.cardReader.slotIssueDetail");
    let actions = "";
    if (readyVolume) {
      const readyLetter = readyVolume.letter.replace(":", "").toUpperCase();
      actions = `<button type="button" class="btn btn-default btn-sm btn-usb-scan-lock" data-letter="${escapeHtml(readyLetter)}">${escapeHtml(t("toolkit.usb.scanLock"))}</button>
        ${renderUsbFormatButton(readyVolume)}
        <button type="button" class="btn btn-default btn-sm btn-usb-eject" data-letter="${escapeHtml(readyLetter)}">${escapeHtml(t("toolkit.usb.eject"))}</button>`;
    } else if (state === "locked" && letter) {
      actions = `<button type="button" class="btn btn-accent btn-sm btn-usb-unlock" data-letter="${escapeHtml(letter)}">${escapeHtml(t("toolkit.usb.unlock"))}</button>`;
    }
    return `<div class="card-reader-slot card-reader-slot-${state === "no_media" ? "empty" : blockedVolume ? "warn" : "ready"}">
      <div class="card-reader-slot-main">
        <div class="card-reader-slot-title">${escapeHtml(t("toolkit.usb.cardReader.slot", { n: index + 1 }))} <span class="result-badge result-badge-${state === "no_media" || readyVolume ? "ok" : "warn"}">${escapeHtml(state === "no_media" ? t("toolkit.usb.cardReader.emptySlot") : t(`toolkit.usb.access.${state}`))}</span></div>
        <div class="result-muted usb-volume-summary">${escapeHtml(detail)}</div>
        <div class="result-muted mono advanced-only">${escapeHtml(slot.instance_id || "")}${slot.disk_number != null ? ` · Disk ${escapeHtml(slot.disk_number)}` : ""}</div>
      </div>
      <div class="card-reader-slot-actions">${actions}</div>
      ${letter ? `<ul class="lock-list usb-lock-list" id="usb-locks-${escapeHtml(letter)}"></ul>` : ""}
    </div>`;
  }).join("");
}

export function renderUsbReport(report) {
  const el = $("usb-result");
  const completionHint = report.complete === false
    ? `<p class="result-muted">${escapeHtml(t("toolkit.loading"))}</p>`
    : "";
  const drivesByLetter = new Map((report.drives ?? []).map((drive) => [drive.letter.replace(":", "").toUpperCase(), drive]));
  const claimedLetters = new Set();
  const groups = (report.devices ?? []).map((device) => {
    const letters = (device.volume_letters ?? []).map((letter) => letter.replace(":", "").toUpperCase());
    letters.forEach((letter) => claimedLetters.add(letter));
    return {
      device,
      letters,
      volumes: letters.map((letter) => drivesByLetter.get(letter)).filter(Boolean).map((volume) => normalizeUsbVolume(device, volume)),
    };
  });
  for (const drive of report.drives ?? []) {
    const letter = drive.letter.replace(":", "").toUpperCase();
    if (!claimedLetters.has(letter)) {
      groups.push({
        device: { name: drive.label, access_state: drive.access_state || "ready", device_kind: "unknown", volume_letters: [drive.letter] },
        letters: [letter],
        volumes: [drive],
      });
    }
  }
  const groupRows = groups.map(({ device, letters, volumes }) => {
    const volumeState = volumes.find((volume) => ["locked", "unavailable"].includes(volume.access_state))?.access_state;
    // A readable volume is authoritative for what the user can actually do. Some
    // enclosures also expose an offline virtual-CD LUN, which must not make the
    // accessible data disk look offline.
    const hasReadyVolume = volumes.some((volume) => volume.access_state === "ready");
    const isCardReader = device.device_kind === "card_reader";
    const nonEmptyVolumes = volumes.filter((volume) => volume.access_state !== "no_media");
    const accessState = isCardReader
      ? (hasReadyVolume ? "mounted" : volumeState || device.access_state || "no_media")
      : volumeState || (hasReadyVolume ? "mounted" : device.access_state) || "unknown";
    const needsAccess = ["locked", "unavailable"].includes(accessState);
    const canPromptUnlock = accessState === "locked";
    const readyVolumes = volumes.filter((volume) => volume.access_state === "ready");
    const letter = (readyVolumes[0]?.letter || nonEmptyVolumes[0]?.letter || letters[0] || "").replace(":", "").toUpperCase();
    const slotIssues = isCardReader
      ? Math.max(
        (device.slots ?? []).filter((slot) => !["mounted", "no_media"].includes(slot.access_state)).length,
        nonEmptyVolumes.filter((volume) => volume.access_state !== "ready").length,
      )
      : 0;
    const statusText = isCardReader
      ? readyVolumes.length
        ? slotIssues
          ? t("toolkit.usb.cardReader.partiallyReady", { ready: readyVolumes.length, issues: slotIssues })
          : t("toolkit.usb.cardReader.ready", { n: readyVolumes.length })
        : nonEmptyVolumes.length || slotIssues
          ? t("toolkit.usb.cardReader.needsAttention")
          : t("toolkit.usb.cardReader.empty")
      : t(`toolkit.usb.access.${accessState}`);
    const statusWarn = device.problem_code || (!isCardReader && needsAccess) || (isCardReader && slotIssues > 0 && !hasReadyVolume);
    const volumeSummary = volumes.length
      ? volumes.map(usbVolumeDetail).join("\n")
      : t(`toolkit.usb.access.${accessState}Hint`) !== `toolkit.usb.access.${accessState}Hint`
        ? t(`toolkit.usb.access.${accessState}Hint`)
        : t("toolkit.usb.noMountedVolume");
    const formatActions = readyVolumes.map(renderUsbFormatButton).join("");
    const ejectLetters = nonEmptyVolumes.map((volume) => volume.letter.replace(":", "").toUpperCase());
    const actions = isCardReader
      ? ejectLetters.length
        ? `<button type="button" class="btn btn-accent btn-sm btn-usb-eject-all" data-letters="${escapeHtml(ejectLetters.join(","))}">${escapeHtml(t("toolkit.usb.cardReader.ejectAll"))}</button>`
        : ""
      : letter
      ? `${canPromptUnlock ? `<button type="button" class="btn btn-accent btn-sm btn-usb-unlock" data-letter="${escapeHtml(letter)}">${escapeHtml(t("toolkit.usb.unlock"))}</button>` : accessState === "mounted" ? `<button type="button" class="btn btn-default btn-sm btn-usb-scan-lock" data-letter="${escapeHtml(letter)}">${escapeHtml(t("toolkit.usb.scanLock"))}</button>` : ""}
         ${formatActions}
         <button type="button" class="btn btn-accent btn-sm btn-usb-eject" data-letter="${escapeHtml(letter)}">${escapeHtml(t("toolkit.usb.ejectDevice"))}</button>`
      : "";
    return `<li class="device-row toolkit-device-row usb-device-row${isCardReader ? " card-reader-device-row" : ""}" data-drive="${escapeHtml(letter)}">
      <div class="toolkit-device-info">
        <div class="device-name">${escapeHtml(isGenericHardwareName(device.name) ? t("terms.usb") : device.name)} <span class="result-badge result-badge-${statusWarn ? "warn" : "ok"}">${escapeHtml(statusText)}</span></div>
        ${isCardReader ? `<div class="card-reader-slots">${renderCardReaderSlots(device, drivesByLetter)}</div>` : `<div class="result-muted usb-volume-summary">${escapeHtml(volumeSummary)}</div>`}
        <div class="result-muted mono advanced-only">${escapeHtml(device.instance_id || device.physical_id || "")}${device.physical_id && device.physical_id !== device.instance_id ? ` · Container ${escapeHtml(device.physical_id)}` : ""}${device.disk_numbers?.length ? ` · Disk ${escapeHtml(device.disk_numbers.join(", "))}` : ""}</div>
      </div>
      <div class="toolkit-device-actions">${actions}</div>
      ${!isCardReader && letter ? `<ul class="lock-list usb-lock-list" id="usb-locks-${escapeHtml(letter)}"></ul>` : ""}
    </li>`;
  }).join("");
  const driveContent = groupRows
    ? `<ul class="device-list">${groupRows}</ul>`
    : `<p class="result-msg warn">${escapeHtml(t("toolkit.usb.notDetectedByWindows"))}</p>`;

  const hasHardware = Boolean(report.devices?.length);
  const hasDrives = Boolean(report.drives?.length);
  const hasDeviceError = (report.devices ?? []).some((device) => Number(device.problem_code) !== 0);
  const hasCardReaderIssue = groups.some(({ device, volumes }) => device.device_kind === "card_reader"
    && ((device.slots ?? []).some((slot) => !["mounted", "no_media"].includes(slot.access_state))
      || volumes.some((volume) => !["ready", "no_media"].includes(volume.access_state))));
  const noMediaLetters = new Set((report.devices ?? [])
    .filter((device) => device.device_kind === "card_reader")
    .flatMap((device) => (device.slots ?? [])
      .filter((slot) => slot.access_state === "no_media")
      .flatMap((slot) => slot.volume_letters ?? []))
    .map((letter) => letter.replace(":", "").toUpperCase()));
  const hasAccessBlocked = (report.drives ?? []).some((drive) => ["locked", "unavailable"].includes(drive.access_state)
    && !noMediaLetters.has(drive.letter.replace(":", "").toUpperCase()))
    || (report.devices ?? []).some((device) => ["locked", "unavailable"].includes(device.access_state));
  let state = "ok";
  let guide = renderGuidance("ok", "toolkit.usb.guide.readyTitle", "toolkit.usb.guide.readySummary");
  if (!hasHardware && !hasDrives) {
    guide = renderGuidance("ok", "toolkit.usb.guide.emptyTitle", "toolkit.usb.guide.emptySummary");
  } else if (hasDeviceError) {
    state = "warn";
    guide = renderGuidance("warn", "toolkit.usb.guide.driverTitle", "toolkit.usb.guide.driverSummary", ["toolkit.usb.guide.driverStep1", "toolkit.usb.guide.driverStep2"]);
  } else if (hasCardReaderIssue) {
    state = "warn";
    guide = renderGuidance("warn", "toolkit.usb.guide.cardIssueTitle", "toolkit.usb.guide.cardIssueSummary", ["toolkit.usb.guide.cardIssueStep1", "toolkit.usb.guide.cardIssueStep2", "toolkit.usb.guide.cardIssueStep3"]);
  } else if (hasAccessBlocked) {
    state = "warn";
    guide = renderGuidance("warn", "toolkit.usb.guide.lockedTitle", "toolkit.usb.guide.lockedSummary", ["toolkit.usb.guide.lockedStep1", "toolkit.usb.guide.lockedStep2"]);
  } else if (hasHardware && !hasDrives) {
    state = "warn";
    guide = renderGuidance("warn", "toolkit.usb.guide.noVolumeTitle", "toolkit.usb.guide.noVolumeSummary", ["toolkit.usb.guide.noVolumeStep1", "toolkit.usb.guide.noVolumeStep2", "toolkit.usb.guide.noVolumeStep3"]);
  }

  setPanel(
    el,
    state,
    `
    ${completionHint}
    ${guide}
    ${renderServicesBlock(report.services, "toolkit.usb.servicesTitle")}
    <div class="result-section">
      <div class="result-section-title">${escapeHtml(t("toolkit.usb.drivesTitle"))}</div>
      ${driveContent}
    </div>
  `,
  );
}

function openUsbFormatDialog({ letter, label, filesystem, size }) {
  return new Promise((resolve) => {
    const dialog = document.createElement("dialog");
    dialog.className = "format-dialog";
    const normalizedFs = ["NTFS", "EXFAT", "FAT32"].includes(String(filesystem).toUpperCase())
      ? String(filesystem).toUpperCase()
      : "EXFAT";
    const initialLabel = label && label !== `${letter}:` ? label : "";
    dialog.innerHTML = `
      <form class="format-dialog-form">
        <div class="format-dialog-header">
          <div class="format-warning-icon" aria-hidden="true">!</div>
          <div>
            <h2>${escapeHtml(t("toolkit.usb.formatTitle", { letter }))}</h2>
            <p>${escapeHtml(t("toolkit.usb.formatIntro"))}</p>
            <div class="format-target-summary">${escapeHtml(t("toolkit.usb.formatTarget", { letter, label: label || `${letter}:`, size, filesystem: filesystem || "—" }))}</div>
          </div>
        </div>
        <fieldset class="format-mode-group">
          <legend>${escapeHtml(t("toolkit.usb.formatMode"))}</legend>
          <label class="format-mode-card is-selected">
            <input type="radio" name="format-mode" value="quick" checked />
            <span><strong>${escapeHtml(t("toolkit.usb.quickFormat"))}</strong><small>${escapeHtml(t("toolkit.usb.quickFormatHint"))}</small></span>
          </label>
          <label class="format-mode-card format-mode-card-danger">
            <input type="radio" name="format-mode" value="full" />
            <span><strong>${escapeHtml(t("toolkit.usb.fullFormat"))}</strong><small>${escapeHtml(t("toolkit.usb.fullFormatHint"))}</small></span>
          </label>
        </fieldset>
        <div class="format-field-grid">
          <label class="format-field">
            <span>${escapeHtml(t("toolkit.usb.filesystem"))}</span>
            <select class="field-select" name="filesystem">
              <option value="NTFS"${normalizedFs === "NTFS" ? " selected" : ""}>NTFS</option>
              <option value="exFAT"${normalizedFs === "EXFAT" ? " selected" : ""}>exFAT</option>
              <option value="FAT32"${normalizedFs === "FAT32" ? " selected" : ""}>FAT32</option>
            </select>
            <small class="format-filesystem-hint"></small>
          </label>
          <label class="format-field">
            <span>${escapeHtml(t("toolkit.usb.volumeLabel"))}</span>
            <input class="format-text-input" name="label" maxlength="32" value="${escapeHtml(initialLabel)}" placeholder="${escapeHtml(t("toolkit.usb.volumeLabelPlaceholder"))}" />
          </label>
        </div>
        <div class="format-danger-notice" hidden>${escapeHtml(t("toolkit.usb.fullFormatDanger"))}</div>
        <label class="format-backup-check">
          <input type="checkbox" name="backup-confirmed" />
          <span>${escapeHtml(t("toolkit.usb.backupConfirmed"))}</span>
        </label>
        <label class="format-field format-letter-confirm" hidden>
          <span>${escapeHtml(t("toolkit.usb.typeDriveLetter", { letter }))}</span>
          <input class="format-text-input" name="letter-confirm" maxlength="2" autocomplete="off" placeholder="${escapeHtml(t("toolkit.usb.typeDriveLetterPlaceholder", { letter }))}" />
        </label>
        <div class="format-dialog-actions">
          <button type="button" class="btn btn-default format-cancel">${escapeHtml(t("toolkit.usb.cancelFormat"))}</button>
          <button type="submit" class="btn btn-danger format-confirm" disabled>${escapeHtml(t("toolkit.usb.confirmQuickFormat"))}</button>
        </div>
      </form>`;
    document.body.appendChild(dialog);
    enhanceSelectMenus(dialog);

    const form = dialog.querySelector("form");
    const confirmButton = dialog.querySelector(".format-confirm");
    const backup = form.elements.namedItem("backup-confirmed");
    const letterConfirm = form.elements.namedItem("letter-confirm");
    const filesystemSelect = form.elements.namedItem("filesystem");
    const dangerNotice = dialog.querySelector(".format-danger-notice");
    const letterConfirmField = dialog.querySelector(".format-letter-confirm");
    const filesystemHint = dialog.querySelector(".format-filesystem-hint");
    let settled = false;

    const selectedMode = () => form.elements.namedItem("format-mode").value;
    const update = () => {
      const full = selectedMode() === "full";
      dangerNotice.hidden = !full;
      letterConfirmField.hidden = !full;
      confirmButton.textContent = t(full ? "toolkit.usb.confirmFullFormat" : "toolkit.usb.confirmQuickFormat");
      confirmButton.disabled = !backup.checked || (full && letterConfirm.value.trim().toUpperCase().replace(":", "") !== letter);
      filesystemHint.textContent = t(`toolkit.usb.filesystemHint.${filesystemSelect.value}`);
      dialog.querySelectorAll(".format-mode-card").forEach((card) => {
        card.classList.toggle("is-selected", card.querySelector("input").checked);
      });
    };
    form.addEventListener("input", update);
    form.addEventListener("change", update);
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      update();
      if (confirmButton.disabled) return;
      settled = true;
      resolve({
        filesystem: filesystemSelect.value,
        label: form.elements.namedItem("label").value.trim(),
        full: selectedMode() === "full",
      });
      dialog.close();
    });
    dialog.querySelector(".format-cancel").addEventListener("click", () => dialog.close());
    dialog.addEventListener("close", () => {
      if (!settled) resolve(null);
      dialog.remove();
    });
    update();
    dialog.showModal();
  });
}

function openBluetoothRemoveDialog(deviceName) {
  return new Promise((resolve) => {
    const dialog = document.createElement("dialog");
    dialog.className = "format-dialog";
    dialog.setAttribute("aria-labelledby", "bluetooth-remove-title");
    dialog.innerHTML = `
      <form class="format-dialog-form">
        <div class="format-dialog-header">
          <span class="format-warning-icon" aria-hidden="true">!</span>
          <div>
            <h2 id="bluetooth-remove-title">${escapeHtml(t("toolkit.bluetooth.removeConfirmTitle", { name: deviceName }))}</h2>
            <p>${escapeHtml(t("toolkit.bluetooth.removeConfirmSummary"))}</p>
            <div class="format-target-summary">${escapeHtml(deviceName)}</div>
          </div>
        </div>
        <div class="format-danger-notice">${escapeHtml(t("toolkit.bluetooth.removeConfirmRisk"))}</div>
        <p class="result-muted">${escapeHtml(t("toolkit.bluetooth.removeConfirmHint"))}</p>
        <div class="format-dialog-actions">
          <button type="button" class="btn btn-default bluetooth-remove-cancel">${escapeHtml(t("toolkit.bluetooth.removeCancel"))}</button>
          <button type="submit" class="btn btn-danger">${escapeHtml(t("toolkit.bluetooth.removeConfirm"))}</button>
        </div>
      </form>`;
    document.body.appendChild(dialog);
    let settled = false;
    const finish = (confirmed) => {
      if (settled) return;
      settled = true;
      resolve(confirmed);
      dialog.close();
    };
    dialog.querySelector("form").addEventListener("submit", (event) => {
      event.preventDefault();
      finish(true);
    });
    dialog.querySelector(".bluetooth-remove-cancel").addEventListener("click", () => finish(false));
    dialog.addEventListener("close", () => {
      if (!settled) resolve(false);
      dialog.remove();
    });
    dialog.showModal();
  });
}

function usbFormatErrorMessage(error) {
  const raw = rawErrorText(error).toLowerCase();
  if (raw.includes("not_usb_storage")) return t("toolkit.usb.formatNotUsb");
  if (raw.includes("system_volume_forbidden")) return t("toolkit.usb.formatSystemBlocked");
  if (raw.includes("volume_not_found")) return t("toolkit.usb.formatVolumeMissing");
  if (raw.includes("invalid_volume_label") || raw.includes("invalid label")) return t("toolkit.usb.formatInvalidLabel");
  return userErrorMessage(error, "toolkit.usb.formatFailed");
}

function renderUsbProcessHints(processes, driveLetter = "") {
  if (!processes?.length) {
    return `<li class="result-muted">${escapeHtml(t("toolkit.usb.noRelatedProcesses"))}</li>`;
  }
  return [
    `<li class="result-msg warn">${escapeHtml(t("toolkit.usb.relatedProcessesHint"))}</li>`,
    ...processes.map(
      (process) =>
        `<li class="lock-row"><span><strong>${escapeHtml(process.name)}</strong><span class="advanced-only advanced-inline"> · PID ${process.pid}</span>${process.path ? `<small class="advanced-only mono">${escapeHtml(process.path)}</small>` : ""}</span>${process.can_close ? `<button type="button" class="btn btn-default btn-sm btn-usb-close-process" data-pid="${process.pid}" data-process="${escapeHtml(process.name)}" data-letter="${escapeHtml(driveLetter)}">${escapeHtml(t("toolkit.usb.requestClose"))}</button>` : `<span class="result-muted">${escapeHtml(t("toolkit.usb.manualOnly"))}</span>`}</li>`,
    ),
  ].join("");
}

function usbEjectFailureMessage(result) {
  if (result.status === "busy") return t("toolkit.usb.ejectBusy");
  if (result.status === "permission_required") return t("toolkit.usb.ejectPermission");
  if (result.status === "vetoed") {
    const key = `toolkit.usb.veto.${result.veto_type || "unknown"}`;
    const reason = t(key);
    return t("toolkit.usb.ejectVetoed", {
      reason: reason === key ? t("toolkit.usb.veto.unknown") : reason,
      name: result.veto_name || t("toolkit.usb.veto.unknownSource"),
    });
  }
  const stageKey = `toolkit.usb.stage.${result.stage || "pnp"}`;
  const stage = t(stageKey);
  return t("toolkit.usb.ejectFailed", {
    stage: stage === stageKey ? t("toolkit.usb.stage.pnp") : stage,
  });
}

export function formatBluetoothIssue(issue) {
  if (!issue?.id) return "";
  const key = `toolkit.bluetooth.issue.${issue.id}`;
  const text = t(key, {
    name: issue.name ?? "—",
    state: issue.state ?? "—",
    code: issue.code ?? "—",
    adapter: t("terms.bluetoothAdapter"),
    service: t("terms.bluetoothSupportService"),
  });
  return text !== key ? text : t("toolkit.bluetooth.issue.unknown");
}

let latestBluetoothReport = null;

export function renderBluetoothReport(ev) {
  latestBluetoothReport = ev;
  const el = $("bluetooth-result");
  const issueIds = new Set((ev.issues ?? []).map((issue) => issue.id));
  const state = issueIds.has("no_radio") ? "crit" : ev.healthy ? "ok" : "warn";
  const issues = renderList(
    (ev.issues ?? []).map(formatBluetoothIssue),
    t("toolkit.bluetooth.noIssues"),
  );
  const technicalIssues = renderList(
    (ev.issues ?? []).map((issue) =>
      [issue.id, issue.name, issue.state, issue.code != null ? `Code ${issue.code}` : ""]
        .filter(Boolean)
        .join(" · "),
    ),
    t("toolkit.bluetooth.noIssues"),
  );
  const devices = (ev.devices ?? [])
    .map((d) => {
      const connectionKnown = typeof d.connected === "boolean";
      const statusBadge = d.connected === true ? "ok" : d.connected === false ? "warn" : "neutral";
      const connectionText = connectionKnown
        ? t(d.connected ? "toolkit.bluetooth.connected" : "toolkit.bluetooth.disconnected")
        : t("toolkit.unknown");
      const batteryPercent = Number.isFinite(Number(d.battery_percent))
        ? Math.min(100, Math.max(0, Number(d.battery_percent)))
        : null;
      const battery = batteryPercent !== null && (!d.battery_state || d.battery_state === "live")
        ? `<span class="result-badge result-badge-${batteryPercent <= 20 ? "warn" : "ok"}">${escapeHtml(t("toolkit.bluetooth.battery", { percent: batteryPercent }))}</span>`
        : "";
      const batteryState = d.connected !== false && d.battery_state === "refreshing"
        ? t("toolkit.bluetooth.batteryRefreshing")
        : d.connected !== false && d.battery_state === "unavailable"
          ? t("toolkit.bluetooth.batteryUnavailable")
          : "";
      const batteryStateBadge = batteryState
        ? `<span class="result-badge result-badge-${d.battery_state === "live" ? "ok" : "neutral"}">${escapeHtml(batteryState)}</span>`
        : "";
      return `
      <li class="device-row toolkit-device-row bt-device-row" data-instance="${escapeHtml(d.instance_id)}">
        <div class="toolkit-device-info">
          <div class="device-name">${escapeHtml(d.name)}</div>
          <div class="result-muted mono advanced-only">${escapeHtml(d.instance_id)} · ${escapeHtml(d.status)}</div>
        </div>
        <div class="toolkit-device-actions">
          ${battery}
          ${batteryStateBadge}
          <span class="result-badge result-badge-${statusBadge}">${escapeHtml(connectionText)}</span>
          <button type="button" class="btn btn-default btn-sm btn-bt-reconnect" data-id="${escapeHtml(d.instance_id)}">${escapeHtml(t("toolkit.bluetooth.reconnect"))}</button>
          <button type="button" class="btn btn-subtle btn-sm btn-bt-remove" data-id="${escapeHtml(d.instance_id)}" data-name="${escapeHtml(d.name)}">${escapeHtml(t("toolkit.bluetooth.remove"))}</button>
        </div>
      </li>`;
    })
    .join("");
  const adapters = (ev.adapters ?? []).length
    ? ev.adapters.join(" · ")
    : t("toolkit.bluetooth.radioCount", { n: ev.adapter_count ?? 0 });

  let guide = renderGuidance("ok", "toolkit.bluetooth.guide.okTitle", "toolkit.bluetooth.guide.okSummary");
  if (issueIds.has("no_radio")) {
    guide = renderGuidance("crit", "toolkit.bluetooth.guide.noRadioTitle", "toolkit.bluetooth.guide.noRadioSummary", ["toolkit.bluetooth.guide.noRadioStep1", "toolkit.bluetooth.guide.noRadioStep2", "toolkit.bluetooth.guide.noRadioStep3"]);
  } else if (!ev.healthy) {
    guide = renderGuidance("warn", "toolkit.bluetooth.guide.problemTitle", "toolkit.bluetooth.guide.problemSummary", ["toolkit.bluetooth.guide.problemStep1", "toolkit.bluetooth.guide.problemStep2"]);
  }

  setPanel(
    el,
    state,
    `
    ${guide}
    <div class="result-head">
      <span class="result-badge result-badge-${state}">${escapeHtml(ev.healthy ? t("toolkit.ok") : t("toolkit.warn"))}</span>
      <span class="result-time">${formatTime(ev.timestamp)}</span>
    </div>
    <div class="result-row advanced-only"><span class="result-label">${escapeHtml(t("terms.bluetoothSupportService"))}</span><span class="result-value">${escapeHtml(formatServiceState(ev.bthserv_state))}</span></div>
    <div class="result-row advanced-only"><span class="result-label">${escapeHtml(t("terms.bluetoothAdapter"))}</span><span class="result-value">${escapeHtml(adapters)}</span></div>
    <div class="result-section">
      <div class="result-section-title">${escapeHtml(t("toolkit.bluetooth.devicesTitle"))}</div>
      <ul class="device-list">${devices || `<li class="result-muted">${escapeHtml(t("toolkit.bluetooth.noDevices"))}</li>`}</ul>
    </div>
    <div class="result-section">
      <div class="result-section-title">${escapeHtml(t("toolkit.bluetooth.issues"))}</div>
      ${issues}
    </div>
    <details class="technical-details advanced-only">
      <summary>${escapeHtml(t("toolkit.technicalDetails"))}</summary>
      ${technicalIssues}
    </details>
  `,
  );
}

export function applyBluetoothBatteryRefresh(devices) {
  if (!latestBluetoothReport || !Array.isArray(devices)) return;
  const batteryById = new Map(devices
    .filter((device) => device?.instance_id
      && (device.battery_state || typeof device.connected === "boolean"))
    .map((device) => [device.instance_id, device]));
  if (!batteryById.size) return;
  const stateRank = { refreshing: 0, unavailable: 1, live: 2 };
  renderBluetoothReport({
    ...latestBluetoothReport,
    devices: (latestBluetoothReport.devices ?? []).map((device) => {
      const update = batteryById.get(device.instance_id);
      if (!update) return device;
      const next = { ...device };
      if (typeof update.connected === "boolean") {
        next.connected = update.connected;
        if (!update.battery_state) {
          next.battery_percent = null;
          next.battery_state = null;
        }
      }
      if (next.connected !== false && update.battery_state
        && (stateRank[update.battery_state] ?? -1) >= (stateRank[device.battery_state] ?? -1)) {
        next.battery_percent = update.battery_percent ?? device.battery_percent;
        next.battery_state = update.battery_state;
      }
      return next;
    }),
  });
}

function formatDeviceProblem(problem) {
  const reasonKey = `toolkit.devices.reason.${problem.reason_id}`;
  const reason = t(reasonKey);
  return reason === reasonKey ? t("toolkit.devices.reason.other") : reason;
}

function driverActionLabel(actionId) {
  const key = `toolkit.devices.action.${actionId}`;
  const label = t(key);
  return label === key ? actionId : label;
}

function renderDriverEvidence(driver) {
  if (!driver) return `<div class="result-muted advanced-only">${escapeHtml(t("toolkit.devices.driverEvidenceUnavailable"))}</div>`;
  const values = [driver.provider, driver.version, driver.inf_name].filter(Boolean);
  return values.length
    ? `<div class="result-muted mono advanced-only">${escapeHtml(values.join(" | "))}</div>`
    : `<div class="result-muted advanced-only">${escapeHtml(t("toolkit.devices.driverEvidenceUnavailable"))}</div>`;
}

function confirmDriverAction(problem, actionId) {
  return new Promise((resolve) => {
    const dialog = document.createElement("dialog");
    dialog.className = "format-dialog";
    dialog.innerHTML = `
      <form class="format-dialog-form">
        <div class="format-dialog-header">
          <h2>${escapeHtml(t("toolkit.devices.confirmTitle"))}</h2>
          <p>${escapeHtml(t("toolkit.devices.confirmMessage", { action: driverActionLabel(actionId), name: problem.name }))}</p>
        </div>
        <div class="format-warning">${escapeHtml(t(`toolkit.devices.risk.${actionId}`))}</div>
        <div class="format-dialog-actions">
          <button type="button" class="btn driver-action-cancel">${escapeHtml(t("toolkit.devices.cancel"))}</button>
          <button type="submit" class="btn btn-accent">${escapeHtml(t("toolkit.devices.confirmRun"))}</button>
        </div>
      </form>`;
    document.body.appendChild(dialog);
    let settled = false;
    const finish = (confirmed) => {
      if (settled) return;
      settled = true;
      resolve(confirmed);
      dialog.close();
    };
    dialog.querySelector("form").addEventListener("submit", (event) => {
      event.preventDefault();
      finish(true);
    });
    dialog.querySelector(".driver-action-cancel").addEventListener("click", () => finish(false));
    dialog.addEventListener("close", () => {
      if (!settled) resolve(false);
      dialog.remove();
    });
    dialog.showModal();
  });
}

function renderDriverRepairResult(result) {
  const key = result.needs_admin
    ? "needsAdmin"
    : result.verified
      ? "verified"
      : !result.command_succeeded
        ? "commandFailed"
        : !result.device_present
          ? "deviceMissing"
          : "stillFaulty";
  const state = result.verified ? "ok" : "warn";
  const code = result.after_code ?? "-";
  return `<p class="repair-verdict ${state}">${escapeHtml(t(`toolkit.devices.result.${key}`, { code }))}</p>
    ${result.reboot_required ? `<p class="result-muted">${escapeHtml(t("toolkit.devices.result.reboot"))}</p>` : ""}
    ${result.details ? `<details class="technical-details advanced-only"><summary>${escapeHtml(t("toolkit.technicalDetails"))}</summary><pre class="technical-error mono">${escapeHtml(result.details)}</pre></details>` : ""}`;
}

export function renderDevicesReport(report) {
  const el = $("devices-result");
  const missingEssential = report.network_missing || report.display_missing;
  const hasProblems = Boolean(report.problems?.length);
  const state = missingEssential ? "crit" : hasProblems ? "warn" : "ok";
  const guide = missingEssential
    ? renderGuidance("crit", "toolkit.devices.guide.missingTitle", "toolkit.devices.guide.missingSummary", ["toolkit.devices.guide.missingStep1", "toolkit.devices.guide.missingStep2", "toolkit.devices.guide.missingStep3"])
    : hasProblems
      ? renderGuidance("warn", "toolkit.devices.guide.problemTitle", "toolkit.devices.guide.problemSummary", ["toolkit.devices.guide.problemStep1", "toolkit.devices.guide.problemStep2"])
      : renderGuidance("ok", "toolkit.devices.guide.okTitle", "toolkit.devices.guide.okSummary");
  const classes = (report.classes ?? [])
    .map((item) => `<div class="device-class-tile">
      <span>${escapeHtml(t(`toolkit.devices.class.${item.id}`))}</span>
      <strong>${escapeHtml(String(item.count ?? 0))}</strong>
    </div>`)
    .join("");
  const problems = (report.problems ?? [])
    .map((problem) => `<li class="device-row toolkit-device-row" data-device-id="${escapeHtml(problem.device_id)}">
      <div class="toolkit-device-info">
        <div class="device-name">${escapeHtml(isGenericHardwareName(problem.name) ? t("events.unknownDevice") : problem.name)}</div>
        <div class="result-muted">${escapeHtml(formatDeviceProblem(problem))}</div>
        ${renderDriverEvidence(problem.driver)}
        <div class="result-muted mono advanced-only">${escapeHtml(problem.device_id)}</div>
        <div class="driver-repair-result" aria-live="polite"></div>
      </div>
      <div class="toolkit-device-actions">
        <span class="result-badge result-badge-warn">${escapeHtml(t(`toolkit.devices.class.${problem.class_id}`))}<span class="advanced-only advanced-inline"> · Code ${problem.error_code}</span></span>
        ${(problem.available_actions ?? []).map((actionId) => `<button type="button" class="btn btn-sm btn-device-driver-action" data-action="${escapeHtml(actionId)}">${escapeHtml(driverActionLabel(actionId))}</button>`).join("")}
      </div>
    </li>`)
    .join("");

  setPanel(el, state, `
    ${guide}
    <div class="device-class-grid">${classes}</div>
    <div class="result-section">
      <div class="result-section-title">${escapeHtml(t("toolkit.devices.problemsTitle"))}</div>
      <ul class="device-list">${problems || `<li class="result-muted">${escapeHtml(t("toolkit.devices.noProblems"))}</li>`}</ul>
    </div>
  `);
}

let portReportCache = null;
let portSortMode = "port";

const PORT_SORT = {
  port: (a, b) => a.port - b.port || a.sort_priority - b.sort_priority,
  category: (a, b) => a.sort_priority - b.sort_priority || a.port - b.port,
  releasable: (a, b) => Number(b.can_release) - Number(a.can_release) || a.sort_priority - b.sort_priority || a.port - b.port,
};

export function sortPortEntries(entries, mode = portSortMode) {
  const fn = PORT_SORT[mode] ?? PORT_SORT.port;
  return [...entries].sort(fn);
}

export function renderPortScan(report, { onToast } = {}) {
  portReportCache = report;
  const rangesEl = $("excluded-ranges");
  if (report.excluded_ranges?.length) {
    const chips = report.excluded_ranges
      .map((r) => `<span class="reserved-chip">${r.start}–${r.end}</span>`)
      .join("");
    rangesEl.innerHTML = `<div class="reserved-title">${escapeHtml(t("ports.reservedTitle"))}</div><div class="reserved-chips">${chips}</div>`;
  } else {
    rangesEl.innerHTML = "";
  }

  const sorted = sortPortEntries(report.entries ?? []);
  const list = $("port-list");
  if (!sorted.length) {
    list.innerHTML = `<li class="empty-state">${escapeHtml(t("ports.noListeners"))}</li>`;
  } else {
    list.innerHTML = sorted.map((e) => renderPortRow(e)).join("");
  }

  const btn = $("btn-release-all-ports");
  const n = report.releasable_count ?? 0;
  btn.disabled = n === 0;
  btn.textContent = n > 0 ? t("ports.releaseAllN", { n }) : t("ports.releaseAll");
  if (onToast) onToast._ = null;
}

function renderPortRow(entry) {
  const pid = entry.pid ?? "—";
  const proc = entry.process_name ?? "—";
  const categoryLabel = t(`ports.category.${entry.category}`);
  const messageText = t(`ports.message.${entry.message_id}`, { state: entry.message });
  let releaseBtn = "";
  if (entry.can_terminate && entry.pid && entry.process_name) {
    releaseBtn = `<button class="btn btn-default btn-sm btn-port-release" data-pid="${entry.pid}" data-process="${escapeHtml(entry.process_name)}" data-port="${entry.port}" type="button">${escapeHtml(t("ports.releaseOne"))}</button>`;
  }
  return `
    <li class="port-row" data-sort="${entry.sort_priority}">
      <span class="port-num">${entry.port}</span>
      <div class="port-info">
        <span class="port-badge port-badge-${entry.category}">${escapeHtml(categoryLabel)}</span>
        <div class="port-meta">${escapeHtml(proc)}<span class="advanced-only advanced-inline"> · ${escapeHtml(entry.state)} · PID ${pid}</span></div>
        <div class="port-msg">${escapeHtml(messageText)}</div>
      </div>
      <div class="port-actions">${releaseBtn}</div>
    </li>`;
}

const FULL_SCAN_CHECKS = [
  ["network", "diagnose_network"],
  ["audio", "diagnose_audio"],
  ["usb", "diagnose_usb"],
  ["bluetooth", "check_bluetooth"],
  ["devices", "diagnose_devices"],
];

let activeFullScan = null;
let fullScanRequestId = 0;

function summarizeFullScanItem(id, command, scanItem) {
  if (!scanItem) {
    return {
      id,
      state: "pending",
      detail: t("overview.fullScanning"),
      technical: "",
    };
  }
  if (scanItem.error || !scanItem.result) {
    return {
      id,
      state: "warn",
      detail: t("overview.checkFailed"),
      technical: `${command} · ${rawErrorText(scanItem.error ?? "missing scan result")}`,
    };
  }

  const report = scanItem.result;
  if (id === "network") {
    if ((report.adapter_count ?? 0) === 0) return { id, state: "crit", detail: t("overview.health.networkMissing") };
    if (report.gateway_reachable === false || report.services?.issues?.length) return { id, state: "warn", detail: t("overview.health.networkIssue") };
  } else if (id === "audio") {
    const count = (report.playback?.length ?? 0) + (report.capture?.length ?? 0);
    if (count === 0) return { id, state: "crit", detail: t("overview.health.audioMissing") };
    if (report.services?.issues?.length) return { id, state: "warn", detail: t("overview.health.audioIssue") };
  } else if (id === "usb") {
    if ((report.devices ?? []).some((device) => Number(device.problem_code) !== 0)) return { id, state: "warn", detail: t("overview.health.usbIssue") };
  } else if (id === "bluetooth") {
    if ((report.issues ?? []).some((issue) => issue.id === "no_radio")) return { id, state: "crit", detail: t("overview.health.bluetoothMissing") };
    if (!report.healthy) return { id, state: "warn", detail: t("overview.health.bluetoothIssue") };
  } else if (id === "devices") {
    if (report.network_missing || report.display_missing) return { id, state: "crit", detail: t("overview.health.devicesMissing") };
    if (report.problems?.length) return { id, state: "warn", detail: t("overview.health.devicesIssue", { n: report.problems.length }) };
  }
  return {
    id,
    state: "ok",
    detail: t("overview.health.ok"),
    technical: formatDuration(scanItem.duration_ms),
  };
}

function renderFullScanOverview(el, scanItems) {
  const items = FULL_SCAN_CHECKS.map(([id, command]) => summarizeFullScanItem(id, command, scanItems[id]));
  const completed = items.filter((item) => item.state !== "pending").length;
  const pending = completed < items.length;
  const hasCritical = items.some((item) => item.state === "crit");
  const hasWarning = items.some((item) => item.state === "warn");
  const overall = hasCritical ? "crit" : hasWarning ? "warn" : "ok";
  const headline = pending
    ? t("overview.fullScanning")
    : t(`overview.healthOverall.${overall}`);
  const badge = pending
    ? `<span class="result-badge">${completed}/${items.length}</span>`
    : `<span class="result-badge result-badge-${overall}">${escapeHtml(t(`toolkit.verdict.${overall}`))}</span>`;
  el.innerHTML = `<div class="health-overview-head">
    <div><span>${escapeHtml(t("overview.healthTitle"))}</span><strong>${escapeHtml(headline)}</strong></div>
    ${badge}
  </div><div class="health-grid">${items.map((item) => `<div class="health-item">
    <span class="health-dot health-dot-${item.state}"></span>
    <div><strong>${escapeHtml(t(`nav.${item.id}`))}</strong><small>${escapeHtml(item.detail)}</small>${item.technical ? `<small class="advanced-only mono">${escapeHtml(item.technical)}</small>` : ""}</div>
  </div>`).join("")}</div>`;
}

export function applyFullScanProgress(progress) {
  if (!activeFullScan || !progress?.id || !progress.item) return;
  if (activeFullScan.requestId !== progress.request_id) return;
  activeFullScan.items[progress.id] = progress.item;
  renderFullScanOverview(activeFullScan.element, activeFullScan.items);
}

export function bindToolkitHandlers({ showToast }) {
  $("btn-full-scan")?.addEventListener("click", (event) => {
    runButtonTask(event.currentTarget, async () => {
      const el = $("health-overview");
      if (!el) {
        showToast(t("overview.checkFailed"), true);
        return;
      }
      el.classList.remove("hidden");
      const requestId = ++fullScanRequestId;
      activeFullScan = { element: el, requestId, items: {} };
      renderFullScanOverview(el, activeFullScan.items);
      let fullScan;
      try {
        fullScan = await invoke("full_scan", { requestId });
      } catch (error) {
        activeFullScan = null;
        el.innerHTML = `<div class="health-scan-progress">${renderOperationError(error, "full_scan")}</div>`;
        showToast(t("overview.checkFailed"), true);
        return;
      }
      const scanItems = fullScan?.items ?? {};
      renderFullScanOverview(el, scanItems);
      activeFullScan = null;
    });
  });

  document.body.addEventListener("input", (e) => {
    const slider = e.target.closest?.(".audio-volume-slider");
    if (!slider) return;
    const output = slider.parentElement?.querySelector(".audio-volume-value");
    if (output) output.textContent = `${slider.value}%`;
  });

  document.body.addEventListener("change", async (e) => {
    const slider = e.target.closest?.(".audio-volume-slider");
    if (slider) {
      try {
        await invoke("set_audio_volume", { deviceId: slider.dataset.id, percent: Number(slider.value) });
      } catch (err) {
        notifyOperationError(showToast, err, "set_audio_volume");
        try {
          renderAudioReport(await invoke("diagnose_audio"));
        } catch (scanError) {
          console.error("[ZeroTick:diagnose_audio_after_volume_error]", scanError);
        }
      }
      return;
    }

    const audioMode = e.target.closest?.(".audio-exclusive-toggle");
    if (audioMode) {
      const row = audioMode.closest(".audio-device-row");
      const allow = row?.querySelector('[data-setting="allow"]');
      const priority = row?.querySelector('[data-setting="priority"]');
      if (!allow || !priority) return;
      if (!allow.checked) priority.checked = false;
      priority.disabled = !allow.checked;
      priority.closest(".audio-option")?.classList.toggle("is-disabled", !allow.checked);
      const mode = allow.checked ? (priority.checked ? "exclusive_priority" : "exclusive") : "shared";
      try {
        await invoke("set_audio_mode", {
          deviceId: audioMode.dataset.id,
          kind: audioMode.dataset.kind || "playback",
          mode,
        });
        renderAudioReport(await invoke("diagnose_audio"));
        showToast(t("toast.audioMode"), false);
      } catch (err) {
        const friendly = formatAudioModeError(err);
        const message = document.documentElement.classList.contains("show-advanced")
          ? `${friendly} ${t("errors.rawDetail")}: ${rawErrorText(err).slice(0, 280)}`
          : friendly;
        console.error("[ZeroTick:set_audio_mode]", err);
        showToast(message, true);
        renderAudioReport(await invoke("diagnose_audio"));
      }
    }
  });

  document.body.addEventListener("click", async (e) => {
    const audioMute = e.target.closest(".btn-audio-mute");
    if (audioMute) {
      try {
        await invoke("set_audio_mute", {
          deviceId: audioMute.dataset.id,
          muted: audioMute.dataset.muted !== "true",
        });
        renderAudioReport(await invoke("diagnose_audio"));
      } catch (err) {
        notifyOperationError(showToast, err, "set_audio_mute");
      }
      return;
    }

    const audioDefault = e.target.closest(".btn-audio-default");
    if (audioDefault) {
      try {
        await invoke("set_default_audio_device", {
          deviceId: audioDefault.dataset.id,
          kind: audioDefault.dataset.kind || "playback",
        });
        showToast(
          (audioDefault.dataset.kind || "playback") === "capture"
            ? t("toast.audioInputDefault")
            : t("toast.audioDefault"),
          false,
        );
        const r = await invoke("diagnose_audio");
        renderAudioReport(r);
      } catch (err) {
        notifyOperationError(showToast, err, "set_default_audio_device");
      }
      return;
    }

    const usbLock = e.target.closest(".btn-usb-scan-lock");
    if (usbLock) {
      const letter = usbLock.dataset.letter;
      const listEl = $(`usb-locks-${letter}`);
      listEl.innerHTML = `<li class="result-muted">${escapeHtml(t("toolkit.loading"))}</li>`;
      try {
        const procs = await invoke("usb_locking_processes", { driveLetter: `${letter}:` });
        listEl.innerHTML = renderUsbProcessHints(procs, letter);
      } catch (err) {
        listEl.innerHTML = `<li>${renderOperationError(err, "usb_locking_processes", "toolkit.usb.usageCheckFailed")}</li>`;
      }
      return;
    }

    const usbUnlock = e.target.closest(".btn-usb-unlock");
    if (usbUnlock) {
      const letter = usbUnlock.dataset.letter;
      try {
        await invoke("usb_open_volume", { driveLetter: `${letter}:` });
        showToast(t("toolkit.usb.unlockOpened", { letter }), false);
      } catch (err) {
        notifyOperationError(showToast, err, "usb_open_volume");
      }
      return;
    }

    const usbCloseProcess = e.target.closest(".btn-usb-close-process");
    if (usbCloseProcess) {
      const letter = usbCloseProcess.dataset.letter;
      usbCloseProcess.disabled = true;
      try {
        const result = await invoke("usb_close_process", {
          pid: Number(usbCloseProcess.dataset.pid),
          driveLetter: `${letter}:`,
          expectedProcessName: usbCloseProcess.dataset.process,
        });
        showToast(t(`toolkit.usb.closeResult.${result.status}`), result.status !== "requested");
        await new Promise((resolve) => window.setTimeout(resolve, 500));
        const listEl = $(`usb-locks-${letter}`);
        if (listEl) {
          const processes = await invoke("usb_locking_processes", { driveLetter: `${letter}:` });
          listEl.innerHTML = renderUsbProcessHints(processes, letter);
        }
      } catch (err) {
        notifyOperationError(showToast, err, "usb_close_process");
      } finally {
        if (usbCloseProcess.isConnected) usbCloseProcess.disabled = false;
      }
      return;
    }

    const usbFormat = e.target.closest(".btn-usb-format");
    if (usbFormat) {
      runButtonTask(usbFormat, async () => {
        const letter = usbFormat.dataset.letter;
        const config = await openUsbFormatDialog({
          letter,
          label: usbFormat.dataset.label || "",
          filesystem: usbFormat.dataset.filesystem || "",
          size: usbFormat.dataset.size || "0.0",
        });
        if (!config) return;
        const listEl = $(`usb-locks-${letter}`);
        const progressText = t(config.full ? "toolkit.usb.formattingFull" : "toolkit.usb.formattingQuick", { letter });
        usbFormat.textContent = progressText;
        if (listEl) listEl.innerHTML = `<li class="result-msg warn">${escapeHtml(progressText)}</li>`;
        try {
          await invoke("usb_format_volume", {
            driveLetter: `${letter}:`,
            filesystem: config.filesystem,
            label: config.label,
            full: config.full,
          });
          showToast(t("toolkit.usb.formatComplete", { letter }), false);
          renderUsbReport(await invoke("diagnose_usb"));
        } catch (err) {
          const friendly = usbFormatErrorMessage(err);
          console.error("[ZeroTick:usb_format_volume]", err);
          if (listEl) listEl.innerHTML = `<li>${renderOperationError(err, "usb_format_volume", "toolkit.usb.formatFailed", friendly)}</li>`;
          const message = document.documentElement.classList.contains("show-advanced")
            ? `${friendly} ${t("errors.rawDetail")}: ${rawErrorText(err).slice(0, 280)}`
            : friendly;
          showToast(message, true);
        }
      });
      return;
    }

    const usbEjectAll = e.target.closest(".btn-usb-eject-all");
    if (usbEjectAll) {
      runButtonTask(usbEjectAll, async () => {
        const letters = (usbEjectAll.dataset.letters || "").split(",").map((letter) => letter.trim()).filter(Boolean);
        if (!letters.length) return;
        usbEjectAll.textContent = t("toolkit.usb.cardReader.ejectingAll");
        const failures = [];
        for (const letter of letters) {
          try {
            const result = await invoke("usb_eject", { driveLetter: `${letter}:` });
            if (result.status !== "ejected") failures.push({
              letter,
              message: usbEjectFailureMessage(result),
              blockers: result.blockers ?? [],
            });
          } catch (error) {
            failures.push({ letter, message: userErrorMessage(error) });
            console.error(`[ZeroTick:usb_eject_all:${letter}]`, error);
          }
        }
        try {
          renderUsbReport(await invoke("diagnose_usb"));
        } catch (error) {
          notifyOperationError(showToast, error, "diagnose_usb_after_eject_all", "errors.detectionFailed");
          return;
        }
        for (const failure of failures) {
          const listEl = $(`usb-locks-${failure.letter}`);
          if (listEl) listEl.innerHTML = [
            `<li class="result-msg warn">${escapeHtml(failure.message)}</li>`,
            failure.blockers?.length ? renderUsbProcessHints(failure.blockers, failure.letter) : "",
          ].join("");
        }
        showToast(
          t(failures.length ? "toolkit.usb.cardReader.ejectAllPartial" : "toolkit.usb.cardReader.ejectAllComplete"),
          failures.length > 0,
        );
      });
      return;
    }

    const usbEject = e.target.closest(".btn-usb-eject");
    if (usbEject) {
      const letter = usbEject.dataset.letter;
      const listEl = $(`usb-locks-${letter}`);
      const originalLabel = usbEject.textContent;
      usbEject.disabled = true;
      usbEject.textContent = t("toolkit.usb.ejecting");
      try {
        const result = await invoke("usb_eject", { driveLetter: `${letter}:` });
        if (result.status === "ejected") {
          showToast(t("toast.usbEjected", { letter }), false);
          renderUsbReport(await invoke("diagnose_usb"));
        } else {
          const message = usbEjectFailureMessage(result);
          listEl.innerHTML = [
            `<li class="result-msg warn">${escapeHtml(message)}</li>`,
            result.status === "busy" || result.veto_type === "outstanding_open"
              ? renderUsbProcessHints(result.blockers, letter)
              : "",
          ].join("");
          showToast(t("toolkit.usb.ejectBlockedShort"), true);
        }
      } catch (err) {
        notifyOperationError(showToast, err, "usb_eject");
      } finally {
        if (usbEject.isConnected) {
          usbEject.disabled = false;
          usbEject.textContent = originalLabel;
        }
      }
      return;
    }

    const btReconnect = e.target.closest(".btn-bt-reconnect");
    if (btReconnect) {
      try {
        await invoke("bluetooth_reconnect_device", { instanceId: btReconnect.dataset.id });
        showToast(t("toast.btReconnect"), false);
        renderBluetoothReport(await invoke("check_bluetooth"));
      } catch (err) {
        notifyOperationError(showToast, err, "bluetooth_reconnect_device");
      }
      return;
    }

    const btRemove = e.target.closest(".btn-bt-remove");
    if (btRemove) {
      const deviceName = btRemove.dataset.name || t("toolkit.bluetooth.unknownDevice");
      if (!await openBluetoothRemoveDialog(deviceName)) return;
      try {
        await invoke("bluetooth_remove_device", { instanceId: btRemove.dataset.id });
        showToast(t("toast.btRemoved"), false);
        renderBluetoothReport(await invoke("check_bluetooth"));
      } catch (err) {
        notifyOperationError(showToast, err, "bluetooth_remove_device");
      }
      return;
    }

  });

  $("btn-network-scan")?.addEventListener("click", async () => {
    setPanel($("network-result"), "loading", `<p class="result-empty">${escapeHtml(t("toolkit.loading"))}</p>`);
    try {
      renderNetworkReport(await invoke("diagnose_network"));
    } catch (err) {
      setPanel($("network-result"), "crit", renderOperationError(err, "diagnose_network"));
    }
  });

  $("btn-network-speed")?.addEventListener("click", async () => {
    const el = $("network-result");
    setPanel(el, "loading", `<p class="result-empty">${escapeHtml(t("toolkit.network.speedTesting"))}</p>`);
    try {
      const r = await invoke("network_speed_test");
      renderNetworkSpeedResult(r);
      showToast(
        document.documentElement.classList.contains("show-advanced")
          ? `${t("toast.speedResultSimple", { speed: r.speed_mbps.toFixed(2) })} · ${t("toolkit.network.speedDuration")}: ${formatDuration(r.duration_ms)}`
          : t("toast.speedResultSimple", { speed: r.speed_mbps.toFixed(2) }),
        false,
      );
    } catch (err) {
      const message = formatSpeedTestError(err);
      setPanel(el, "crit", renderOperationError(err, "network_speed_test", "toolkit.network.speedError.unknown", message));
      const toastMessage = document.documentElement.classList.contains("show-advanced")
        ? `${message} ${t("errors.rawDetail")}: ${rawErrorText(err).slice(0, 280)}`
        : message;
      console.error("[ZeroTick:network_speed_test]", err);
      showToast(toastMessage, true);
    }
  });

  $("btn-network-flush")?.addEventListener("click", async () => {
    try {
      await invoke("network_flush_dns");
      showToast(t("toast.dnsFlushed"), false);
    } catch (err) {
      notifyOperationError(showToast, err, "network_flush_dns");
    }
  });

  $("btn-network-repair")?.addEventListener("click", (event) => {
    runButtonTask(event.currentTarget, async () => {
      try {
        const r = await invoke("repair_network");
        const report = await invoke("diagnose_network");
        renderNetworkReport(report);
        const remainingIssue = (report.adapter_count ?? 0) === 0
          || report.gateway_reachable === false
          || (report.services?.issues?.length ?? 0) > 0;
        const el = $("network-result");
        el.insertAdjacentHTML("beforeend", renderRepairBlock(r, remainingIssue));
        showRepairOutcome(showToast, r, remainingIssue);
      } catch (err) {
        notifyOperationError(showToast, err, "repair_network");
      }
    });
  });

  $("btn-audio-scan")?.addEventListener("click", async () => {
    setPanel($("audio-result"), "loading", `<p class="result-empty">${escapeHtml(t("toolkit.loading"))}</p>`);
    try {
      renderAudioReport(await invoke("diagnose_audio"));
    } catch (err) {
      setPanel($("audio-result"), "crit", renderOperationError(err, "diagnose_audio"));
    }
  });

  $("btn-audio-repair")?.addEventListener("click", (event) => {
    runButtonTask(event.currentTarget, async () => {
      try {
        const r = await invoke("repair_audio");
        const report = await invoke("diagnose_audio");
        renderAudioReport(report);
        const remainingIssue = (report.services?.issues?.length ?? 0) > 0
          || (report.playback?.length ?? 0) + (report.capture?.length ?? 0) === 0;
        $("audio-result").insertAdjacentHTML("beforeend", renderRepairBlock(r, remainingIssue));
        showRepairOutcome(showToast, r, remainingIssue);
      } catch (err) {
        notifyOperationError(showToast, err, "repair_audio");
      }
    });
  });

  $("btn-usb-scan")?.addEventListener("click", async () => {
    setPanel($("usb-result"), "loading", `<p class="result-empty">${escapeHtml(t("toolkit.loading"))}</p>`);
    try {
      renderUsbReport(await invoke("diagnose_usb_progressive"));
    } catch (err) {
      setPanel($("usb-result"), "crit", renderOperationError(err, "diagnose_usb"));
    }
  });

  $("btn-usb-repair")?.addEventListener("click", (event) => {
    runButtonTask(event.currentTarget, async () => {
      try {
        const r = await invoke("repair_usb");
        const report = await invoke("diagnose_usb");
        renderUsbReport(report);
        const remainingIssue = (report.services?.issues?.length ?? 0) > 0
          || (report.devices ?? []).some((device) => Number(device.problem_code) !== 0);
        $("usb-result").insertAdjacentHTML("beforeend", renderRepairBlock(r, remainingIssue));
        showRepairOutcome(showToast, r, remainingIssue);
      } catch (err) {
        notifyOperationError(showToast, err, "repair_usb");
      }
    });
  });

  $("btn-bluetooth-scan")?.addEventListener("click", async () => {
    setPanel($("bluetooth-result"), "loading", `<p class="result-empty">${escapeHtml(t("toolkit.loading"))}</p>`);
    try {
      renderBluetoothReport(await invoke("check_bluetooth"));
    } catch (err) {
      setPanel($("bluetooth-result"), "crit", renderOperationError(err, "check_bluetooth"));
    }
  });

  $("btn-bluetooth-repair")?.addEventListener("click", (event) => {
    runButtonTask(event.currentTarget, async () => {
      try {
        const r = await invoke("repair_bluetooth");
        const report = await invoke("check_bluetooth");
        renderBluetoothReport(report);
        const remainingIssue = !report.healthy;
        $("bluetooth-result").insertAdjacentHTML("beforeend", renderRepairBlock(r, remainingIssue));
        showRepairOutcome(showToast, r, remainingIssue);
      } catch (err) {
        notifyOperationError(showToast, err, "repair_bluetooth");
      }
    });
  });

  $("btn-devices-scan")?.addEventListener("click", async () => {
    setPanel($("devices-result"), "loading", `<p class="result-empty">${escapeHtml(t("toolkit.loading"))}</p>`);
    try {
      renderDevicesReport(await invoke("diagnose_devices"));
    } catch (err) {
      setPanel($("devices-result"), "crit", renderOperationError(err, "diagnose_devices"));
    }
  });

  $("btn-devices-rescan")?.addEventListener("click", (event) => {
    runButtonTask(event.currentTarget, async () => {
      try {
        const result = await invoke("rescan_devices");
        renderDevicesReport(await invoke("diagnose_devices"));
        const message = result.success
          ? t("toolkit.devices.rescanDone")
          : result.needs_admin
            ? t("toolkit.devices.rescanAdmin")
            : t("toolkit.devices.rescanFailed");
        $("devices-result").insertAdjacentHTML("beforeend", `<p class="repair-verdict ${result.success ? "ok" : "warn"}">${escapeHtml(message)}</p>${result.details ? `<p class="result-muted mono advanced-only">${escapeHtml(result.details)}</p>` : ""}`);
        showToast(message, !result.success);
      } catch (err) {
        notifyOperationError(showToast, err, "rescan_devices");
      }
    });
  });

  $("devices-result")?.addEventListener("click", async (event) => {
    const button = event.target.closest(".btn-device-driver-action");
    if (!button) return;
    const row = button.closest(".toolkit-device-row");
    const problem = {
      name: row?.querySelector(".device-name")?.textContent || t("events.unknownDevice"),
      device_id: row?.dataset.deviceId,
    };
    const actionId = button.dataset.action;
    if (!(await confirmDriverAction(problem, actionId))) return;
    let infPath = null;
    if (actionId === "install_inf") {
      infPath = await open({
        multiple: false,
        directory: false,
        filters: [{ name: t("toolkit.devices.infFilter"), extensions: ["inf"] }],
      });
      if (!infPath) return;
    }
    await runButtonTask(button, async () => {
      const resultEl = row?.querySelector(".driver-repair-result");
      if (resultEl) resultEl.innerHTML = `<p class="result-muted">${escapeHtml(t("toolkit.devices.repairing"))}</p>`;
      try {
        const result = await invoke("repair_device_driver", {
          deviceId: problem.device_id,
          actionId,
          infPath,
        });
        if (resultEl) resultEl.innerHTML = renderDriverRepairResult(result);
        showToast(t(result.verified ? "toolkit.devices.result.verified" : "toolkit.devices.result.notVerified", { code: result.after_code ?? "-" }), !result.verified);
        if (result.verified) renderDevicesReport(await invoke("diagnose_devices"));
      } catch (err) {
        if (resultEl) resultEl.innerHTML = renderOperationError(err, "repair_device_driver");
        notifyOperationError(showToast, err, "repair_device_driver");
      }
    });
  });

  $("port-sort")?.addEventListener("change", () => {
    portSortMode = $("port-sort").value;
    if (portReportCache) renderPortScan(portReportCache);
  });
}
