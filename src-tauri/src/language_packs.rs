//! 与应用版本严格匹配的可选完整语言包。

use reqwest::blocking::Client;
use reqwest::header::USER_AGENT;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

const MAX_PACK_BYTES: usize = 8 * 1024 * 1024;
const RELEASE_DOWNLOAD_BASE: &str = "https://github.com/ichenh/zerotick/releases/download/";

static PACK_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn init(path: PathBuf) {
    let _ = PACK_PATH.set(path);
}

pub fn load() -> Result<Option<Value>, String> {
    let path = PACK_PATH.get().ok_or("语言包存储未初始化")?;
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read(path).map_err(|error| format!("读取语言包失败: {error}"))?;
    validate_pack(&raw).map(Some)
}

fn download_pack(client: &Client, asset_url: &str, version: &str) -> Result<Vec<u8>, String> {
    let mut last_error = String::new();
    for attempt in 1..=3 {
        match client
            .get(asset_url)
            .header(USER_AGENT, format!("ZeroTick/{version}"))
            .send()
        {
            Ok(response) if response.status().is_success() => {
                if response
                    .content_length()
                    .is_some_and(|length| length > MAX_PACK_BYTES as u64)
                {
                    return Err("语言包超过 8 MiB 安全限制".into());
                }
                match response.bytes() {
                    Ok(raw) if raw.len() <= MAX_PACK_BYTES => return Ok(raw.to_vec()),
                    Ok(_) => return Err("语言包超过 8 MiB 安全限制".into()),
                    Err(error) => last_error = format!("读取语言包失败: {error}"),
                }
            }
            Ok(response) => {
                let status = response.status();
                last_error = format!("下载语言包失败（HTTP {}）", status.as_u16());
                if status.is_client_error() && status.as_u16() != 429 {
                    return Err(last_error);
                }
            }
            Err(error) => last_error = format!("下载语言包失败: {error}"),
        }
        if attempt < 3 {
            std::thread::sleep(Duration::from_millis(400 * attempt));
        }
    }
    Err(format!("{last_error}（已重试 3 次）"))
}

pub fn install(locale: &str) -> Result<Value, String> {
    let version = env!("CARGO_PKG_VERSION");
    let locale = crate::i18n::normalize_locale(locale);
    if locale == "en" || !crate::i18n::SUPPORTED.contains(&locale.as_str()) {
        return Err("不支持的可下载语言".into());
    }
    let asset_name = format!("zerotick-language-pack-v{version}.json");
    let asset_url = format!("{RELEASE_DOWNLOAD_BASE}v{version}/{asset_name}");
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| format!("创建语言包下载客户端失败: {error}"))?;
    let raw = download_pack(&client, &asset_url, version)?;
    let pack = validate_pack(&raw)?;
    let locale_metadata = pack
        .get("locales")
        .and_then(Value::as_array)
        .and_then(|locales| {
            locales
                .iter()
                .find(|item| item.get("code").and_then(Value::as_str) == Some(locale.as_str()))
        })
        .cloned();
    let bundle = pack
        .get("bundles")
        .and_then(Value::as_object)
        .and_then(|bundles| bundles.get(&locale))
        .cloned();
    let (Some(locale_metadata), Some(bundle)) = (locale_metadata, bundle) else {
        return Err("下载的语言包与请求语言不匹配".into());
    };
    let mut selected_bundles = serde_json::Map::new();
    selected_bundles.insert(locale, bundle);
    Ok(serde_json::json!({
        "schema_version": 1,
        "app_version": version,
        "locales": [locale_metadata],
        "bundles": selected_bundles,
    }))
}

pub fn persist(pack: &Value) -> Result<(), String> {
    let raw = serde_json::to_vec(pack).map_err(|error| format!("序列化语言包失败: {error}"))?;
    validate_pack(&raw)?;
    let path = PACK_PATH.get().ok_or("语言包存储未初始化")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("创建语言包目录失败: {error}"))?;
    }
    let mut merged = if path.exists() {
        let existing = fs::read(path).map_err(|error| format!("读取已有语言包失败: {error}"))?;
        validate_pack(&existing)?
    } else {
        serde_json::json!({
            "schema_version": 1,
            "app_version": env!("CARGO_PKG_VERSION"),
            "locales": [],
            "bundles": {}
        })
    };
    let incoming_locales = pack["locales"].as_array().cloned().unwrap_or_default();
    let merged_locales = merged["locales"]
        .as_array_mut()
        .ok_or("已有语言包列表无效")?;
    for locale in incoming_locales {
        let code = locale
            .get("code")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if let Some(existing) = merged_locales
            .iter_mut()
            .find(|item| item.get("code").and_then(Value::as_str) == Some(code))
        {
            *existing = locale;
        } else {
            merged_locales.push(locale);
        }
    }
    let incoming_bundles = pack["bundles"].as_object().ok_or("语言包内容无效")?;
    let merged_bundles = merged["bundles"]
        .as_object_mut()
        .ok_or("已有语言包内容无效")?;
    for (code, bundle) in incoming_bundles {
        merged_bundles.insert(code.clone(), bundle.clone());
    }
    let merged_raw =
        serde_json::to_vec(&merged).map_err(|error| format!("序列化合并语言包失败: {error}"))?;
    validate_pack(&merged_raw)?;
    fs::write(path, merged_raw).map_err(|error| format!("保存语言包失败: {error}"))?;
    Ok(())
}

fn validate_pack(raw: &[u8]) -> Result<Value, String> {
    if raw.len() > MAX_PACK_BYTES {
        return Err("语言包超过 8 MiB 安全限制".into());
    }
    let pack: Value =
        serde_json::from_slice(raw).map_err(|error| format!("解析语言包失败: {error}"))?;
    if pack.get("schema_version").and_then(Value::as_u64) != Some(1) {
        return Err("不支持的语言包格式".into());
    }
    if pack.get("app_version").and_then(Value::as_str) != Some(env!("CARGO_PKG_VERSION")) {
        return Err(format!(
            "语言包与 ZeroTick {} 版本不匹配",
            env!("CARGO_PKG_VERSION")
        ));
    }
    if !pack.get("locales").is_some_and(Value::is_array)
        || !pack.get("bundles").is_some_and(Value::is_object)
    {
        return Err("语言包缺少语言列表或翻译内容".into());
    }
    Ok(pack)
}
