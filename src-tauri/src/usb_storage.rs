//! 可移动存储（U 盘）— 列表、占用解除、安全弹出

use crate::services::{self, ServicesReport, USB};
use crate::utils::{powershell, wmi_runner};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use windows::core::PCWSTR;
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    CM_Get_DevNode_Registry_PropertyW, CM_Get_Parent, CM_Locate_DevNodeW, CM_Request_Device_EjectW,
    CM_DEVCAP_REMOVABLE, CM_DRP_CAPABILITIES, CM_LOCATE_DEVNODE_NORMAL, CR_SUCCESS, PNP_VETO_TYPE,
};
use windows::Win32::Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FlushFileBuffers, GetLogicalDrives, QueryDosDeviceW, FILE_ATTRIBUTE_NORMAL,
    FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
use windows::Win32::System::Ioctl::{
    FSCTL_DISMOUNT_VOLUME, FSCTL_LOCK_VOLUME, FSCTL_UNLOCK_VOLUME,
};
use windows::Win32::System::IO::DeviceIoControl;
use wmi::WMIConnection;

#[derive(Debug, Clone, Serialize)]
pub struct UsbDrive {
    pub letter: String,
    pub label: String,
    pub size_gb: f64,
    pub free_gb: f64,
    pub filesystem: String,
    /// ready | locked | unavailable
    pub access_state: String,
    /// Physical disk number resolved from the DOS device path when Windows exposes it.
    pub disk_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsbStorageSlot {
    pub name: String,
    pub instance_id: String,
    pub access_state: String,
    pub volume_letters: Vec<String>,
    pub disk_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsbStorageDevice {
    pub name: String,
    pub instance_id: String,
    pub bus_type: String,
    pub status: String,
    pub problem_code: u32,
    /// mounted | locked | offline | unpartitioned | no_drive_letter | no_media | unknown
    pub access_state: String,
    pub volume_letters: Vec<String>,
    /// Windows device container for one physical enclosure. Card readers remain per disk.
    pub physical_id: String,
    /// external_disk | card_reader | unknown
    pub device_kind: String,
    pub disk_numbers: Vec<u32>,
    /// Physical members exposed by the enclosure. Primarily used as card-reader slots.
    pub slots: Vec<UsbStorageSlot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LockingProcess {
    pub pid: u32,
    pub name: String,
    pub path: Option<String>,
    /// This list is a path/command-line correlation, not authoritative handle evidence.
    pub confidence: String,
    pub can_close: bool,
}

#[derive(Debug, Serialize)]
pub struct UsbDiagReport {
    pub services: ServicesReport,
    pub drives: Vec<UsbDrive>,
    pub devices: Vec<UsbStorageDevice>,
}

#[derive(Debug, Deserialize)]
struct LogicalDisk {
    #[serde(rename = "DeviceID")]
    device_id: Option<String>,
    #[serde(rename = "VolumeName")]
    volume_name: Option<String>,
    #[serde(rename = "Size")]
    size: Option<u64>,
    #[serde(rename = "FreeSpace")]
    free_space: Option<u64>,
    #[serde(rename = "FileSystem")]
    file_system: Option<String>,
}

pub fn diagnose() -> Result<UsbDiagReport, String> {
    let (services, drives, mut devices) = std::thread::scope(|scope| {
        let services_task = scope.spawn(|| services::diagnose_group(USB));
        let drives_task = scope.spawn(list_drives);
        let devices_task = scope.spawn(list_storage_devices);
        let services = services_task
            .join()
            .map_err(|_| "USB 服务扫描异常终止".to_string())??;
        let drives = drives_task
            .join()
            .map_err(|_| "USB 盘符扫描异常终止".to_string())??;
        let devices = devices_task
            .join()
            .map_err(|_| "USB 硬件扫描异常终止".to_string())?
            .unwrap_or_else(|error| {
                crate::utils::logging::warn(format!("USB 存储硬件枚举失败: {error}"));
                Vec::new()
            });
        Ok::<_, String>((services, drives, devices))
    })?;
    attach_drive_letters_to_slots(&drives, &mut devices);
    Ok(UsbDiagReport {
        services,
        drives,
        devices,
    })
}

#[derive(Debug, Serialize)]
pub struct UsbEjectResult {
    /// ejected | busy | permission_required | vetoed | failed
    pub status: String,
    /// flush | lock | dismount | pnp
    pub stage: String,
    pub blockers: Vec<LockingProcess>,
    pub veto_type: Option<String>,
    pub veto_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsbCloseProcessResult {
    /// requested | protected | no_window | not_found
    pub status: String,
}

#[derive(Debug, Deserialize)]
struct EjectTarget {
    pnp_device_id: String,
    volume_letters: Vec<String>,
}

struct VolumeHandle(HANDLE);

impl Drop for VolumeHandle {
    fn drop(&mut self) {
        // SAFETY: this type is only constructed from a successful CreateFileW call.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn list_storage_devices() -> Result<Vec<UsbStorageDevice>, String> {
    let script = r#"
$devices = @{}
function Add-StorageDevice($name, $id, $bus, $status, $problem, $accessState, $letters, $physicalId, $deviceKind, $diskNumber) {
  if (-not $id) { return }
  $key = [string]$id
  $devices[$key] = [pscustomobject]@{
    name = if ($name) { [string]$name } else { 'USB Storage Device' }
    instance_id = $key
    bus_type = if ($bus) { [string]$bus } else { 'USB' }
    status = if ($status) { [string]$status } else { 'Unknown' }
    problem_code = [uint32]$problem
    access_state = if ($accessState) { [string]$accessState } else { 'unknown' }
    volume_letters = @($letters)
    physical_id = if ($physicalId) { [string]$physicalId } else { $key }
    device_kind = if ($deviceKind) { [string]$deviceKind } else { 'unknown' }
    disk_numbers = if ($null -ne $diskNumber) { @([uint32]$diskNumber) } else { @() }
  }
}

# Storage Management 能准确识别 UASP/SCSI 外壳后的实际 USB 总线。
$usbDisks = @(Get-Disk -ErrorAction SilentlyContinue | Where-Object {
  [string]$_.BusType -eq 'USB'
})
$usbDisks | ForEach-Object {
  $id = if ($_.Path) { [string]$_.Path } elseif ($_.UniqueId) { [string]$_.UniqueId } else { "disk:$($_.Number)" }
  $diskDrive = Get-CimInstance Win32_DiskDrive -Filter "Index=$($_.Number)" -ErrorAction SilentlyContinue
  $pnpId = if ($diskDrive -and $diskDrive.PNPDeviceID) { [string]$diskDrive.PNPDeviceID } else { $null }
  $container = if ($pnpId) { Get-PnpDeviceProperty -InstanceId $pnpId -KeyName 'DEVPKEY_Device_ContainerId' -ErrorAction SilentlyContinue } else { $null }
  $identityText = "$($_.FriendlyName) $($diskDrive.Model) $($diskDrive.Caption)"
  $isCardReader = $identityText -match '(?i)card\s*reader|multi[ -]?card|sd\s*reader|mmc|memory\s*stick|compact\s*flash|smart\s*media|xd[ -]?picture|ms[/ -]?ms-pro|读卡器'
  $deviceKind = if ($isCardReader) { 'card_reader' } else { 'external_disk' }
  $containerId = if ($container -and $container.Data) { [string]$container.Data } else { $id }
  $physicalId = $containerId
  $partitions = @(Get-Partition -DiskNumber $_.Number -ErrorAction SilentlyContinue)
  $letters = @($partitions | Where-Object { $_.DriveLetter } | ForEach-Object { "$($_.DriveLetter):" })
  $volumes = @($letters | ForEach-Object { Get-Volume -DriveLetter $_.TrimEnd(':') -ErrorAction SilentlyContinue })
  $isUnavailable = $letters.Count -gt 0 -and @($volumes | Where-Object { -not $_.FileSystem }).Count -gt 0
  $operational = [string]$_.OperationalStatus
  # Empty reader slots are commonly also marked IsOffline. No Media/Size 0 is
  # stronger evidence and must be evaluated before the generic offline state.
  $accessState = if ([uint64]$_.Size -eq 0 -or $operational -match 'No Media') {
    'no_media'
  } elseif ($_.IsOffline -or $operational -match 'Offline') {
    'offline'
  } elseif ($partitions.Count -eq 0) {
    'unpartitioned'
  } elseif ($letters.Count -eq 0) {
    'no_drive_letter'
  } elseif ($isUnavailable) {
    'unavailable'
  } else {
    'mounted'
  }
  Add-StorageDevice $_.FriendlyName $id 'USB' $operational 0 $accessState $letters $physicalId $deviceKind $_.Number
}

# USBSTOR 是传统大容量存储；UASPStor 常表现为 SCSI 路径，不能按实例前缀过滤。
if ($devices.Count -eq 0) {
  Get-CimInstance Win32_PnPEntity -ErrorAction SilentlyContinue | Where-Object {
    $_.Service -in @('USBSTOR', 'UASPStor')
  } | ForEach-Object {
    $bus = if ($_.Service -eq 'UASPStor') { 'USB (UASP)' } else { 'USB' }
    Add-StorageDevice $_.Name $_.PNPDeviceID $bus $_.Status $_.ConfigManagerErrorCode 'unknown' @() $_.PNPDeviceID 'unknown' $null
  }

  # 保留 PnP 兜底，兼容未暴露 Storage Management 对象的普通 U 盘。
  Get-PnpDevice -PresentOnly -ErrorAction SilentlyContinue | Where-Object {
    $_.InstanceId -like 'USBSTOR\*'
  } | ForEach-Object {
    Add-StorageDevice $_.FriendlyName $_.InstanceId 'USB' $_.Status $_.Problem 'unknown' @() $_.InstanceId 'unknown' $null
  }
}

@($devices.Values | Sort-Object name)
"#;
    let val = powershell::run_json(script)?;
    let arr = match val {
        serde_json::Value::Array(items) => items,
        serde_json::Value::Object(_) => vec![val],
        serde_json::Value::Null => vec![],
        _ => return Err("USB 存储设备列表格式异常".into()),
    };
    let devices = arr
        .into_iter()
        .filter_map(|item| {
            Some(UsbStorageDevice {
                name: item.get("name")?.as_str()?.to_string(),
                instance_id: item.get("instance_id")?.as_str()?.to_string(),
                bus_type: item
                    .get("bus_type")
                    .and_then(|value| value.as_str())
                    .unwrap_or("USB")
                    .to_string(),
                status: item
                    .get("status")
                    .and_then(|value| value.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                problem_code: item
                    .get("problem_code")
                    .and_then(|value| value.as_u64())
                    .unwrap_or(0) as u32,
                access_state: item
                    .get("access_state")
                    .and_then(|value| value.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                volume_letters: item
                    .get("volume_letters")
                    .and_then(|value| value.as_array())
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(|value| value.as_str().map(str::to_string))
                            .collect()
                    })
                    .unwrap_or_default(),
                physical_id: item
                    .get("physical_id")
                    .and_then(|value| value.as_str())
                    .unwrap_or_else(|| {
                        item.get("instance_id")
                            .and_then(|value| value.as_str())
                            .unwrap_or("unknown")
                    })
                    .to_string(),
                device_kind: item
                    .get("device_kind")
                    .and_then(|value| value.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                disk_numbers: item
                    .get("disk_numbers")
                    .and_then(|value| value.as_array())
                    .map(|values| {
                        values
                            .iter()
                            .filter_map(|value| value.as_u64().map(|number| number as u32))
                            .collect()
                    })
                    .unwrap_or_default(),
                slots: Vec::new(),
            })
        })
        .map(|mut device| {
            device.slots.push(UsbStorageSlot {
                name: device.name.clone(),
                instance_id: device.instance_id.clone(),
                access_state: device.access_state.clone(),
                volume_letters: device.volume_letters.clone(),
                disk_number: device.disk_numbers.first().copied(),
            });
            device
        })
        .collect::<Vec<_>>();
    Ok(group_physical_devices(devices))
}

fn group_physical_devices(devices: Vec<UsbStorageDevice>) -> Vec<UsbStorageDevice> {
    use std::collections::HashMap;

    fn state_rank(state: &str) -> u8 {
        match state {
            "locked" => 6,
            "unavailable" => 5,
            // One enclosure can expose several LUNs (for example, a usable data disk
            // plus an offline virtual CD). A mounted member means the physical device
            // is usable and must not be hidden by an inactive companion LUN.
            "mounted" => 4,
            "offline" => 3,
            "no_drive_letter" | "unpartitioned" => 2,
            _ => 1,
        }
    }

    let mut groups: HashMap<String, UsbStorageDevice> = HashMap::new();
    for mut device in devices {
        let key = device.physical_id.clone();
        if let Some(existing) = groups.get_mut(&key) {
            if state_rank(&device.access_state) > state_rank(&existing.access_state) {
                existing.access_state = device.access_state;
            }
            existing.problem_code = existing.problem_code.max(device.problem_code);
            existing.volume_letters.append(&mut device.volume_letters);
            existing.disk_numbers.append(&mut device.disk_numbers);
            existing.slots.append(&mut device.slots);
            existing.volume_letters.sort();
            existing.volume_letters.dedup();
            existing.disk_numbers.sort_unstable();
            existing.disk_numbers.dedup();
            existing.slots.sort_by_key(|slot| slot.disk_number);
        } else {
            groups.insert(key, device);
        }
    }
    let mut grouped: Vec<_> = groups.into_values().collect();
    for device in &mut grouped {
        let generic_multi_slot = device.slots.len() > 1
            && device.slots.iter().all(|slot| {
                let name = slot.name.to_ascii_lowercase();
                name.contains("massstorageclass") || name.contains("card reader")
            });
        let has_empty_slot = device.slots.len() > 1
            && device
                .slots
                .iter()
                .any(|slot| slot.access_state == "no_media");
        if generic_multi_slot || has_empty_slot {
            device.device_kind = "card_reader".into();
        }
    }
    grouped.sort_by(|a, b| a.name.cmp(&b.name));
    grouped
}

fn attach_drive_letters_to_slots(drives: &[UsbDrive], devices: &mut [UsbStorageDevice]) {
    let mut matched = vec![false; drives.len()];
    for (drive_index, drive) in drives.iter().enumerate() {
        let Some(disk_number) = drive.disk_number else {
            continue;
        };
        for device in devices.iter_mut() {
            if let Some(slot) = device
                .slots
                .iter_mut()
                .find(|slot| slot.disk_number == Some(disk_number))
            {
                if !slot.volume_letters.contains(&drive.letter) {
                    slot.volume_letters.push(drive.letter.clone());
                    slot.volume_letters.sort();
                }
                if !device.volume_letters.contains(&drive.letter) {
                    device.volume_letters.push(drive.letter.clone());
                    device.volume_letters.sort();
                }
                matched[drive_index] = true;
                break;
            }
        }
    }

    // QueryDosDevice normally maps an empty reader letter (for example E:) to
    // HarddiskN. Some drivers expose only a generic removable letter. In the
    // unambiguous single-reader case, pair remaining zero-size unavailable
    // letters with its No Media slots so they do not become fake locked devices.
    let reader_indexes = devices
        .iter()
        .enumerate()
        .filter_map(|(index, device)| (device.device_kind == "card_reader").then_some(index))
        .collect::<Vec<_>>();
    if reader_indexes.len() != 1 {
        return;
    }
    let device = &mut devices[reader_indexes[0]];
    let mut empty_slots = device
        .slots
        .iter()
        .enumerate()
        .filter_map(|(index, slot)| {
            (slot.access_state == "no_media" && slot.volume_letters.is_empty())
                .then_some((index, slot.disk_number))
        })
        .collect::<Vec<_>>();
    empty_slots.sort_by_key(|(_, disk_number)| *disk_number);
    let mut empty_drives = drives
        .iter()
        .enumerate()
        .filter(|(index, drive)| {
            !matched[*index]
                && drive.access_state == "unavailable"
                && drive.size_gb == 0.0
                && drive.free_gb == 0.0
        })
        .map(|(_, drive)| drive)
        .collect::<Vec<_>>();
    empty_drives.sort_by(|a, b| a.letter.cmp(&b.letter));
    if empty_slots.len() != empty_drives.len() {
        return;
    }
    for ((slot_index, _), drive) in empty_slots.into_iter().zip(empty_drives) {
        device.slots[slot_index]
            .volume_letters
            .push(drive.letter.clone());
        if !device.volume_letters.contains(&drive.letter) {
            device.volume_letters.push(drive.letter.clone());
        }
    }
    device.volume_letters.sort();
}

fn query_drive_disk_number(letter: &str) -> Option<u32> {
    let letter = normalize_drive_letter(letter).ok()?;
    let dos_name = format!("{letter}:");
    let wide = dos_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut target = [0u16; 1024];
    let length = unsafe { QueryDosDeviceW(PCWSTR(wide.as_ptr()), Some(&mut target)) };
    if length == 0 {
        return None;
    }
    let path = String::from_utf16_lossy(&target[..length as usize]);
    parse_disk_number_from_dos_target(&path)
}

fn parse_disk_number_from_dos_target(path: &str) -> Option<u32> {
    let lower = path.to_ascii_lowercase();
    let marker = "harddisk";
    let start = lower.find(marker)? + marker.len();
    let digits = lower[start..]
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();
    (!digits.is_empty()).then(|| digits.parse().ok()).flatten()
}

pub fn list_drives() -> Result<Vec<UsbDrive>, String> {
    let mut drives = match list_drives_powershell() {
        Ok(drives) => drives,
        Err(error) => {
            crate::utils::logging::warn(format!("USB PowerShell 枚举失败: {error}"));
            Vec::new()
        }
    };
    if drives.is_empty() {
        drives = match wmi_runner::run(list_drives_inner) {
            Ok(drives) => drives,
            Err(error) => {
                crate::utils::logging::warn(format!("USB WMI 枚举失败: {error}"));
                Vec::new()
            }
        };
    }
    for drive in &mut drives {
        drive.disk_number = query_drive_disk_number(&drive.letter);
    }
    drives.sort_by(|a, b| a.letter.cmp(&b.letter));
    Ok(drives)
}

fn list_drives_inner(wmi: &WMIConnection) -> Result<Vec<UsbDrive>, wmi::WMIError> {
    let disks: Vec<LogicalDisk> = wmi
        .raw_query(
            "SELECT DeviceID, VolumeName, Size, FreeSpace, FileSystem FROM Win32_LogicalDisk WHERE DriveType=2",
        )
        .unwrap_or_default();
    Ok(disks.into_iter().filter_map(map_logical_disk).collect())
}

fn map_logical_disk(d: LogicalDisk) -> Option<UsbDrive> {
    let letter = d.device_id?;
    Some(UsbDrive {
        letter: letter.clone(),
        label: d
            .volume_name
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| letter.clone()),
        size_gb: d.size.unwrap_or(0) as f64 / 1_073_741_824.0,
        free_gb: d.free_space.unwrap_or(0) as f64 / 1_073_741_824.0,
        filesystem: d.file_system.unwrap_or_else(|| "—".into()),
        access_state: "ready".into(),
        disk_number: None,
    })
}

fn list_drives_powershell() -> Result<Vec<UsbDrive>, String> {
    let script = r#"
$byLetter = @{}
$lockedLetters = @{}
if (Get-Command Get-BitLockerVolume -ErrorAction SilentlyContinue) {
  Get-BitLockerVolume -ErrorAction SilentlyContinue | Where-Object { [string]$_.LockStatus -eq 'Locked' } | ForEach-Object {
    if ($_.MountPoint) { $lockedLetters[[string]$_.MountPoint.TrimEnd(':').ToUpperInvariant()] = $true }
  }
}
function Add-Vol($letter, $label, $size, $free, $fs, $access) {
  if (-not $letter) { return }
  $ch = [string]$letter
  if ($ch.Length -gt 1) { $ch = $ch.Substring(0, 1) }
  $key = $ch.ToUpperInvariant()
  if (-not $key) { return }
  $letterStr = "${key}:"
  $sz = [double]$size
  $fr = [double]$free
  if ($sz -lt 0) { $sz = 0 }
  if ($fr -lt 0) { $fr = 0 }
  $byLetter[$key] = [pscustomobject]@{
    letter = $letterStr
    label = if ($label) { [string]$label } else { $letterStr }
    size_gb = [math]::Round($sz / 1GB, 2)
    free_gb = [math]::Round($fr / 1GB, 2)
    filesystem = if ($fs) { [string]$fs } else { '—' }
    access_state = if ($lockedLetters.ContainsKey($key)) { 'locked' } elseif ($access) { [string]$access } elseif (-not $fs -and $sz -eq 0) { 'unavailable' } else { 'ready' }
  }
}

# DriveInfo 不依赖 Storage/CIM 管理权限。普通权限运行时也应先发现常规 U 盘，
# 后续 Storage/CIM 查询仅用于补充被 Windows 标记为 Fixed 的 USB 存储设备。
[System.IO.DriveInfo]::GetDrives() | Where-Object {
  $_.DriveType -eq [System.IO.DriveType]::Removable
} | ForEach-Object {
  if ($_.IsReady) {
    Add-Vol $_.Name $_.VolumeLabel $_.TotalSize $_.AvailableFreeSpace $_.DriveFormat 'ready'
  } elseif ($lockedLetters.ContainsKey($_.Name.Substring(0, 1).ToUpperInvariant())) {
    Add-Vol $_.Name $_.Name 0 0 '—' 'locked'
  }
}

Get-CimInstance Win32_LogicalDisk -Filter "DriveType=2" -ErrorAction SilentlyContinue | ForEach-Object {
  $ltr = if ($_.DeviceID) { $_.DeviceID[0] } else { $null }
  Add-Vol $ltr $_.VolumeName $_.Size $_.FreeSpace $_.FileSystem
}

Get-Volume -ErrorAction SilentlyContinue | Where-Object {
  $_.DriveLetter -and ($_.DriveType -eq 'Removable' -or $_.DriveType -eq 'Unknown')
} | ForEach-Object {
  Add-Vol $_.DriveLetter $_.FileSystemLabel $_.Size $_.SizeRemaining $_.FileSystem
}

Get-Disk -ErrorAction SilentlyContinue | Where-Object {
  $_.BusType -eq 'USB' -and -not $_.IsSystem -and -not $_.IsBoot
} | ForEach-Object {
  Get-Partition -DiskNumber $_.Number -ErrorAction SilentlyContinue | Where-Object { $_.DriveLetter } | ForEach-Object {
    $vol = Get-Volume -DriveLetter $_.DriveLetter -ErrorAction SilentlyContinue
    if ($vol) {
      Add-Vol $vol.DriveLetter $vol.FileSystemLabel $vol.Size $vol.SizeRemaining $vol.FileSystem
    }
  }
}

Get-CimInstance Win32_DiskDrive -Filter "InterfaceType='USB'" -ErrorAction SilentlyContinue | ForEach-Object {
  Get-CimAssociatedInstance -InputObject $_ -ResultClassName 'Win32_DiskPartition' -ErrorAction SilentlyContinue | ForEach-Object {
    Get-CimAssociatedInstance -InputObject $_ -ResultClassName 'Win32_LogicalDisk' -ErrorAction SilentlyContinue | ForEach-Object {
      $ltr = if ($_.DeviceID) { $_.DeviceID[0] } else { $null }
      Add-Vol $ltr $_.VolumeName $_.Size $_.FreeSpace $_.FileSystem
    }
  }
}

@($byLetter.Values | Sort-Object { $_.letter })
"#;
    let val = powershell::run_json(script)?;
    parse_drives_json(val)
}

fn parse_drives_json(val: serde_json::Value) -> Result<Vec<UsbDrive>, String> {
    let arr = match val {
        serde_json::Value::Array(a) => a,
        serde_json::Value::Object(_) => vec![val],
        serde_json::Value::Null => vec![],
        _ => return Err("U 盘列表格式异常".into()),
    };
    Ok(arr
        .into_iter()
        .filter_map(|item| {
            let letter = item.get("letter")?.as_str()?.to_string();
            Some(UsbDrive {
                label: item
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&letter)
                    .to_string(),
                size_gb: item.get("size_gb").and_then(|v| v.as_f64()).unwrap_or(0.0),
                free_gb: item.get("free_gb").and_then(|v| v.as_f64()).unwrap_or(0.0),
                filesystem: item
                    .get("filesystem")
                    .and_then(|v| v.as_str())
                    .unwrap_or("—")
                    .to_string(),
                access_state: item
                    .get("access_state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("ready")
                    .to_string(),
                disk_number: None,
                letter,
            })
        })
        .collect())
}

pub fn find_locking_processes(drive_letter: &str) -> Result<Vec<LockingProcess>, String> {
    let letter = normalize_drive_letter(drive_letter)?;
    let val = powershell::run_json(&locking_process_script(&letter))?;
    parse_locking(val)
}

fn normalize_format_filesystem(value: &str) -> Result<&'static str, String> {
    match value.trim().to_ascii_uppercase().as_str() {
        "NTFS" => Ok("NTFS"),
        "EXFAT" => Ok("exFAT"),
        "FAT32" => Ok("FAT32"),
        _ => Err("unsupported_filesystem".into()),
    }
}

fn powershell_literal(value: &str) -> String {
    value.replace('\'', "''")
}

/// Format one mounted volume, never an entire physical disk. The USB and system-volume
/// checks are repeated immediately before the destructive operation so a stale scan or
/// reassigned drive letter cannot redirect the request to another disk.
pub fn format_volume(
    drive_letter: &str,
    filesystem: &str,
    label: &str,
    full: bool,
) -> Result<(), String> {
    let letter = normalize_drive_letter(drive_letter)?;
    let filesystem = normalize_format_filesystem(filesystem)?;
    let label = label.trim();
    if label.chars().count() > 32 || label.contains(['\r', '\n', '\0']) {
        return Err("invalid_volume_label".into());
    }
    let label = powershell_literal(label);
    let full = if full { "$true" } else { "$false" };
    let script = format!(
        r#"$letter = '{letter}'
$fileSystem = '{filesystem}'
$label = '{label}'
$partition = Get-Partition -DriveLetter $letter -ErrorAction SilentlyContinue
if (-not $partition) {{ throw 'volume_not_found' }}
$disk = Get-Disk -Number $partition.DiskNumber -ErrorAction SilentlyContinue
$diskDrive = Get-CimInstance Win32_DiskDrive -Filter "Index=$($partition.DiskNumber)" -ErrorAction SilentlyContinue
$isUsb = ($disk -and [string]$disk.BusType -eq 'USB') -or ($diskDrive -and ([string]$diskDrive.InterfaceType -eq 'USB' -or [string]$diskDrive.PNPDeviceID -like 'USBSTOR\*'))
if (-not $isUsb) {{ throw 'not_usb_storage' }}
if ($letter -eq $env:SystemDrive[0] -or ($disk -and ($disk.IsSystem -or $disk.IsBoot))) {{ throw 'system_volume_forbidden' }}
$volume = Get-Volume -DriveLetter $letter -ErrorAction SilentlyContinue
if (-not $volume) {{ throw 'volume_not_found' }}
Format-Volume -DriveLetter $letter -FileSystem $fileSystem -NewFileSystemLabel $label -Full:{full} -Confirm:$false -ErrorAction Stop | Out-Null"#
    );

    // Full formatting of a large external disk may legitimately take many hours. It runs
    // on a blocking worker, so the WebView remains responsive while this timeout guards
    // against a permanently stuck provider.
    powershell::run_void_with_timeout(&script, Duration::from_secs(24 * 60 * 60))
}

fn locking_process_script(letter: &str) -> String {
    format!(
        r#"$selfProcess = Get-CimInstance Win32_Process -Filter "ProcessId=$PID" -ErrorAction SilentlyContinue
$zeroTickPid = if ($selfProcess) {{ [uint32]$selfProcess.ParentProcessId }} else {{ 0 }}
Get-CimInstance Win32_Process |
  Where-Object {{
    $_.ProcessId -ne $PID -and
    -not ($_.ParentProcessId -eq $zeroTickPid -and $_.Name -in @('powershell.exe', 'pwsh.exe')) -and
    ($_.ExecutablePath -like '{letter}:\*' -or $_.CommandLine -like '*{letter}:*')
  }} |
  Select-Object ProcessId, Name, ExecutablePath |
  ForEach-Object {{ [PSCustomObject]@{{ pid=$_.ProcessId; name=$_.Name; path=$_.ExecutablePath }} }}"#
    )
}

/// Ask an interactive application to close normally. This never force-terminates a process,
/// so the application can still ask the user to save work or refuse to close.
pub fn request_close_process(pid: u32) -> Result<UsbCloseProcessResult, String> {
    if pid <= 4 || pid == std::process::id() {
        return Ok(UsbCloseProcessResult {
            status: "protected".into(),
        });
    }
    let script = format!(
        r#"$process = Get-Process -Id {pid} -ErrorAction SilentlyContinue
if (-not $process) {{ [pscustomobject]@{{ status = 'not_found' }}; return }}
$protected = @('system','registry','smss','csrss','wininit','services','lsass','svchost','winlogon')
if ($protected -contains $process.ProcessName.ToLowerInvariant()) {{
  [pscustomobject]@{{ status = 'protected' }}
}} elseif ($process.MainWindowHandle -eq 0) {{
  [pscustomobject]@{{ status = 'no_window' }}
}} elseif ($process.CloseMainWindow()) {{
  [pscustomobject]@{{ status = 'requested' }}
}} else {{
  [pscustomobject]@{{ status = 'no_window' }}
}}"#
    );
    let value = powershell::run_json(&script)?;
    serde_json::from_value(value).map_err(|error| format!("Invalid close-process result: {error}"))
}

pub fn open_volume(drive_letter: &str) -> Result<(), String> {
    let letter = normalize_drive_letter(drive_letter)?;
    std::process::Command::new("explorer.exe")
        .arg(format!(r"{letter}:\"))
        .spawn()
        .map_err(|error| format!("usb_open:failed:{error}"))?;
    Ok(())
}

pub fn eject_drive(drive_letter: &str) -> Result<UsbEjectResult, String> {
    let letter = normalize_drive_letter(drive_letter)?;
    let target = resolve_eject_target(&letter)?;
    let mut handles = Vec::with_capacity(target.volume_letters.len());

    // Lock every mounted volume in the same physical enclosure before dismounting any of them.
    // Multi-LUN external disks are one eject unit; card-reader slots remain independent.
    for volume in &target.volume_letters {
        match open_flush_and_lock(volume) {
            Ok(handle) => handles.push(handle),
            Err((status, stage)) => {
                unlock_all(&handles);
                drop(handles);
                // Direct volume access normally requires elevation. The interactive PnP API
                // still performs Windows' own safe-removal checks, so use it as the safe
                // non-elevated path instead of making quick eject administrator-only.
                if status == "permission_required" {
                    return request_pnp_eject(&target.pnp_device_id, &target.volume_letters);
                }
                return Ok(UsbEjectResult {
                    status: status.into(),
                    stage: stage.into(),
                    blockers: find_locking_processes(&format!("{letter}:")).unwrap_or_default(),
                    veto_type: None,
                    veto_name: None,
                });
            }
        }
    }

    for handle in &handles {
        if unsafe {
            DeviceIoControl(
                handle.0,
                FSCTL_DISMOUNT_VOLUME,
                None,
                0,
                None,
                0,
                None,
                None,
            )
        }
        .is_err()
        {
            unlock_all(&handles);
            return Ok(UsbEjectResult {
                status: "failed".into(),
                stage: "dismount".into(),
                blockers: Vec::new(),
                veto_type: None,
                veto_name: None,
            });
        }
    }

    // Closing the volume handles releases our own locks before the PnP manager checks for vetoes.
    drop(handles);
    request_pnp_eject(&target.pnp_device_id, &target.volume_letters)
}

fn normalize_drive_letter(value: &str) -> Result<String, String> {
    let trimmed = value.trim().trim_end_matches([':', '\\', '/']);
    if trimmed.len() != 1 || !trimmed.as_bytes()[0].is_ascii_alphabetic() {
        return Err("Invalid drive letter".into());
    }
    Ok(trimmed.to_ascii_uppercase())
}

fn resolve_eject_target(letter: &str) -> Result<EjectTarget, String> {
    let script = format!(
        r#"$letter = '{letter}'
$partition = Get-Partition -DriveLetter $letter -ErrorAction SilentlyContinue
$disk = if ($partition) {{ Get-Disk -Number $partition.DiskNumber -ErrorAction SilentlyContinue }} else {{ $null }}
$diskDrive = if ($disk) {{ Get-CimInstance Win32_DiskDrive -Filter "Index=$($disk.Number)" -ErrorAction SilentlyContinue }} else {{ $null }}

if (-not $diskDrive) {{
  $logical = Get-CimInstance Win32_LogicalDisk -Filter "DeviceID='${{letter}}:'" -ErrorAction SilentlyContinue
  $legacyPartition = if ($logical) {{ Get-CimAssociatedInstance -InputObject $logical -ResultClassName Win32_DiskPartition -ErrorAction SilentlyContinue | Select-Object -First 1 }} else {{ $null }}
  $diskDrive = if ($legacyPartition) {{ Get-CimAssociatedInstance -InputObject $legacyPartition -ResultClassName Win32_DiskDrive -ErrorAction SilentlyContinue | Select-Object -First 1 }} else {{ $null }}
}}

if (-not $diskDrive) {{ throw 'Cannot map the drive letter to a physical disk.' }}
$isUsb = ($disk -and [string]$disk.BusType -eq 'USB') -or [string]$diskDrive.InterfaceType -eq 'USB' -or [string]$diskDrive.PNPDeviceID -like 'USBSTOR\*'
if (-not $isUsb) {{ throw 'The selected volume is not on a USB storage device.' }}
if (($disk -and ($disk.IsSystem -or $disk.IsBoot)) -or $diskDrive.Index -eq 0 -and $letter -eq $env:SystemDrive[0]) {{ throw 'The system disk cannot be ejected.' }}

$identityText = "$($diskDrive.Model) $($diskDrive.Caption)"
$isCardReader = $identityText -match '(?i)card\s*reader|multi[ -]?card|sd\s*reader|mmc|memory\s*stick|compact\s*flash|smart\s*media|xd[ -]?picture|ms[/ -]?ms-pro|读卡器'
$targetDisks = @($diskDrive)
$pnpTargetId = [string]$diskDrive.PNPDeviceID
if (-not $isCardReader -and $diskDrive.PNPDeviceID) {{
  $container = Get-PnpDeviceProperty -InstanceId $diskDrive.PNPDeviceID -KeyName 'DEVPKEY_Device_ContainerId' -ErrorAction SilentlyContinue
  if ($container -and $container.Data) {{
    $containerId = [string]$container.Data
    $sameContainer = @(Get-CimInstance Win32_DiskDrive -ErrorAction SilentlyContinue | Where-Object {{
      if (-not $_.PNPDeviceID) {{ return $false }}
      $candidateContainer = Get-PnpDeviceProperty -InstanceId $_.PNPDeviceID -KeyName 'DEVPKEY_Device_ContainerId' -ErrorAction SilentlyContinue
      $candidateContainer -and [string]$candidateContainer.Data -eq $containerId
    }})
    if ($sameContainer.Count) {{ $targetDisks = $sameContainer }}

    # A hardware-encrypted external disk can expose both a data LUN and a virtual unlock CD.
    # Walk to the highest parent that remains in the same physical container so PnP ejects
    # the enclosure, not only one child LUN. Card readers intentionally skip this step.
    $currentId = [string]$diskDrive.PNPDeviceID
    for ($i = 0; $i -lt 8; $i++) {{
      $parentProperty = Get-PnpDeviceProperty -InstanceId $currentId -KeyName 'DEVPKEY_Device_Parent' -ErrorAction SilentlyContinue
      if (-not $parentProperty -or -not $parentProperty.Data) {{ break }}
      $parentId = [string]$parentProperty.Data
      $parentContainer = Get-PnpDeviceProperty -InstanceId $parentId -KeyName 'DEVPKEY_Device_ContainerId' -ErrorAction SilentlyContinue
      if (-not $parentContainer -or [string]$parentContainer.Data -ne $containerId) {{ break }}
      $pnpTargetId = $parentId
      $currentId = $parentId
    }}
  }}
}}

$letters = @($targetDisks | ForEach-Object {{
  Get-Partition -DiskNumber $_.Index -ErrorAction SilentlyContinue |
    Where-Object {{ $_.DriveLetter }} |
    ForEach-Object {{ "$($_.DriveLetter):" }}
}} | Sort-Object -Unique)
if (-not $letters.Count) {{ $letters = @("${{letter}}:") }}

[pscustomobject]@{{
  pnp_device_id = $pnpTargetId
  volume_letters = @($letters)
}}"#
    );
    let value = powershell::run_json(&script)?;
    let target: EjectTarget = serde_json::from_value(value)
        .map_err(|error| format!("Invalid USB eject target: {error}"))?;
    if target.pnp_device_id.trim().is_empty() || target.volume_letters.is_empty() {
        return Err("USB eject target is incomplete".into());
    }
    Ok(target)
}

fn open_flush_and_lock(volume: &str) -> Result<VolumeHandle, (&'static str, &'static str)> {
    let path = format!(r"\\.\{}", volume.trim_end_matches('\\'));
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide.as_ptr()),
            GENERIC_READ.0 | GENERIC_WRITE.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }
    .map_err(|error| classify_volume_error(&error, "flush"))?;
    let handle = VolumeHandle(handle);

    unsafe { FlushFileBuffers(handle.0) }
        .map_err(|error| classify_volume_error(&error, "flush"))?;
    unsafe { DeviceIoControl(handle.0, FSCTL_LOCK_VOLUME, None, 0, None, 0, None, None) }
        .map_err(|error| classify_volume_error(&error, "lock"))?;
    Ok(handle)
}

fn classify_volume_error(
    error: &windows::core::Error,
    stage: &'static str,
) -> (&'static str, &'static str) {
    // HRESULT_FROM_WIN32(ERROR_ACCESS_DENIED) = 0x80070005.
    if error.code().0 as u32 == 0x8007_0005 {
        ("permission_required", stage)
    } else {
        // An unsuccessful lock is authoritative evidence that the volume still has open files.
        ("busy", stage)
    }
}

fn unlock_all(handles: &[VolumeHandle]) {
    for handle in handles {
        unsafe {
            let _ = DeviceIoControl(handle.0, FSCTL_UNLOCK_VOLUME, None, 0, None, 0, None, None);
        }
    }
}

fn request_pnp_eject(device_id: &str, drive_letters: &[String]) -> Result<UsbEjectResult, String> {
    let wide: Vec<u16> = device_id.encode_utf16().chain(std::iter::once(0)).collect();
    let mut dev_inst = 0u32;
    let located = unsafe {
        CM_Locate_DevNodeW(
            &mut dev_inst,
            PCWSTR(wide.as_ptr()),
            CM_LOCATE_DEVNODE_NORMAL,
        )
    };
    if located != CR_SUCCESS {
        return Ok(eject_failure("failed", "pnp", None, None));
    }

    // The disk PDO is often below the removable USB device node. Walk upward to the first
    // removable ancestor, without ever climbing past it to a hub/controller.
    let mut eject_node = dev_inst;
    let mut current = dev_inst;
    for _ in 0..8 {
        let mut capabilities = 0u32;
        let mut size = std::mem::size_of::<u32>() as u32;
        let result = unsafe {
            CM_Get_DevNode_Registry_PropertyW(
                current,
                CM_DRP_CAPABILITIES,
                None,
                Some((&mut capabilities as *mut u32).cast()),
                &mut size,
                0,
            )
        };
        if result == CR_SUCCESS && capabilities & CM_DEVCAP_REMOVABLE.0 != 0 {
            eject_node = current;
            break;
        }
        let mut parent = 0u32;
        if unsafe { CM_Get_Parent(&mut parent, current, 0) } != CR_SUCCESS {
            break;
        }
        current = parent;
    }

    let mut veto = PNP_VETO_TYPE(0);
    let mut veto_name = [0u16; 260];
    let result =
        unsafe { CM_Request_Device_EjectW(eject_node, Some(&mut veto), Some(&mut veto_name), 0) };
    if result == CR_SUCCESS || veto.0 == 13 {
        return Ok(UsbEjectResult {
            status: "ejected".into(),
            stage: "pnp".into(),
            blockers: Vec::new(),
            veto_type: None,
            veto_name: None,
        });
    }

    // Some multi-LUN devices report the last child veto even though Windows has already
    // removed the physical enclosure. Recheck the observable volume state before showing
    // a failure; this prevents a stale WD virtual-CD veto from becoming a false error.
    for _ in 0..5 {
        if logical_volumes_are_gone(drive_letters) {
            return Ok(UsbEjectResult {
                status: "ejected".into(),
                stage: "pnp".into(),
                blockers: Vec::new(),
                veto_type: None,
                veto_name: None,
            });
        }
        std::thread::sleep(std::time::Duration::from_millis(120));
    }

    let name_len = veto_name
        .iter()
        .position(|c| *c == 0)
        .unwrap_or(veto_name.len());
    let name = String::from_utf16_lossy(&veto_name[..name_len]);
    let veto_type = veto_type_name(veto).to_string();
    let status = if veto.0 == 12 {
        "permission_required"
    } else {
        "vetoed"
    };
    let mut failure = eject_failure(
        status,
        "pnp",
        Some(veto_type),
        (!name.is_empty()).then_some(name),
    );
    if veto.0 == 3 || veto.0 == 4 || veto.0 == 5 {
        if let Some(letter) = drive_letters.first() {
            failure.blockers = find_locking_processes(letter).unwrap_or_default();
        }
    }
    Ok(failure)
}

fn logical_volumes_are_gone(drive_letters: &[String]) -> bool {
    let mask = unsafe { GetLogicalDrives() };
    if mask == 0 {
        return false;
    }
    logical_volumes_are_gone_in_mask(mask, drive_letters)
}

fn logical_volumes_are_gone_in_mask(mask: u32, drive_letters: &[String]) -> bool {
    drive_letters.iter().all(|letter| {
        let normalized = letter.trim().trim_end_matches([':', '\\', '/']);
        let Some(byte) = normalized.as_bytes().first().copied() else {
            return true;
        };
        if !byte.is_ascii_alphabetic() {
            return true;
        }
        let bit = byte.to_ascii_uppercase() - b'A';
        mask & (1u32 << bit) == 0
    })
}

fn eject_failure(
    status: &str,
    stage: &str,
    veto_type: Option<String>,
    veto_name: Option<String>,
) -> UsbEjectResult {
    UsbEjectResult {
        status: status.into(),
        stage: stage.into(),
        blockers: Vec::new(),
        veto_type,
        veto_name,
    }
}

fn veto_type_name(veto: PNP_VETO_TYPE) -> &'static str {
    match veto.0 {
        1 => "legacy_device",
        2 => "pending_close",
        3 => "windows_app",
        4 => "windows_service",
        5 => "outstanding_open",
        6 => "device",
        7 => "driver",
        8 => "illegal_request",
        9 => "insufficient_power",
        10 => "non_disableable",
        11 => "legacy_driver",
        12 => "insufficient_rights",
        13 => "already_removed",
        _ => "unknown",
    }
}

pub fn repair() -> (Vec<String>, Vec<String>) {
    services::repair_group(USB)
}

fn parse_locking(val: serde_json::Value) -> Result<Vec<LockingProcess>, String> {
    let arr = match val {
        serde_json::Value::Array(a) => a,
        serde_json::Value::Object(_) => vec![val],
        serde_json::Value::Null => vec![],
        _ => return Err("进程列表格式异常".into()),
    };
    Ok(arr
        .into_iter()
        .filter_map(|item| {
            let pid = item.get("pid").and_then(|v| v.as_u64())? as u32;
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let path = item
                .get("path")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let lower = name.to_ascii_lowercase();
            let protected = matches!(
                lower.as_str(),
                "system"
                    | "registry"
                    | "smss.exe"
                    | "csrss.exe"
                    | "wininit.exe"
                    | "services.exe"
                    | "lsass.exe"
                    | "svchost.exe"
                    | "winlogon.exe"
            );
            Some(LockingProcess {
                pid,
                name,
                path,
                confidence: "possible".into(),
                can_close: !protected,
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn storage_device(id: &str, physical_id: &str, kind: &str, letter: &str) -> UsbStorageDevice {
        let disk_number = letter.as_bytes()[0] as u32;
        let slot = UsbStorageSlot {
            name: "Test slot".into(),
            instance_id: id.into(),
            access_state: "mounted".into(),
            volume_letters: vec![letter.into()],
            disk_number: Some(disk_number),
        };
        UsbStorageDevice {
            name: "Test storage".into(),
            instance_id: id.into(),
            bus_type: "USB".into(),
            status: "Online".into(),
            problem_code: 0,
            access_state: "mounted".into(),
            volume_letters: vec![letter.into()],
            physical_id: physical_id.into(),
            device_kind: kind.into(),
            disk_numbers: vec![disk_number],
            slots: vec![slot],
        }
    }

    #[test]
    fn drive_letter_is_normalized_and_restricted() {
        assert_eq!(normalize_drive_letter(" e: ").unwrap(), "E");
        assert!(normalize_drive_letter("C:\\Windows").is_err());
        assert!(normalize_drive_letter("1:").is_err());
    }

    #[test]
    fn dos_device_target_maps_card_reader_letter_to_disk_number() {
        assert_eq!(
            parse_disk_number_from_dos_target(r"\Device\Harddisk2\DP(1)0-0+11"),
            Some(2)
        );
        assert_eq!(
            parse_disk_number_from_dos_target(r"\Device\HarddiskVolume12"),
            None
        );
    }

    #[test]
    fn format_parameters_are_restricted_before_script_execution() {
        assert_eq!(normalize_format_filesystem("ntfs").unwrap(), "NTFS");
        assert_eq!(normalize_format_filesystem("exFAT").unwrap(), "exFAT");
        assert!(normalize_format_filesystem("ReFS").is_err());
        assert_eq!(powershell_literal("Owner's Disk"), "Owner''s Disk");
    }

    #[test]
    fn pnp_veto_values_have_stable_ui_ids() {
        assert_eq!(veto_type_name(PNP_VETO_TYPE(5)), "outstanding_open");
        assert_eq!(veto_type_name(PNP_VETO_TYPE(12)), "insufficient_rights");
        assert_eq!(veto_type_name(PNP_VETO_TYPE(999)), "unknown");
    }

    #[test]
    fn related_process_scan_excludes_its_own_powershell_processes() {
        let script = locking_process_script("E");
        assert!(script.contains("$_.ProcessId -ne $PID"));
        assert!(script.contains("$_.ParentProcessId -eq $zeroTickPid"));
        assert!(script.contains("'powershell.exe', 'pwsh.exe'"));
    }

    #[test]
    fn external_disk_luns_and_card_reader_slots_group_by_container() {
        let grouped = group_physical_devices(vec![
            storage_device("wd-lun-0", "wd-container", "external_disk", "E:"),
            storage_device("wd-lun-1", "wd-container", "external_disk", "F:"),
        ]);
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].volume_letters, vec!["E:", "F:"]);

        let readers = group_physical_devices(vec![
            storage_device("reader-slot-0", "reader-container", "card_reader", "G:"),
            storage_device("reader-slot-1", "reader-container", "card_reader", "H:"),
        ]);
        assert_eq!(readers.len(), 1);
        assert_eq!(readers[0].slots.len(), 2);
        assert_eq!(readers[0].volume_letters, vec!["G:", "H:"]);
    }

    #[test]
    fn empty_card_slot_absorbs_its_unavailable_zero_size_letter() {
        let mut reader = storage_device("reader-slot-0", "reader-container", "card_reader", "E:");
        reader.access_state = "no_media".into();
        reader.volume_letters.clear();
        reader.slots[0].access_state = "no_media".into();
        reader.slots[0].volume_letters.clear();
        let mut devices = group_physical_devices(vec![reader]);
        let drives = vec![UsbDrive {
            letter: "E:".into(),
            label: "E:".into(),
            size_gb: 0.0,
            free_gb: 0.0,
            filesystem: "".into(),
            access_state: "unavailable".into(),
            disk_number: None,
        }];

        attach_drive_letters_to_slots(&drives, &mut devices);
        assert_eq!(devices[0].volume_letters, vec!["E:"]);
        assert_eq!(devices[0].slots[0].volume_letters, vec!["E:"]);
        assert_eq!(devices[0].slots[0].access_state, "no_media");
    }

    #[test]
    fn generic_mass_storage_luns_with_empty_slot_are_recognized_as_card_reader() {
        let mut empty = storage_device("generic-0", "generic-container", "external_disk", "E:");
        empty.name = "Generic MassStorageClass".into();
        empty.slots[0].name = empty.name.clone();
        empty.access_state = "no_media".into();
        empty.slots[0].access_state = "no_media".into();
        let mut inserted = storage_device("generic-1", "generic-container", "external_disk", "F:");
        inserted.name = "Generic MassStorageClass".into();
        inserted.slots[0].name = inserted.name.clone();

        let devices = group_physical_devices(vec![empty, inserted]);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_kind, "card_reader");
        assert_eq!(devices[0].slots.len(), 2);
    }

    #[test]
    fn mounted_lun_is_not_overridden_by_offline_companion_lun() {
        let mounted = storage_device("wd-data", "wd-container", "external_disk", "E:");
        let mut offline = storage_device("wd-virtual-cd", "wd-container", "external_disk", "F:");
        offline.access_state = "offline".into();
        offline.volume_letters.clear();

        let grouped = group_physical_devices(vec![mounted, offline]);
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].access_state, "mounted");
        assert_eq!(grouped[0].volume_letters, vec!["E:"]);
    }

    #[test]
    fn stale_veto_is_ignored_only_after_all_target_volumes_disappear() {
        let targets = vec!["E:".to_string(), "F:".to_string()];
        let e_present = 1u32 << (b'E' - b'A');
        let f_present = 1u32 << (b'F' - b'A');
        assert!(!logical_volumes_are_gone_in_mask(
            e_present | f_present,
            &targets
        ));
        assert!(!logical_volumes_are_gone_in_mask(f_present, &targets));
        assert!(logical_volumes_are_gone_in_mask(1, &targets));
    }
}
