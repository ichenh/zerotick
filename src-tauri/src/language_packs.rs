//! 与应用版本严格匹配的可选完整语言包。

use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Duration;

const MAX_PACK_BYTES: usize = 8 * 1024 * 1024;
const RELEASE_BY_TAG_API: &str = "https://api.github.com/repos/ichenh/zerotick/releases/tags/";

static PACK_PATH: OnceLock<PathBuf> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

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

pub fn install(locale: &str) -> Result<Value, String> {
    let version = env!("CARGO_PKG_VERSION");
    let locale = crate::i18n::normalize_locale(locale);
    if locale == "en" || !crate::i18n::SUPPORTED.contains(&locale.as_str()) {
        return Err("不支持的可下载语言".into());
    }
    let asset_name = format!("zerotick-language-{locale}-v{version}.json");
    let client = Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| format!("创建语言包下载客户端失败: {error}"))?;
    let release_url = format!("{RELEASE_BY_TAG_API}v{version}");
    let response = client
        .get(release_url)
        .header(USER_AGENT, format!("ZeroTick/{version}"))
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .map_err(|error| format!("无法连接 GitHub Releases: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "未找到 ZeroTick v{version} 的 GitHub Release（HTTP {}）",
            response.status().as_u16()
        ));
    }
    let release_raw = response
        .text()
        .map_err(|error| format!("读取 GitHub Release 响应失败: {error}"))?;
    let release: GitHubRelease = serde_json::from_str(&release_raw)
        .map_err(|error| format!("解析 GitHub Release 失败: {error}"))?;
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| format!("该版本尚未提供语言包 {asset_name}"))?;
    let response = client
        .get(&asset.browser_download_url)
        .header(USER_AGENT, format!("ZeroTick/{version}"))
        .send()
        .map_err(|error| format!("下载语言包失败: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "下载语言包失败（HTTP {}）",
            response.status().as_u16()
        ));
    }
    if response
        .content_length()
        .is_some_and(|length| length > MAX_PACK_BYTES as u64)
    {
        return Err("语言包超过 8 MiB 安全限制".into());
    }
    let raw = response
        .bytes()
        .map_err(|error| format!("读取语言包失败: {error}"))?;
    if raw.len() > MAX_PACK_BYTES {
        return Err("语言包超过 8 MiB 安全限制".into());
    }
    let pack = validate_pack(&raw)?;
    let contains_requested_locale =
        pack.get("locales")
            .and_then(Value::as_array)
            .is_some_and(|locales| {
                locales
                    .iter()
                    .any(|item| item.get("code").and_then(Value::as_str) == Some(locale.as_str()))
            });
    if !contains_requested_locale {
        return Err("下载的语言包与请求语言不匹配".into());
    }
    Ok(pack)
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
