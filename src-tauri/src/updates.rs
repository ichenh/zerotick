//! GitHub Release 更新检查与受限外部链接打开。

use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

const LATEST_RELEASE_API: &str = "https://api.github.com/repos/ichenh/zerotick/releases/latest";
const REPOSITORY_URL: &str = "https://github.com/ichenh/zerotick";
const OFFICIAL_WEBSITE_URL: &str = "https://physchen.com";
const SUPPORT_EMAIL_URL: &str = "mailto:support@physchen.com";
const CACHE_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, Serialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_name: String,
    pub release_url: String,
    pub download_url: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    html_url: String,
    published_at: Option<String>,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

static CACHE: OnceLock<Mutex<Option<(Instant, UpdateInfo)>>> = OnceLock::new();

fn cache() -> &'static Mutex<Option<(Instant, UpdateInfo)>> {
    CACHE.get_or_init(|| Mutex::new(None))
}

pub fn check(force: bool) -> Result<UpdateInfo, String> {
    if !force {
        if let Ok(guard) = cache().lock() {
            if let Some((checked_at, info)) = guard.as_ref() {
                if checked_at.elapsed() < CACHE_TTL {
                    return Ok(info.clone());
                }
            }
        }
    }

    let client = Client::builder()
        .connect_timeout(Duration::from_secs(8))
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| format!("创建更新检查客户端失败: {error}"))?;
    let response = client
        .get(LATEST_RELEASE_API)
        .header(
            USER_AGENT,
            format!("ZeroTick/{}", env!("CARGO_PKG_VERSION")),
        )
        .header(ACCEPT, "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .map_err(|error| format!("无法连接 GitHub Releases: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub Releases 返回 HTTP {}",
            response.status().as_u16()
        ));
    }

    let raw = response
        .text()
        .map_err(|error| format!("读取 GitHub Release 响应失败: {error}"))?;
    let release: GitHubRelease =
        serde_json::from_str(&raw).map_err(|error| format!("解析 GitHub Release 失败: {error}"))?;
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let latest_version = release.tag_name.trim_start_matches(['v', 'V']).to_string();
    let update_available = is_newer_version(&latest_version, &current_version)?;
    let download_url = preferred_windows_installer(&release.assets);
    let info = UpdateInfo {
        current_version,
        latest_version,
        update_available,
        release_name: release
            .name
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(|| release.tag_name.clone()),
        release_url: release.html_url,
        download_url,
        published_at: release.published_at,
    };

    if let Ok(mut guard) = cache().lock() {
        *guard = Some((Instant::now(), info.clone()));
    }
    Ok(info)
}

fn preferred_windows_installer(assets: &[GitHubAsset]) -> Option<String> {
    assets
        .iter()
        .filter(|asset| asset.name.to_ascii_lowercase().ends_with(".exe"))
        .min_by_key(|asset| {
            let name = asset.name.to_ascii_lowercase();
            if name.contains("setup") || name.contains("installer") {
                0
            } else {
                1
            }
        })
        .or_else(|| {
            assets
                .iter()
                .find(|asset| asset.name.to_ascii_lowercase().ends_with(".msi"))
        })
        .map(|asset| asset.browser_download_url.clone())
}

fn is_newer_version(candidate: &str, current: &str) -> Result<bool, String> {
    Ok(parse_version(candidate)? > parse_version(current)?)
}

fn parse_version(value: &str) -> Result<(u64, u64, u64), String> {
    let stable = value
        .trim()
        .trim_start_matches(['v', 'V'])
        .split(['-', '+'])
        .next()
        .unwrap_or_default();
    let parts = stable
        .split('.')
        .map(|part| part.parse::<u64>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| format!("无法识别版本号: {value}"))?;
    match parts.as_slice() {
        [major, minor, patch] => Ok((*major, *minor, *patch)),
        _ => Err(format!("无法识别版本号: {value}")),
    }
}

pub fn open_project_url(app: &AppHandle, url: &str) -> Result<(), String> {
    let allowed = is_allowed_external_url(url);
    if !allowed {
        return Err("只允许打开 ZeroTick 官网、官方 GitHub 链接或支持邮箱".into());
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|error| format!("打开浏览器失败: {error}"))
}

fn is_allowed_external_url(url: &str) -> bool {
    url == SUPPORT_EMAIL_URL
        || url == OFFICIAL_WEBSITE_URL
        || url.starts_with(&format!("{OFFICIAL_WEBSITE_URL}/"))
        || url == REPOSITORY_URL
        || url.starts_with(&format!("{REPOSITORY_URL}/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_stable_versions_numerically() {
        assert!(is_newer_version("0.2.10", "0.2.4").unwrap());
        assert!(!is_newer_version("v0.2.4", "0.2.4").unwrap());
        assert!(!is_newer_version("0.1.99", "0.2.4").unwrap());
    }

    #[test]
    fn rejects_non_project_urls() {
        assert!(parse_version("latest").is_err());
        assert!(!is_allowed_external_url(
            "https://github.com/ichenh/zerotick.evil.example"
        ));
        assert!(is_allowed_external_url(SUPPORT_EMAIL_URL));
        assert!(is_allowed_external_url(OFFICIAL_WEBSITE_URL));
        assert!(!is_allowed_external_url(
            "https://physchen.com.evil.example"
        ));
    }
}
