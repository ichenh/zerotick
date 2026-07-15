//! 音频诊断 — 服务、输入/输出设备、默认设备、独占模式

use crate::services::{self, ServicesReport, AUDIO};
use crate::utils::powershell;
use serde::{Deserialize, Serialize};
use windows::core::{GUID, PCWSTR};
use windows::Win32::Foundation::{
    E_ACCESSDENIED, PROPERTYKEY, RPC_E_CHANGED_MODE, STG_E_ACCESSDENIED,
};
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    eCapture, eConsole, eRender, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
    COINIT_MULTITHREADED, STGM_READWRITE,
};

const AUDIO_ENDPOINT_PROPERTY_SET: GUID = GUID::from_u128(0xb3f8fa53_0004_438e_9003_51a46e139bfc);
const PKEY_AUDIO_ENDPOINT_ALLOW_EXCLUSIVE: PROPERTYKEY = PROPERTYKEY {
    fmtid: AUDIO_ENDPOINT_PROPERTY_SET,
    pid: 3,
};
const PKEY_AUDIO_ENDPOINT_EXCLUSIVE_PRIORITY: PROPERTYKEY = PROPERTYKEY {
    fmtid: AUDIO_ENDPOINT_PROPERTY_SET,
    pid: 4,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: String,
    pub name: String,
    /// playback | capture
    pub kind: String,
    /// speakers | headphones | digital | microphone | headset | other
    pub category: String,
    pub is_default: bool,
    /// shared | exclusive | exclusive_priority（仅输出）
    pub mode: String,
    /// 端点主音量（0-100）；驱动不支持时为 None。
    #[serde(default)]
    pub volume_percent: Option<u8>,
    /// 端点静音状态；驱动不支持时为 None。
    #[serde(default)]
    pub is_muted: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct AudioDiagReport {
    pub services: ServicesReport,
    pub playback: Vec<AudioDevice>,
    pub capture: Vec<AudioDevice>,
}

pub fn diagnose() -> Result<AudioDiagReport, String> {
    let (services, devices) = std::thread::scope(|scope| {
        let services_task = scope.spawn(|| services::diagnose_group(AUDIO));
        let devices_task = scope.spawn(list_devices);
        let services = services_task
            .join()
            .map_err(|_| "音频服务扫描异常终止".to_string())??;
        let devices = devices_task
            .join()
            .map_err(|_| "音频设备扫描异常终止".to_string())??;
        Ok::<_, String>((services, devices))
    })?;
    let mut playback = Vec::new();
    let mut capture = Vec::new();
    for dev in devices {
        if dev.kind == "capture" {
            capture.push(dev);
        } else {
            playback.push(dev);
        }
    }
    Ok(AudioDiagReport {
        services,
        playback,
        capture,
    })
}

pub fn list_devices() -> Result<Vec<AudioDevice>, String> {
    let script = r#"
function Get-DeviceMode($item) {
  $exclusive = $item.'{b3f8fa53-0004-438e-9003-51a46e139bfc},3'
  $priority = $item.'{b3f8fa53-0004-438e-9003-51a46e139bfc},4'
  # Windows defaults both options to enabled when no per-endpoint override exists.
  $exclusiveOn = if ($null -eq $exclusive) { $true } else { $exclusive -eq 1 -or $exclusive -eq 2 }
  $priorityOn = if ($null -eq $priority) { $true } else { $priority -eq 1 -or $priority -eq 2 }
  if ($exclusiveOn -and $priorityOn) { 'exclusive_priority' }
  elseif ($exclusiveOn) { 'exclusive' }
  else { 'shared' }
}

function Normalize-NameKey($name) {
  $s = ($name -as [string]).Trim().ToLowerInvariant()
  if ($s -match '^(.*)\s+\(\d+\)$') { return $Matches[1].Trim() }
  $s
}

function Get-Category($kind, $formFactor, $name) {
  $ff = 0
  if ($null -ne $formFactor) {
    [void][int]::TryParse(([string]$formFactor), [ref]$ff)
  }
  $n = ($name -as [string]).ToLowerInvariant()
  if ($kind -eq 'capture') {
    if ($ff -in 5,6 -or $n -match 'headset|耳麦') { return 'headset' }
    if ($n -match 'line|线路') { return 'line' }
    if ($ff -eq 4 -or $n -match 'mic|麦克风|microphone|阵列|array') { return 'microphone' }
    return 'microphone'
  }
  if ($ff -eq 1 -or $n -match 'speaker|扬声器|喇叭') { return 'speakers' }
  if ($ff -in 3,5,6 -or $n -match 'headphone|耳机') { return 'headphones' }
  if ($ff -in 8,9,10,13,14 -or $n -match 'hdmi|spdif|s/pdif|digital|数字|display|optical|光纤|dp') { return 'digital' }
  if ($ff -eq 2 -or $n -match 'line|线路') { return 'line' }
  'other'
}

function Add-Endpoint($byKey, $entry, $kind, $name) {
  $key = "$kind::$(Normalize-NameKey $name)"
  if (-not $byKey.ContainsKey($key)) {
    $byKey[$key] = $entry
  } elseif ($entry.is_default -and -not $byKey[$key].is_default) {
    $byKey[$key] = $entry
  } elseif (-not $entry.name.Contains('(') -and $byKey[$key].name.Contains('(')) {
    $byKey[$key] = $entry
  }
}

function Enumerate-AudioEndpoints($subKey, $kind, $defaultId) {
  $root = "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\MMDevices\Audio\$subKey"
  $byKey = @{}
  if (-not (Test-Path $root)) { return @() }

  Get-ChildItem $root -ErrorAction SilentlyContinue | ForEach-Object {
    $id = $_.PSChildName
    $endpoint = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
    $deviceState = 0
    if (-not $endpoint -or -not [int]::TryParse(([string]$endpoint.DeviceState), [ref]$deviceState)) { return }
    # DEVICE_STATE_ACTIVE = 1；不展示已禁用、未连接或已拔出的历史端点。
    if ($deviceState -ne 1) { return }

    $props = Join-Path $_.PSPath 'Properties'
    if (-not (Test-Path $props)) { return }
    $item = Get-ItemProperty $props -ErrorAction SilentlyContinue
    if (-not $item) { return }

    $description = ($item.'{a45c254e-df1c-4efd-8020-67d146a850e0},2' -as [string]).Trim()
    $containerName = ($item.'{b3f8fa53-0004-438e-9003-51a46e139bfc},6' -as [string]).Trim()
    if ($description -and $containerName -and $description -ne $containerName) {
      $name = "$description ($containerName)"
    } elseif ($description) {
      $name = $description
    } else {
      $name = $containerName
    }
    if (-not $name) { return }

    # PKEY_AudioEndpoint_FormFactor；旧代码误用了字符串属性并强制转换为 Int32。
    $formFactor = $item.'{1da5d803-d492-4edd-8c23-e0c0ffee7f0e},0'
    $category = Get-Category $kind $formFactor $name
    $endpointId = if ($kind -eq 'capture') { "{0.0.1.00000000}.$id" } else { "{0.0.0.00000000}.$id" }
    $entry = [PSCustomObject]@{
      id = $endpointId
      name = $name
      kind = $kind
      category = $category
      is_default = ($id -eq $defaultId -or $endpointId -eq $defaultId)
      mode = if ($kind -eq 'playback') { Get-DeviceMode $item } else { 'shared' }
    }
    Add-Endpoint $byKey $entry $kind $name
  }
  @($byKey.Values)
}

$mapper = 'HKCU:\Software\Microsoft\Multimedia\Sound Mapper'
$defaults = $null
if (Test-Path $mapper) { $defaults = Get-ItemProperty $mapper -ErrorAction SilentlyContinue }
$playbackDefault = if ($defaults) { $defaults.Playback } else { $null }
$recordDefault = if ($defaults) { $defaults.Record } else { $null }

$all = @()
$all += Enumerate-AudioEndpoints 'Render' 'playback' $playbackDefault
$all += Enumerate-AudioEndpoints 'Capture' 'capture' $recordDefault
$all
"#;
    let val = powershell::run_json(script)?;
    let mut devices = dedupe_devices(parse_devices(val)?);
    if let Ok(access) = AudioEndpointAccess::new() {
        for device in &mut devices {
            if let Ok((volume, muted)) = access.state(&device.id) {
                device.volume_percent = Some(volume);
                device.is_muted = Some(muted);
            }
            device.is_default = access.is_default(&device.kind, &device.id);
        }
    }
    Ok(devices)
}

struct AudioEndpointAccess {
    enumerator: IMMDeviceEnumerator,
    playback_default: Option<String>,
    capture_default: Option<String>,
    uninitialize_com: bool,
}

impl AudioEndpointAccess {
    fn new() -> Result<Self, String> {
        unsafe {
            let init = CoInitializeEx(None, COINIT_MULTITHREADED);
            let uninitialize_com = if init.is_ok() {
                true
            } else if init == RPC_E_CHANGED_MODE {
                false
            } else {
                return Err(format!("初始化 Windows 音频接口失败: {init:?}"));
            };
            let enumerator: IMMDeviceEnumerator =
                match CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) {
                    Ok(value) => value,
                    Err(error) => {
                        if uninitialize_com {
                            CoUninitialize();
                        }
                        return Err(format!("创建 Windows 音频设备枚举器失败: {error}"));
                    }
                };
            let playback_default = enumerator
                .GetDefaultAudioEndpoint(eRender, eConsole)
                .ok()
                .and_then(endpoint_id);
            let capture_default = enumerator
                .GetDefaultAudioEndpoint(eCapture, eConsole)
                .ok()
                .and_then(endpoint_id);
            Ok(Self {
                enumerator,
                playback_default,
                capture_default,
                uninitialize_com,
            })
        }
    }

    fn is_default(&self, kind: &str, device_id: &str) -> bool {
        let default = if kind == "capture" {
            self.capture_default.as_deref()
        } else {
            self.playback_default.as_deref()
        };
        default.is_some_and(|id| id.eq_ignore_ascii_case(device_id))
    }

    fn volume(&self, device_id: &str) -> Result<IAudioEndpointVolume, String> {
        let wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            let device = self
                .enumerator
                .GetDevice(PCWSTR(wide.as_ptr()))
                .map_err(|error| format!("未找到音频端点: {error}"))?;
            device
                .Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
                .map_err(|error| format!("无法读取音频端点控制: {error}"))
        }
    }

    fn state(&self, device_id: &str) -> Result<(u8, bool), String> {
        let endpoint = self.volume(device_id)?;
        unsafe {
            let scalar = endpoint
                .GetMasterVolumeLevelScalar()
                .map_err(|error| format!("读取端点音量失败: {error}"))?;
            let muted = endpoint
                .GetMute()
                .map_err(|error| format!("读取端点静音状态失败: {error}"))?
                .as_bool();
            Ok(((scalar.clamp(0.0, 1.0) * 100.0).round() as u8, muted))
        }
    }
}

fn endpoint_id(device: IMMDevice) -> Option<String> {
    unsafe {
        let value = device.GetId().ok()?;
        let text = value.to_string().ok();
        CoTaskMemFree(Some(value.0.cast()));
        text
    }
}

impl Drop for AudioEndpointAccess {
    fn drop(&mut self) {
        if self.uninitialize_com {
            unsafe { CoUninitialize() };
        }
    }
}

pub fn set_endpoint_volume(device_id: &str, percent: u8) -> Result<(), String> {
    let access = AudioEndpointAccess::new()?;
    let endpoint = access.volume(device_id)?;
    unsafe {
        endpoint
            .SetMasterVolumeLevelScalar(f32::from(percent.min(100)) / 100.0, std::ptr::null())
            .map_err(|error| format!("设置端点音量失败: {error}"))
    }
}

pub fn set_endpoint_mute(device_id: &str, muted: bool) -> Result<(), String> {
    let access = AudioEndpointAccess::new()?;
    let endpoint = access.volume(device_id)?;
    unsafe {
        endpoint
            .SetMute(muted, std::ptr::null())
            .map_err(|error| format!("设置端点静音失败: {error}"))
    }
}

pub fn set_default_device(device_id: &str, kind: &str) -> Result<(), String> {
    let id = device_id.replace('\'', "''");
    let (prop, roles) = match kind {
        "capture" => ("Record", "0,1"),
        _ => ("Playback", "0,1"),
    };
    let script = format!(
        r#"
$id = '{id}'
$prop = '{prop}'
$roles = @({roles})

function Ensure-SoundMapper {{
  $parent = 'HKCU:\Software\Microsoft\Multimedia'
  $mapper = Join-Path $parent 'Sound Mapper'
  if (-not (Test-Path -LiteralPath $parent)) {{
    New-Item -Path $parent -Force | Out-Null
  }}
  if (-not (Test-Path -LiteralPath $mapper)) {{
    New-Item -Path $mapper -Force | Out-Null
  }}
  $mapper
}}

$mapper = Ensure-SoundMapper
Set-ItemProperty -LiteralPath $mapper -Name $prop -Value $id

if (-not ('AudioSwitcher.PolicyConfigClient' -as [type])) {{
  Add-Type -Language CSharp -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
namespace AudioSwitcher {{
  [ComImport, Guid(""870af99c-171d-4f9e-af0d-e63df40c2bc9"")]
  public class PolicyConfigClient {{ }}
  [ComImport, Guid(""F8679F50-850A-41CF-9C72-430F290290C8""), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
  public interface IPolicyConfig {{
    [PreserveSig] int Reserved1();
    [PreserveSig] int Reserved2();
    [PreserveSig] int Reserved3();
    [PreserveSig] int Reserved4();
    [PreserveSig] int Reserved5();
    [PreserveSig] int Reserved6();
    [PreserveSig] int Reserved7();
    [PreserveSig] int Reserved8();
    [PreserveSig] int Reserved9();
    [PreserveSig] int Reserved10();
    [PreserveSig] int SetDefaultEndpoint([MarshalAs(UnmanagedType.LPWStr)] string pszDeviceId, [MarshalAs(UnmanagedType.U4)] int role);
  }}
  public static class PolicyConfig {{
    public static int SetDefaultEndpoint(string deviceId, int role) {{
      var policy = (IPolicyConfig)new PolicyConfigClient();
      return policy.SetDefaultEndpoint(deviceId, role);
    }}
  }}
}}
"@
}}

try {{
  foreach ($role in $roles) {{
    $hr = [AudioSwitcher.PolicyConfig]::SetDefaultEndpoint($id, $role)
    if ($hr -ne 0) {{ throw ('0x{{0:X8}}' -f ([uint32]$hr)) }}
  }}
}} catch {{
  throw "设置默认音频设备失败: $($_.Exception.Message)"
}}
"#
    );
    powershell::run_void(&script)
}

pub fn set_device_mode(device_id: &str, kind: &str, mode: &str) -> Result<(), String> {
    if kind != "playback" {
        return Err("仅输出设备支持音频模式切换".into());
    }
    if !crate::utils::elevated::is_elevated() {
        return Err("audio_mode:admin_required".into());
    }
    let (exclusive, priority) = mode_property_values(mode)?;
    let access = AudioEndpointAccess::new()
        .map_err(|error| format!("audio_mode:interface_unavailable:{error}"))?;
    let wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
    let device = unsafe { access.enumerator.GetDevice(PCWSTR(wide.as_ptr())) }
        .map_err(|error| format!("audio_mode:endpoint_not_found:{error}"))?;

    unsafe {
        let store = device
            .OpenPropertyStore(STGM_READWRITE)
            .map_err(audio_mode_store_error)?;
        let exclusive_value = PROPVARIANT::from(exclusive);
        let priority_value = PROPVARIANT::from(priority);

        store
            .SetValue(&PKEY_AUDIO_ENDPOINT_ALLOW_EXCLUSIVE, &exclusive_value)
            .map_err(audio_mode_write_error)?;
        store
            .SetValue(&PKEY_AUDIO_ENDPOINT_EXCLUSIVE_PRIORITY, &priority_value)
            .map_err(audio_mode_write_error)?;
        store.Commit().map_err(audio_mode_write_error)?;

        let actual_exclusive = store
            .GetValue(&PKEY_AUDIO_ENDPOINT_ALLOW_EXCLUSIVE)
            .ok()
            .and_then(|value| u32::try_from(&value).ok());
        let actual_priority = store
            .GetValue(&PKEY_AUDIO_ENDPOINT_EXCLUSIVE_PRIORITY)
            .ok()
            .and_then(|value| u32::try_from(&value).ok());
        if actual_exclusive != Some(exclusive) || actual_priority != Some(priority) {
            return Err(format!(
                "audio_mode:verify_failed:expected={exclusive},{priority};actual={actual_exclusive:?},{actual_priority:?}"
            ));
        }
    }
    Ok(())
}

fn mode_property_values(mode: &str) -> Result<(u32, u32), String> {
    match mode {
        "shared" => Ok((0, 0)),
        "exclusive" => Ok((1, 0)),
        "exclusive_priority" => Ok((1, 1)),
        _ => Err(format!("不支持的音频模式: {mode}")),
    }
}

fn audio_mode_store_error(error: windows::core::Error) -> String {
    if error.code() == E_ACCESSDENIED || error.code() == STG_E_ACCESSDENIED {
        format!("audio_mode:access_denied:{error}")
    } else {
        format!("audio_mode:property_store_unavailable:{error}")
    }
}

fn audio_mode_write_error(error: windows::core::Error) -> String {
    if error.code() == E_ACCESSDENIED || error.code() == STG_E_ACCESSDENIED {
        format!("audio_mode:access_denied:{error}")
    } else {
        format!("audio_mode:write_failed:{error}")
    }
}

pub fn repair() -> (Vec<String>, Vec<String>) {
    services::repair_group(AUDIO)
}

fn dedupe_devices(devices: Vec<AudioDevice>) -> Vec<AudioDevice> {
    use std::collections::HashMap;

    fn norm_name(name: &str) -> String {
        let s = name.trim().to_lowercase();
        if let Some(idx) = s.rfind(" (") {
            if s.ends_with(')') {
                let inner = &s[idx + 2..s.len() - 1];
                if !inner.is_empty() && inner.chars().all(|c| c.is_ascii_digit()) {
                    return s[..idx].to_string();
                }
            }
        }
        s
    }

    fn has_numeric_suffix(name: &str) -> bool {
        let s = name.trim();
        s.rfind(" (").is_some_and(|idx| {
            s.ends_with(')') && s[idx + 2..s.len() - 1].chars().all(|c| c.is_ascii_digit())
        })
    }

    fn prefer(a: &AudioDevice, b: &AudioDevice) -> bool {
        if a.is_default != b.is_default {
            return a.is_default;
        }
        let a_plain = !has_numeric_suffix(&a.name);
        let b_plain = !has_numeric_suffix(&b.name);
        if a_plain != b_plain {
            return a_plain;
        }
        a.id < b.id
    }

    let mut by_key: HashMap<String, AudioDevice> = HashMap::new();
    for dev in devices {
        let key = format!("{}:{}:{}", dev.kind, dev.category, norm_name(&dev.name));
        match by_key.get(&key) {
            None => {
                by_key.insert(key, dev);
            }
            Some(existing) if prefer(&dev, existing) => {
                by_key.insert(key, dev);
            }
            Some(_) => {}
        }
    }
    let mut out: Vec<_> = by_key.into_values().collect();
    out.sort_by(|a, b| {
        a.kind
            .cmp(&b.kind)
            .then_with(|| category_order(&a.category).cmp(&category_order(&b.category)))
            .then_with(|| b.is_default.cmp(&a.is_default))
            .then_with(|| a.name.cmp(&b.name))
    });
    out
}

fn category_order(category: &str) -> u8 {
    match category {
        "speakers" => 0,
        "headphones" => 1,
        "digital" => 2,
        "microphone" => 0,
        "headset" => 1,
        "other" => 9,
        _ => 8,
    }
}

fn parse_devices(val: serde_json::Value) -> Result<Vec<AudioDevice>, String> {
    let arr = match val {
        serde_json::Value::Array(a) => a,
        serde_json::Value::Object(_) => vec![val],
        serde_json::Value::Null => vec![],
        _ => return Err("音频设备列表格式异常".into()),
    };
    let mut out = Vec::new();
    let mut seen_ids = std::collections::HashSet::new();
    for item in arr {
        let id = item
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let name = item
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if id.is_empty() || name.is_empty() || !seen_ids.insert(id.clone()) {
            continue;
        }
        let kind = item
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or("playback")
            .to_string();
        let category = item
            .get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("other")
            .to_string();
        let mode = item
            .get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("shared")
            .to_string();
        out.push(AudioDevice {
            id,
            name,
            kind,
            category,
            is_default: item
                .get("is_default")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            mode,
            volume_percent: item
                .get("volume_percent")
                .and_then(|v| v.as_u64())
                .map(|v| v.min(100) as u8),
            is_muted: item.get("is_muted").and_then(|v| v.as_bool()),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{
        mode_property_values, AUDIO_ENDPOINT_PROPERTY_SET, PKEY_AUDIO_ENDPOINT_ALLOW_EXCLUSIVE,
        PKEY_AUDIO_ENDPOINT_EXCLUSIVE_PRIORITY,
    };

    #[test]
    fn exclusive_modes_map_to_windows_endpoint_values() {
        assert_eq!(mode_property_values("shared").unwrap(), (0, 0));
        assert_eq!(mode_property_values("exclusive").unwrap(), (1, 0));
        assert_eq!(mode_property_values("exclusive_priority").unwrap(), (1, 1));
        assert!(mode_property_values("invalid").is_err());
    }

    #[test]
    fn exclusive_mode_uses_correct_endpoint_property_keys() {
        assert_eq!(
            PKEY_AUDIO_ENDPOINT_ALLOW_EXCLUSIVE.fmtid,
            AUDIO_ENDPOINT_PROPERTY_SET
        );
        assert_eq!(PKEY_AUDIO_ENDPOINT_ALLOW_EXCLUSIVE.pid, 3);
        assert_eq!(
            PKEY_AUDIO_ENDPOINT_EXCLUSIVE_PRIORITY.fmtid,
            AUDIO_ENDPOINT_PROPERTY_SET
        );
        assert_eq!(PKEY_AUDIO_ENDPOINT_EXCLUSIVE_PRIORITY.pid, 4);
    }
}
