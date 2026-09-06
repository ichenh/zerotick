//! 网络诊断 — 服务、网速测试、连通性、VPN

mod adapters;
mod icmp;
pub mod probe;
pub use adapters::NetworkAdapter;

use crate::services::{self, ServicesReport, NETWORK};
use crate::utils::logging;
use crate::utils::powershell;
use crate::utils::process::CommandExt;
use serde::{Deserialize, Serialize};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use windows::Win32::Foundation::RPC_E_CHANGED_MODE;
use windows::Win32::Networking::NetworkListManager::{INetworkListManager, NetworkListManager};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED,
};

#[derive(Debug, Clone, Serialize)]
pub struct VpnAdapterInfo {
    pub name: String,
    pub description: String,
    pub status: String,
    pub detection: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VpnConnectionInfo {
    pub name: String,
    pub server: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VpnReport {
    pub active: bool,
    pub tunnel_active: bool,
    pub connections: Vec<VpnConnectionInfo>,
    pub adapters: Vec<VpnAdapterInfo>,
    pub proxy: ProxyInfo,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProxyInfo {
    pub active: bool,
    /// manual | pac | environment | combined | none
    pub mode: String,
    #[serde(default)]
    pub sources: Vec<ProxySourceInfo>,
    #[serde(default)]
    pub providers: Vec<ProxyProviderInfo>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProxySourceInfo {
    /// manual | pac | environment
    pub kind: String,
    /// Redacted proxy endpoint or PAC location. Never includes credentials.
    pub address: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProxyProviderInfo {
    pub name: String,
    pub pid: Option<u32>,
    pub path: Option<String>,
    /// listener | related_process
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkDiagReport {
    pub services: ServicesReport,
    pub service_error: Option<String>,
    pub adapters: Vec<NetworkAdapter>,
    pub adapter_error: Option<String>,
    /// Windows Network List Manager currently sees a network connection.
    pub network_connected: Option<bool>,
    /// Windows NCSI currently reports IPv4 or IPv6 Internet access.
    pub internet_reachable: Option<bool>,
    pub gateway: Option<String>,
    pub gateway_reachable: Option<bool>,
    pub gateway_checks: Vec<GatewayCheck>,
    /// Adapters currently present in Windows, including disconnected or disabled adapters.
    pub adapter_present_count: usize,
    /// Adapters whose current operational status is Up.
    pub adapter_count: usize,
    pub vpn: Option<VpnReport>,
    pub vpn_error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GatewayCheck {
    pub gateway: String,
    /// ICMP response evidence only. No reply does not establish a broken router.
    pub reachable: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct SpeedTestResult {
    pub bytes: u64,
    pub duration_ms: u64,
    pub speed_mbps: f64,
    pub url: String,
    pub vpn_active: Option<bool>,
}

#[derive(Default)]
struct DiagnosticFlight {
    result: Mutex<Option<Result<NetworkDiagReport, String>>>,
    completed: Condvar,
}

static ACTIVE_DIAGNOSIS: Mutex<Option<Arc<DiagnosticFlight>>> = Mutex::new(None);

pub fn diagnose() -> Result<NetworkDiagReport, String> {
    diagnose_with_freshness(false)
}

fn diagnose_with_freshness(require_fresh: bool) -> Result<NetworkDiagReport, String> {
    loop {
        let (flight, leader) = {
            let mut active = ACTIVE_DIAGNOSIS
                .lock()
                .map_err(|_| "network_scan:lock_failed")?;
            match active.as_ref() {
                Some(flight) => (flight.clone(), false),
                None => {
                    if require_fresh {
                        // Reserve this new flight and invalidate under the same lock:
                        // another caller cannot publish an older snapshot between them.
                        services::invalidate_diagnostic_cache();
                    }
                    let flight = Arc::new(DiagnosticFlight::default());
                    *active = Some(flight.clone());
                    (flight, true)
                }
            }
        };
        if leader {
            let result = std::panic::catch_unwind(diagnose_uncached)
                .unwrap_or_else(|_| Err("network_scan:worker_failed".into()));
            *flight
                .result
                .lock()
                .map_err(|_| "network_scan:lock_failed")? = Some(result);
            // Only overlapping scans share this result. A later scan starts fresh.
            *ACTIVE_DIAGNOSIS
                .lock()
                .map_err(|_| "network_scan:lock_failed")? = None;
            flight.completed.notify_all();
        }
        let mut result = flight
            .result
            .lock()
            .map_err(|_| "network_scan:lock_failed")?;
        while result.is_none() {
            result = flight
                .completed
                .wait(result)
                .map_err(|_| "network_scan:lock_failed")?;
        }
        if require_fresh && !leader {
            // An overlapping scan may have started before the repair. Wait for it,
            // then reserve a new generation rather than returning its evidence.
            drop(result);
            continue;
        }
        return result
            .as_ref()
            .expect("completed network scan has a result")
            .clone();
    }
}

/// A repair must observe a new snapshot, never join a scan started before it.
pub fn diagnose_after_repair() -> Result<NetworkDiagReport, String> {
    diagnose_with_freshness(true)
}

fn diagnose_uncached() -> Result<NetworkDiagReport, String> {
    let (services, connectivity, adapters, vpn) = std::thread::scope(|scope| {
        let services_task = scope.spawn(|| services::diagnose_group(NETWORK));
        let connectivity_task = scope.spawn(detect_windows_connectivity);
        let adapters_task = scope.spawn(adapters::snapshot);
        let vpn_task = scope.spawn(detect_vpn);
        let services = services_task
            .join()
            .unwrap_or_else(|_| Err("网络服务扫描异常终止".into()));
        let connectivity = connectivity_task
            .join()
            .unwrap_or_else(|_| Err("网络连通性扫描异常终止".into()));
        let adapters = adapters_task
            .join()
            .unwrap_or_else(|_| Err("网络适配器扫描异常终止".into()));
        let vpn = vpn_task
            .join()
            .unwrap_or_else(|_| Err("VPN 扫描异常终止".into()));
        (services, connectivity, adapters, vpn)
    });
    let (services, service_error) = match services {
        Ok(report) => (report, None),
        Err(error) => (ServicesReport::default(), Some(error)),
    };
    let (adapters, adapter_error) = match adapters {
        Ok(adapters) => (adapters, None),
        Err(error) => (Vec::new(), Some(error)),
    };
    let (vpn, vpn_error) = match vpn {
        Ok(report) => (Some(report), None),
        Err(error) => (None, Some(error)),
    };
    let (network_connected, internet_reachable) = match connectivity {
        Ok(status) => (Some(status.connected), Some(status.internet)),
        Err(error) => {
            logging::warn(format!("Windows 网络连通性读取失败: {error}"));
            (None, None)
        }
    };
    for error in [&service_error, &adapter_error, &vpn_error]
        .into_iter()
        .flatten()
    {
        logging::warn(error);
    }
    let gateway_checks = check_gateways(&adapters);
    // Multiple interfaces/VPNs can have different valid gateways. Do not pick
    // an arbitrary first route and present its failure as the whole network.
    let gateway = (gateway_checks.len() == 1).then(|| gateway_checks[0].gateway.clone());
    let gateway_reachable = if gateway_checks.len() == 1 {
        gateway_checks[0].reachable
    } else {
        None
    };
    Ok(NetworkDiagReport {
        services,
        service_error,
        network_connected,
        internet_reachable,
        gateway,
        gateway_reachable,
        gateway_checks,
        adapter_present_count: adapters
            .iter()
            .filter(|adapter| adapter.oper_status != "not_present")
            .count(),
        adapter_count: adapters
            .iter()
            .filter(|adapter| adapter.oper_status == "up")
            .count(),
        adapters,
        adapter_error,
        vpn,
        vpn_error,
    })
}

#[derive(Debug, PartialEq, Eq)]
struct WindowsConnectivity {
    connected: bool,
    internet: bool,
}

fn detect_windows_connectivity() -> Result<WindowsConnectivity, String> {
    unsafe {
        let init = CoInitializeEx(None, COINIT_MULTITHREADED);
        let uninitialize_com = if init.is_ok() {
            true
        } else if init == RPC_E_CHANGED_MODE {
            false
        } else {
            return Err(format!("network_connectivity:com_init_failed:{init:?}"));
        };

        let result = (|| {
            let manager: INetworkListManager =
                CoCreateInstance(&NetworkListManager, None, CLSCTX_ALL)
                    .map_err(|error| format!("network_connectivity:create_failed:{error}"))?;
            let connected = manager
                .IsConnected()
                .map_err(|error| format!("network_connectivity:connected_failed:{error}"))?
                .as_bool();
            let internet = manager
                .IsConnectedToInternet()
                .map_err(|error| format!("network_connectivity:internet_failed:{error}"))?
                .as_bool();
            Ok(WindowsConnectivity {
                connected,
                internet,
            })
        })();

        if uninitialize_com {
            CoUninitialize();
        }
        result
    }
}

pub fn flush_dns() -> Result<(), String> {
    // ipconfig /flushdns is the documented Windows cache-management operation;
    // use it only for explicit repairs, with no shell and a bounded lifetime.
    let status = run_system_command("ipconfig.exe", &["/flushdns"], Duration::from_secs(10))
        .map_err(|error| format!("network_dns_flush:{error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("network_dns_flush:exit_code:{:?}", status.code()))
    }
}

fn run_system_command(name: &str, args: &[&str], timeout: Duration) -> Result<ExitStatus, String> {
    let mut system_directory = [0u16; 32768];
    let length = unsafe {
        windows::Win32::System::SystemInformation::GetSystemDirectoryW(Some(&mut system_directory))
    } as usize;
    if length == 0 || length >= system_directory.len() {
        return Err("system_directory_unavailable".into());
    }
    let executable =
        std::path::PathBuf::from(String::from_utf16_lossy(&system_directory[..length])).join(name);
    let mut child = Command::new(executable)
        .hide_window()
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("start_failed:{error}"))?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if started.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(50))
            }
            result => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(match result {
                    Err(error) => format!("wait_failed:{error}"),
                    _ => "timeout".into(),
                });
            }
        }
    }
}

pub fn speed_test() -> Result<SpeedTestResult, String> {
    const URL: &str = "https://speed.cloudflare.com/__down?bytes=1048576";
    let vpn_active = detect_vpn().map(|report| report.active).ok();
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
    command.hide_window().args([
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

fn detect_vpn() -> Result<VpnReport, String> {
    let script = r#"
$pattern = 'VPN|TAP-Windows|TUN|WireGuard|Wintun|OpenVPN|ZeroTier|Tailscale|NordLynx|Cisco|AnyConnect|PANGP|GlobalProtect|Juniper|Pulse Secure|Fortinet|Cloudflare|WARP|Clash|Mihomo|sing-box|V2Ray|Xray|Outline|Hiddify|Neko|SSTP|IKEv2|L2TP|PPTP'
$platformPattern = 'Hyper-V|VMware|VirtualBox|WSL|Docker|vEthernet|Loopback|Npcap'
$defaultIndexes = @(Get-NetRoute -ErrorAction Stop | Where-Object {
  $_.DestinationPrefix -in @('0.0.0.0/0', '::/0')
} | Select-Object -ExpandProperty InterfaceIndex -Unique)
$adapters = @(Get-NetAdapter -IncludeHidden -ErrorAction Stop | Where-Object {
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
    let value = powershell::run_json(script)?;
    if !value
        .get("active")
        .is_some_and(serde_json::Value::is_boolean)
        || !value.get("proxy").is_some_and(serde_json::Value::is_object)
    {
        return Err("network_vpn:invalid_response".into());
    }
    Ok(parse_vpn_json(value))
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

fn ping_once(host: &str) -> Option<bool> {
    icmp::ping(host)
}

fn check_gateways(adapters: &[NetworkAdapter]) -> Vec<GatewayCheck> {
    let gateways = adapters
        .iter()
        .filter(|adapter| adapter.oper_status == "up")
        .flat_map(|adapter| adapter.gateways.iter().cloned())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut results = Vec::new();
    // Preserve gateway tests for all candidate routes, with at most two probes
    // active at once. Each child has its own timeout and no unbounded output pipe.
    for batch in gateways.chunks(2) {
        std::thread::scope(|scope| {
            let tasks = batch
                .iter()
                .map(|gateway| (gateway, scope.spawn(move || ping_once(gateway))))
                .collect::<Vec<_>>();
            for (gateway, task) in tasks {
                results.push(GatewayCheck {
                    gateway: gateway.clone(),
                    reachable: task.join().unwrap_or(None),
                });
            }
        });
    }
    results
}

#[cfg(test)]
mod tests {
    use super::{curl_error_id, detect_windows_connectivity, parse_vpn_json};

    #[test]
    fn windows_connectivity_evidence_is_consistent() {
        let status =
            detect_windows_connectivity().expect("Network List Manager should be available");
        assert!(!status.internet || status.connected);
    }

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
