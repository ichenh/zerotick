//! 网络诊断 — 服务、网速测试、连通性、VPN

use crate::services::{self, ServicesReport, NETWORK};
use crate::utils::powershell;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Serialize)]
pub struct VpnAdapterInfo {
    pub name: String,
    pub description: String,
    pub status: String,
    pub detection: String,
}

#[derive(Debug, Serialize)]
pub struct VpnConnectionInfo {
    pub name: String,
    pub server: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct VpnReport {
    pub active: bool,
    pub tunnel_active: bool,
    pub connections: Vec<VpnConnectionInfo>,
    pub adapters: Vec<VpnAdapterInfo>,
    pub proxy: ProxyInfo,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ProxyInfo {
    pub active: bool,
    /// manual | pac | environment | combined | none
    pub mode: String,
    #[serde(default)]
    pub sources: Vec<ProxySourceInfo>,
    #[serde(default)]
    pub providers: Vec<ProxyProviderInfo>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ProxySourceInfo {
    /// manual | pac | environment
    pub kind: String,
    /// Redacted proxy endpoint or PAC location. Never includes credentials.
    pub address: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ProxyProviderInfo {
    pub name: String,
    pub pid: Option<u32>,
    pub path: Option<String>,
    /// listener | related_process
    pub evidence: String,
}

#[derive(Debug, Serialize)]
pub struct NetworkDiagReport {
    pub services: ServicesReport,
    pub gateway: Option<String>,
    pub gateway_reachable: Option<bool>,
    pub adapter_count: usize,
    pub vpn: VpnReport,
    pub dns_flush_ok: bool,
}

#[derive(Debug, Serialize)]
pub struct SpeedTestResult {
    pub bytes: u64,
    pub duration_ms: u64,
    pub speed_mbps: f64,
    pub url: String,
    pub vpn_active: bool,
}

pub fn diagnose() -> Result<NetworkDiagReport, String> {
    let (services, gateway, gateway_reachable, adapter_count, vpn) = std::thread::scope(|scope| {
        let services_task = scope.spawn(|| services::diagnose_group(NETWORK));
        let gateway_task = scope.spawn(|| {
            let gateway = detect_gateway();
            let reachable = gateway.as_ref().map(|value| ping_once(value));
            (gateway, reachable)
        });
        let adapters_task = scope.spawn(count_adapters);
        let vpn_task = scope.spawn(detect_vpn);
        let services = services_task
            .join()
            .map_err(|_| "网络服务扫描异常终止".to_string())??;
        let (gateway, gateway_reachable) = gateway_task
            .join()
            .map_err(|_| "网关扫描异常终止".to_string())?;
        let adapter_count = adapters_task
            .join()
            .map_err(|_| "网络适配器扫描异常终止".to_string())?;
        let vpn = vpn_task
            .join()
            .map_err(|_| "VPN 扫描异常终止".to_string())?;
        Ok::<_, String>((services, gateway, gateway_reachable, adapter_count, vpn))
    })?;
    Ok(NetworkDiagReport {
        services,
        gateway,
        gateway_reachable,
        adapter_count,
        vpn,
        dns_flush_ok: false,
    })
}

pub fn flush_dns() -> Result<(), String> {
    let output = Command::new("ipconfig")
        .args(["/flushdns"])
        .output()
        .map_err(|e| format!("ipconfig 失败: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

pub fn speed_test() -> Result<SpeedTestResult, String> {
    const URL: &str = "https://speed.cloudflare.com/__down?bytes=1048576";
    let vpn_active = detect_vpn().active;
    let (bytes, duration_ms) = download_speed_sample_rustls(URL)
        .or_else(|_| download_speed_sample_curl(URL, false))
        .or_else(|_| download_speed_sample_curl(URL, true))?;
    let speed_mbps = if duration_ms > 0 {
        (bytes as f64 * 8.0) / (duration_ms as f64 * 1000.0)
    } else {
        0.0
    };
    Ok(SpeedTestResult {
        bytes,
        duration_ms,
        speed_mbps,
        url: URL.into(),
        vpn_active,
    })
}

fn download_speed_sample_rustls(url: &str) -> Result<(u64, u64), String> {
    let timeout_secs = crate::settings::get().network_test_timeout_secs;
    let connect_timeout_secs = (timeout_secs / 3).clamp(2, 10);
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(connect_timeout_secs))
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .user_agent("ZeroTick speed test")
        .build()
        .map_err(|_| "speed_test:tls".to_string())?;
    let started = Instant::now();
    let response = client
        .get(url)
        .send()
        .map_err(|error| format!("speed_test:{}", reqwest_error_id(&error)))?
        .error_for_status()
        .map_err(|_| "speed_test:server".to_string())?;
    let data = response
        .bytes()
        .map_err(|error| format!("speed_test:{}", reqwest_error_id(&error)))?;
    if data.is_empty() {
        return Err("speed_test:empty".into());
    }
    Ok((
        data.len() as u64,
        started.elapsed().as_millis().max(1) as u64,
    ))
}

fn reqwest_error_id(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        return "timeout";
    }
    let detail = error.to_string().to_ascii_lowercase();
    if detail.contains("dns") || detail.contains("resolve") {
        "dns"
    } else if detail.contains("certificate") || detail.contains("tls") || detail.contains("peer") {
        "tls"
    } else if error.is_connect() {
        "connect"
    } else {
        "transfer"
    }
}

fn download_speed_sample_curl(url: &str, ipv4_only: bool) -> Result<(u64, u64), String> {
    let started = Instant::now();
    let timeout_secs = crate::settings::get().network_test_timeout_secs;
    let connect_timeout_secs = (timeout_secs / 3).clamp(2, 10);
    let connect_timeout_arg = connect_timeout_secs.to_string();
    let timeout_arg = timeout_secs.to_string();
    let mut command = Command::new("curl.exe");
    command.args([
        "--location",
        "--fail",
        "--silent",
        "--show-error",
        "--connect-timeout",
        &connect_timeout_arg,
        "--max-time",
        &timeout_arg,
        "--output",
        "-",
    ]);
    if ipv4_only {
        command.arg("--ipv4");
    }
    let output = command.arg(url).output().map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            "speed_test:curl_missing".to_string()
        } else {
            "speed_test:launch_failed".to_string()
        }
    })?;
    if !output.status.success() {
        return Err(format!(
            "speed_test:{}",
            curl_error_id(output.status.code())
        ));
    }
    if output.stdout.is_empty() {
        return Err("speed_test:empty".into());
    }
    Ok((
        output.stdout.len() as u64,
        started.elapsed().as_millis().max(1) as u64,
    ))
}

fn curl_error_id(code: Option<i32>) -> &'static str {
    match code {
        Some(5) => "proxy_dns",
        Some(6) => "dns",
        Some(7) => "connect",
        Some(28) => "timeout",
        Some(35 | 51 | 58 | 59 | 60 | 77 | 80 | 82 | 83 | 90 | 91) => "tls",
        Some(47) => "redirect",
        Some(52 | 55 | 56 | 92) => "transfer",
        _ => "unknown",
    }
}

pub fn repair() -> (Vec<String>, Vec<String>) {
    services::repair_group(NETWORK)
}

fn detect_vpn() -> VpnReport {
    let script = r#"
$pattern = 'VPN|TAP-Windows|TUN|WireGuard|Wintun|OpenVPN|ZeroTier|Tailscale|NordLynx|Cisco|AnyConnect|PANGP|GlobalProtect|Juniper|Pulse Secure|Fortinet|Cloudflare|WARP|Clash|Mihomo|sing-box|V2Ray|Xray|Outline|Hiddify|Neko|SSTP|IKEv2|L2TP|PPTP'
$platformPattern = 'Hyper-V|VMware|VirtualBox|WSL|Docker|vEthernet|Loopback|Npcap'
$defaultIndexes = @(Get-NetRoute -ErrorAction SilentlyContinue | Where-Object {
  $_.DestinationPrefix -in @('0.0.0.0/0', '::/0')
} | Select-Object -ExpandProperty InterfaceIndex -Unique)
$adapters = @(Get-NetAdapter -IncludeHidden -ErrorAction SilentlyContinue | Where-Object {
  if ($_.Status -ne 'Up') { return $false }
  $namedTunnel = $_.InterfaceDescription -match $pattern -or $_.Name -match $pattern
  $defaultVirtual = $defaultIndexes -contains $_.ifIndex -and $_.HardwareInterface -eq $false -and $_.InterfaceDescription -notmatch $platformPattern -and $_.Name -notmatch $platformPattern
  $namedTunnel -or $defaultVirtual
} | ForEach-Object {
  $namedTunnel = $_.InterfaceDescription -match $pattern -or $_.Name -match $pattern
  [pscustomobject]@{
    name = $_.Name
    description = $_.InterfaceDescription
    status = $_.Status
    detection = if ($namedTunnel) { 'tunnel_name' } else { 'default_virtual_route' }
  }
})
$vpnConns = @()
if (Get-Command Get-VpnConnection -ErrorAction SilentlyContinue) {
  $vpnConns = @(
    Get-VpnConnection -ErrorAction SilentlyContinue
    Get-VpnConnection -AllUserConnection -ErrorAction SilentlyContinue
  ) | Where-Object { $_.ConnectionStatus -eq 'Connected' } | Sort-Object Name -Unique
  $vpnConns = @($vpnConns | ForEach-Object {
    [pscustomobject]@{ name = $_.Name; server = $_.ServerAddress; status = [string]$_.ConnectionStatus }
  })
}
$inet = Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Internet Settings' -ErrorAction SilentlyContinue
$manualProxy = [bool]$inet.ProxyEnable -and -not [string]::IsNullOrWhiteSpace([string]$inet.ProxyServer)
$pacProxy = -not [string]::IsNullOrWhiteSpace([string]$inet.AutoConfigURL)
$envProxies = @($env:HTTPS_PROXY, $env:HTTP_PROXY, $env:ALL_PROXY) | Where-Object { -not [string]::IsNullOrWhiteSpace([string]$_) } | Select-Object -Unique
$proxyModes = @()
if ($manualProxy) { $proxyModes += 'manual' }
if ($pacProxy) { $proxyModes += 'pac' }
if ($envProxies.Count -gt 0) { $proxyModes += 'environment' }
$proxyActive = $proxyModes.Count -gt 0
$proxyMode = if ($proxyModes.Count -gt 1) { 'combined' } elseif ($proxyModes.Count -eq 1) { $proxyModes[0] } else { 'none' }

function ConvertTo-ProxyEndpoint($value) {
  $items = @()
  foreach ($part in @(([string]$value) -split ';')) {
    $candidate = $part.Trim()
    if (-not $candidate) { continue }
    if ($candidate.Contains('=')) { $candidate = $candidate.Substring($candidate.IndexOf('=') + 1).Trim() }
    $uriText = if ($candidate -match '^[a-zA-Z][a-zA-Z0-9+.-]*://') { $candidate } else { "http://$candidate" }
    try {
      $uri = [Uri]$uriText
      if (-not $uri.Host) { continue }
      $port = if ($uri.IsDefaultPort) { if ($uri.Scheme -eq 'https') { 443 } else { 80 } } else { $uri.Port }
      $hostText = if ($uri.Host.Contains(':')) { "[$($uri.Host)]" } else { $uri.Host }
      $items += [pscustomobject]@{
        address = "$($uri.Scheme)://$hostText`:$port"
        host = $uri.Host
        port = $port
      }
    } catch { }
  }
  $items
}

function Get-RedactedPacAddress($value) {
  try {
    $uri = [Uri]([string]$value)
    if (-not $uri.Host) { return 'PAC script' }
    $port = if ($uri.IsDefaultPort) { '' } else { ":$($uri.Port)" }
    "$($uri.Scheme)://$($uri.Host)$port$($uri.AbsolutePath)"
  } catch { 'PAC script' }
}

$proxySources = @()
$proxyEndpoints = @()
if ($manualProxy) {
  $endpoints = @(ConvertTo-ProxyEndpoint $inet.ProxyServer)
  $proxyEndpoints += $endpoints
  $proxySources += @($endpoints | ForEach-Object { [pscustomobject]@{ kind = 'manual'; address = $_.address } })
}
if ($pacProxy) {
  $proxySources += [pscustomobject]@{ kind = 'pac'; address = Get-RedactedPacAddress $inet.AutoConfigURL }
  try {
    $pacUri = [Uri]([string]$inet.AutoConfigURL)
    if ($pacUri.Host) {
      $pacPort = if ($pacUri.IsDefaultPort) { if ($pacUri.Scheme -eq 'https') { 443 } else { 80 } } else { $pacUri.Port }
      $proxyEndpoints += [pscustomobject]@{ address = Get-RedactedPacAddress $inet.AutoConfigURL; host = $pacUri.Host; port = $pacPort }
    }
  } catch { }
}
foreach ($envProxy in $envProxies) {
  $endpoints = @(ConvertTo-ProxyEndpoint $envProxy)
  $proxyEndpoints += $endpoints
  $proxySources += @($endpoints | ForEach-Object { [pscustomobject]@{ kind = 'environment'; address = $_.address } })
}
$proxySources = @($proxySources | Sort-Object kind,address -Unique)

$proxyProviders = @()
$localHosts = @('127.0.0.1', 'localhost', '::1') + @(Get-NetIPAddress -ErrorAction SilentlyContinue | Select-Object -ExpandProperty IPAddress)
foreach ($endpoint in @($proxyEndpoints | Where-Object { $localHosts -contains $_.host } | Sort-Object port -Unique)) {
  $listeners = @(& "$env:SystemRoot\System32\netstat.exe" -ano -p tcp 2>$null | ForEach-Object {
    if ($_ -match "^\s*TCP\s+\S+:$($endpoint.port)\s+\S+\s+LISTENING\s+(\d+)\s*$") {
      [uint32]$Matches[1]
    }
  } | Select-Object -Unique)
  foreach ($ownerPid in $listeners) {
    $process = Get-Process -Id $ownerPid -ErrorAction SilentlyContinue
    if (-not $process) { continue }
    $displayName = $process.ProcessName
    $processPath = try { $process.Path } catch { $null }
    if ($processPath) {
      try {
        $version = (Get-Item -LiteralPath $processPath -ErrorAction Stop).VersionInfo
        if ($version.FileDescription) { $displayName = $version.FileDescription }
        elseif ($version.ProductName) { $displayName = $version.ProductName }
      } catch { }
    }
    $proxyProviders += [pscustomobject]@{
      name = $displayName
      pid = [uint32]$ownerPid
      path = $processPath
      evidence = 'listener'
    }
  }
}
if ($proxyActive -and $proxyProviders.Count -eq 0) {
  $knownProxyPattern = 'clash|mihomo|v2ray|xray|sing-box|nekoray|hiddify|shadowsocks|outline|warp|proxifier|surge|trojan|naiveproxy'
  $proxyProviders = @(Get-Process -ErrorAction SilentlyContinue | Where-Object {
    $_.ProcessName -match $knownProxyPattern
  } | ForEach-Object {
    $candidatePath = try { $_.Path } catch { $null }
    [pscustomobject]@{ name = $_.ProcessName; pid = [uint32]$_.Id; path = $candidatePath; evidence = 'related_process' }
  })
}
$proxyProviders = @($proxyProviders | Sort-Object pid -Unique)
$tunnelActive = ($adapters.Count -gt 0) -or ($vpnConns.Count -gt 0)
[pscustomobject]@{
  active = $tunnelActive -or $proxyActive
  tunnel_active = $tunnelActive
  connections = $vpnConns
  adapters = $adapters
  proxy = [pscustomobject]@{
    active = $proxyActive
    mode = $proxyMode
    sources = $proxySources
    providers = $proxyProviders
  }
}
"#;
    powershell::run_json(script)
        .ok()
        .map(parse_vpn_json)
        .unwrap_or_default()
}

fn parse_vpn_json(v: serde_json::Value) -> VpnReport {
    let connections: Vec<VpnConnectionInfo> = v
        .get("connections")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(VpnConnectionInfo {
                        name: item.get("name")?.as_str()?.to_string(),
                        server: item
                            .get("server")
                            .and_then(|s| s.as_str())
                            .map(str::to_string),
                        status: item
                            .get("status")
                            .and_then(|s| s.as_str())
                            .unwrap_or("Connected")
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let adapters: Vec<VpnAdapterInfo> = v
        .get("adapters")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    Some(VpnAdapterInfo {
                        name: item.get("name")?.as_str()?.to_string(),
                        description: item
                            .get("description")
                            .and_then(|s| s.as_str())
                            .unwrap_or("")
                            .to_string(),
                        status: item
                            .get("status")
                            .and_then(|s| s.as_str())
                            .unwrap_or("Up")
                            .to_string(),
                        detection: item
                            .get("detection")
                            .and_then(|s| s.as_str())
                            .unwrap_or("tunnel_name")
                            .to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let proxy = v
        .get("proxy")
        .cloned()
        .and_then(|value| serde_json::from_value::<ProxyInfo>(value).ok())
        .unwrap_or_default();
    let tunnel_active = v
        .get("tunnel_active")
        .and_then(|b| b.as_bool())
        .unwrap_or(!connections.is_empty() || !adapters.is_empty());
    let active = v
        .get("active")
        .and_then(|b| b.as_bool())
        .unwrap_or(tunnel_active || proxy.active);

    VpnReport {
        active,
        tunnel_active,
        connections,
        adapters,
        proxy,
    }
}

impl Default for VpnReport {
    fn default() -> Self {
        Self {
            active: false,
            tunnel_active: false,
            connections: Vec::new(),
            adapters: Vec::new(),
            proxy: ProxyInfo {
                active: false,
                mode: "none".into(),
                sources: Vec::new(),
                providers: Vec::new(),
            },
        }
    }
}

fn detect_gateway() -> Option<String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "(Get-NetRoute -DestinationPrefix '0.0.0.0/0' | Sort-Object RouteMetric | Select-Object -First 1).NextHop",
        ])
        .output()
        .ok()?;
    let ip = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if ip.is_empty() || ip == "0.0.0.0" {
        None
    } else {
        Some(ip)
    }
}

fn ping_once(host: &str) -> bool {
    Command::new("ping")
        .args(["-n", "1", "-w", "1500", host])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn count_adapters() -> usize {
    let script = "(@(Get-NetAdapter | Where-Object Status -eq 'Up')).Count";
    powershell::run_json(script)
        .ok()
        .and_then(|v| v.as_u64().map(|n| n as usize))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::{curl_error_id, parse_vpn_json};

    #[test]
    fn curl_failures_map_to_actionable_error_ids() {
        assert_eq!(curl_error_id(Some(5)), "proxy_dns");
        assert_eq!(curl_error_id(Some(28)), "timeout");
        assert_eq!(curl_error_id(Some(60)), "tls");
        assert_eq!(curl_error_id(Some(56)), "transfer");
    }

    #[test]
    fn proxy_provider_details_survive_json_parsing() {
        let report = parse_vpn_json(serde_json::json!({
            "active": true,
            "tunnel_active": false,
            "connections": [],
            "adapters": [],
            "proxy": {
                "active": true,
                "mode": "manual",
                "sources": [{ "kind": "manual", "address": "http://127.0.0.1:7890" }],
                "providers": [{
                    "name": "Example Proxy",
                    "pid": 42,
                    "path": "C:\\Example\\proxy.exe",
                    "evidence": "listener"
                }]
            }
        }));
        assert_eq!(report.proxy.sources.len(), 1);
        assert_eq!(report.proxy.providers.len(), 1);
        assert_eq!(report.proxy.providers[0].pid, Some(42));
        assert_eq!(report.proxy.providers[0].evidence, "listener");
    }
}
