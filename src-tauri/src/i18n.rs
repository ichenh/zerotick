//! Backend user-visible strings for tray, notifications, and dialogs (per locale).

use crate::events::{BluetoothIssue, DeviceEvent};

use std::collections::HashMap;

use std::sync::OnceLock;

pub const SUPPORTED: &[&str] = &[
    "en", "zh-CN", "zh-TW", "ja", "ko", "de", "fr", "es", "pt-BR", "ru", "ar", "hi", "it", "nl",
    "pl", "tr", "vi", "th", "id", "cs", "da", "fi", "nb", "sv", "uk", "he", "ms", "ro", "hu",
];

static TRAY_STRINGS: OnceLock<HashMap<String, HashMap<String, String>>> = OnceLock::new();

fn tray_strings() -> &'static HashMap<String, HashMap<String, String>> {
    TRAY_STRINGS.get_or_init(|| {
        serde_json::from_str(include_str!("../locales/tray.json")).unwrap_or_default()
    })
}

/// Normalize a BCP 47 language tag.
pub fn normalize_locale(raw: &str) -> String {
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return "en".into();
    }

    let mapped = match trimmed {
        "zh" | "zh-Hans" | "zh-CN" => "zh-CN",

        "zh-Hant" | "zh-TW" | "zh-HK" => "zh-TW",

        "pt" => "pt-BR",

        "pt-PT" => "pt-PT",

        "nb-NO" | "no" => "nb",

        "sv-SE" => "sv",

        "da-DK" => "da",

        "fi-FI" => "fi",

        "uk-UA" => "uk",

        "he-IL" => "he",

        "id-ID" => "id",

        "ms-MY" => "ms",

        "vi-VN" => "vi",

        "th-TH" => "th",

        "tr-TR" => "tr",

        "pl-PL" => "pl",

        "nl-NL" => "nl",

        "it-IT" => "it",

        "hi-IN" => "hi",

        "ar-SA" | "ar" => "ar",

        "ru-RU" => "ru",

        "ja-JP" => "ja",

        "ko-KR" => "ko",

        "de-DE" => "de",

        "fr-FR" => "fr",

        "es-ES" | "es-MX" => "es",

        "ro-RO" => "ro",

        "hu-HU" => "hu",

        "cs-CZ" => "cs",

        other => other,
    };

    if SUPPORTED.contains(&mapped) {
        mapped.to_string()
    } else if let Some(base) = mapped.split('-').next() {
        if SUPPORTED.contains(&base) {
            base.to_string()
        } else {
            "en".into()
        }
    } else {
        "en".into()
    }
}

pub fn is_supported(locale: &str) -> bool {
    SUPPORTED.contains(&normalize_locale(locale).as_str())
}

fn pick(locale: &str, key: &str, en_fallback: &str) -> String {
    let loc = normalize_locale(locale);

    let map = tray_strings();

    if let Some(loc_map) = map.get(loc.as_str()) {
        if let Some(s) = loc_map.get(key) {
            return s.clone();
        }
    }

    if let Some(en_map) = map.get("en") {
        if let Some(s) = en_map.get(key) {
            return s.clone();
        }
    }

    en_fallback.to_string()
}

fn interpolate(template: &str, pairs: &[(&str, String)]) -> String {
    let mut out = template.to_string();

    for (k, v) in pairs {
        out = out.replace(&format!("{{{k}}}"), v);
    }

    out
}

pub fn tray_show(locale: &str) -> String {
    pick(locale, "show", "Open Dashboard")
}

pub fn tray_quit(locale: &str) -> String {
    pick(locale, "quit", "Quit ZeroTick")
}

pub fn tray_tooltip_normal(locale: &str) -> String {
    pick(locale, "tooltip_normal", "ZeroTick — Monitoring OK")
}

pub fn tray_tooltip_warning(locale: &str, reason: &str) -> String {
    let tpl = pick(
        locale,
        "tooltip_warning",
        "ZeroTick — Device fluctuation · {reason}",
    );

    tpl.replace("{reason}", reason)
}

pub fn tray_tooltip_critical(locale: &str, reason: &str) -> String {
    let tpl = pick(locale, "tooltip_critical", "ZeroTick — Alert · {reason}");

    tpl.replace("{reason}", reason)
}

pub fn tray_reason(locale: &str, reason_id: &str) -> String {
    let key = format!("reason_{reason_id}");

    pick(locale, &key, reason_id)
}

pub fn notify_repair_title(locale: &str) -> String {
    pick(locale, "notify_repair", "ZeroTick — Repair complete")
}

pub fn notify_bluetooth_title(locale: &str) -> String {
    pick(locale, "notify_bluetooth", "ZeroTick — Bluetooth issue")
}

pub fn notify_bsod_title(locale: &str) -> String {
    pick(locale, "notify_bsod", "ZeroTick — BSOD alert")
}

pub fn notify_transient_title(locale: &str) -> String {
    pick(
        locale,
        "notify_transient",
        "ZeroTick — Transient disconnect",
    )
}

pub fn notify_disconnect_title(locale: &str) -> String {
    pick(
        locale,
        "notify_disconnect",
        "ZeroTick — Device disconnected",
    )
}

pub fn format_device_notify(locale: &str, event_type: &str, event: &DeviceEvent) -> String {
    let name = event
        .friendly_name
        .as_deref()
        .or(event.vid_pid.as_deref())
        .unwrap_or("Unknown");

    let key = match event_type {
        "transient_reconnect" => "notify_body_transient",

        "remove" => "notify_body_remove",

        _ => return name.to_string(),
    };

    let tpl = pick(locale, key, "{name} ({ms}ms)");

    interpolate(
        &tpl,
        &[
            ("name", name.to_string()),
            ("ms", event.disconnect_ms.unwrap_or(0).to_string()),
        ],
    )
}

pub fn format_bluetooth_issue(locale: &str, issue: &BluetoothIssue) -> String {
    let key = format!("issue_{}", issue.id);

    let tpl = pick(locale, &key, &issue.id);

    interpolate(
        &tpl,
        &[
            (
                "name",
                issue.name.clone().unwrap_or_else(|| "Unknown".into()),
            ),
            ("state", issue.state.clone().unwrap_or_else(|| "—".into())),
            (
                "code",
                issue
                    .code
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "—".into()),
            ),
        ],
    )
}

pub fn repair_summary(locale: &str, summary_id: &str, count: Option<usize>) -> String {
    let key = format!("repair_{summary_id}");

    let tpl = pick(locale, &key, summary_id);

    interpolate(&tpl, &[("n", count.unwrap_or(0).to_string())])
}

pub fn export_dialog_title(locale: &str) -> String {
    pick(locale, "export_title", "Export device history")
}

pub fn export_error(locale: &str, code: &str) -> String {
    let key = format!("export_{code}");

    pick(locale, &key, code)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]

    fn normalizes_zh_aliases() {
        assert_eq!(normalize_locale("zh-Hans"), "zh-CN");

        assert_eq!(normalize_locale("zh-Hant"), "zh-TW");
    }
}
