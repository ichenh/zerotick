//! 设备友好名称解析 — SetupAPI 精确匹配 + 注册表实例回退

use std::collections::HashMap;
use std::mem::size_of;
use std::sync::{Mutex, OnceLock};
use windows::core::PCWSTR;
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    SetupDiDestroyDeviceInfoList, SetupDiGetClassDevsW, SetupDiGetDeviceInterfaceDetailW,
    SetupDiGetDeviceRegistryPropertyW, SetupDiOpenDeviceInterfaceW, DIGCF_ALLCLASSES,
    DIGCF_DEVICEINTERFACE, DIGCF_PRESENT, HDEVINFO, SETUP_DI_GET_CLASS_DEVS_FLAGS,
    SETUP_DI_REGISTRY_PROPERTY, SP_DEVINFO_DATA, SP_DEVICE_INTERFACE_DATA,
    SP_DEVICE_INTERFACE_DETAIL_DATA_W, SPDRP_DEVICEDESC, SPDRP_FRIENDLYNAME,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ,
};

static NAME_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn cache() -> &'static Mutex<HashMap<String, String>> {
    NAME_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 根据设备接口路径解析友好名称（SetupAPI → 实例注册表 → VID 回退）
pub fn resolve(device_path: &str, vid_pid: Option<&str>) -> Option<String> {
    if let Ok(guard) = cache().lock() {
        if let Some(hit) = guard.get(device_path) {
            return Some(hit.clone());
        }
    }

    let name = resolve_via_setupapi(device_path)
        .or_else(|| resolve_via_instance_registry(device_path))
        .or_else(|| vid_pid.and_then(resolve_usb_name_first))
        .or_else(|| resolve_bluetooth_name(device_path));

    if let Some(ref n) = name {
        if let Ok(mut guard) = cache().lock() {
            guard.insert(device_path.to_string(), n.clone());
        }
    }

    name
}

/// SetupAPI：通过设备接口路径精确定位到单一设备实例
fn resolve_via_setupapi(device_path: &str) -> Option<String> {
    unsafe {
        let path_wide: Vec<u16> = device_path.encode_utf16().chain(std::iter::once(0)).collect();
        let flags = SETUP_DI_GET_CLASS_DEVS_FLAGS(
            DIGCF_ALLCLASSES.0 | DIGCF_DEVICEINTERFACE.0 | DIGCF_PRESENT.0,
        );
        let dev_info = SetupDiGetClassDevsW(None, None, None, flags).ok()?;

        let result = resolve_setupapi_inner(dev_info, &path_wide);
        let _ = SetupDiDestroyDeviceInfoList(dev_info);
        result
    }
}

unsafe fn resolve_setupapi_inner(dev_info: HDEVINFO, path_wide: &[u16]) -> Option<String> {
    let mut if_data = SP_DEVICE_INTERFACE_DATA {
        cbSize: size_of::<SP_DEVICE_INTERFACE_DATA>() as u32,
        ..Default::default()
    };

    SetupDiOpenDeviceInterfaceW(dev_info, PCWSTR(path_wide.as_ptr()), 0, Some(&mut if_data)).ok()?;

    let mut dev_data = SP_DEVINFO_DATA {
        cbSize: size_of::<SP_DEVINFO_DATA>() as u32,
        ..Default::default()
    };

    let mut required = 0u32;
    let _ = SetupDiGetDeviceInterfaceDetailW(
        dev_info,
        &if_data,
        None,
        0,
        Some(&mut required),
        Some(&mut dev_data),
    );

    if required == 0 {
        return None;
    }

    let mut buffer = vec![0u8; required as usize];
    let detail_ptr = buffer.as_mut_ptr() as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W;
    (*detail_ptr).cbSize = size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>() as u32;

    SetupDiGetDeviceInterfaceDetailW(
        dev_info,
        &if_data,
        Some(detail_ptr),
        required,
        None,
        Some(&mut dev_data),
    )
    .ok()?;

    read_setup_property(dev_info, &dev_data, SPDRP_FRIENDLYNAME)
        .or_else(|| read_setup_property(dev_info, &dev_data, SPDRP_DEVICEDESC))
        .map(|s| clean_registry_string(&s))
}

unsafe fn read_setup_property(
    dev_info: HDEVINFO,
    dev_data: &SP_DEVINFO_DATA,
    property: SETUP_DI_REGISTRY_PROPERTY,
) -> Option<String> {
    let mut buf = [0u8; 512];
    let mut required = 0u32;
    SetupDiGetDeviceRegistryPropertyW(
        dev_info,
        dev_data,
        property,
        None,
        Some(&mut buf),
        Some(&mut required),
    )
    .ok()?;

    decode_reg_sz(&buf, required)
}

/// 从接口路径解析实例 ID，读取对应注册表项
fn resolve_via_instance_registry(device_path: &str) -> Option<String> {
    let (bus, hardware_id, instance) = parse_interface_path(device_path)?;
    let key_path = format!(r"SYSTEM\CurrentControlSet\Enum\{bus}\{hardware_id}\{instance}");
    read_reg_key_name(&key_path)
}

/// 解析 `\\?\USB#VID_xxxx&PID_xxxx#instance#{guid}` 结构
fn parse_interface_path(device_path: &str) -> Option<(String, String, String)> {
    let trimmed = device_path.trim_start_matches(r"\\?\");
    let mut parts = trimmed.split('#');
    let bus = parts.next()?.to_string();
    let hardware_id = parts.next()?.to_string();
    let instance = parts.next()?.to_string();
    if bus.is_empty() || hardware_id.is_empty() || instance.is_empty() {
        return None;
    }
    Some((bus, hardware_id, instance))
}

fn read_reg_key_name(enum_key_path: &str) -> Option<String> {
    unsafe {
        let wide_path: Vec<u16> = enum_key_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut key = HKEY::default();
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(wide_path.as_ptr()),
            None,
            KEY_READ,
            &mut key,
        )
        .is_err()
        {
            return None;
        }

        let name = read_reg_string(key, "FriendlyName").or_else(|| read_reg_string(key, "DeviceDesc"));
        let _ = RegCloseKey(key);
        name.map(|s| clean_registry_string(&s))
    }
}

fn resolve_usb_name_first(vid_pid: &str) -> Option<String> {
    let key_path = format!(r"SYSTEM\CurrentControlSet\Enum\USB\{vid_pid}");
    read_first_child_name(&key_path)
}

fn resolve_bluetooth_name(device_path: &str) -> Option<String> {
    if !device_path.to_uppercase().contains("BTH") {
        return None;
    }
    read_first_child_name(r"SYSTEM\CurrentControlSet\Enum\BTHENUM")
        .or_else(|| read_first_child_name(r"SYSTEM\CurrentControlSet\Enum\BTHLE"))
}

fn read_first_child_name(enum_key_path: &str) -> Option<String> {
    unsafe {
        let wide_path: Vec<u16> = enum_key_path
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut enum_key = HKEY::default();
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(wide_path.as_ptr()),
            None,
            KEY_READ,
            &mut enum_key,
        )
        .is_err()
        {
            return None;
        }

        let mut name_buf = [0u16; 256];
        let mut name_len = name_buf.len() as u32;
        if windows::Win32::System::Registry::RegEnumKeyExW(
            enum_key,
            0,
            Some(windows::core::PWSTR(name_buf.as_mut_ptr())),
            &mut name_len,
            None,
            None,
            None,
            None,
        )
        .is_err()
        {
            let _ = RegCloseKey(enum_key);
            return None;
        }

        let mut inst_key = HKEY::default();
        if RegOpenKeyExW(
            enum_key,
            PCWSTR(name_buf.as_ptr()),
            None,
            KEY_READ,
            &mut inst_key,
        )
        .is_err()
        {
            let _ = RegCloseKey(enum_key);
            return None;
        }

        let name = read_reg_string(inst_key, "FriendlyName")
            .or_else(|| read_reg_string(inst_key, "DeviceDesc"));
        let _ = RegCloseKey(inst_key);
        let _ = RegCloseKey(enum_key);
        name.map(|s| clean_registry_string(&s))
    }
}

unsafe fn read_reg_string(key: HKEY, value_name: &str) -> Option<String> {
    let wide_name: Vec<u16> = value_name
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut buf = [0u8; 512];
    let mut buf_size = buf.len() as u32;
    let mut value_type = REG_SZ;

    if RegQueryValueExW(
        key,
        PCWSTR(wide_name.as_ptr()),
        None,
        Some(&mut value_type),
        Some(buf.as_mut_ptr()),
        Some(&mut buf_size),
    )
    .is_err()
    {
        return None;
    }

    decode_reg_sz(&buf, buf_size)
}

fn decode_reg_sz(buf: &[u8], size: u32) -> Option<String> {
    if size < 2 {
        return None;
    }
    let wchar_count = (size as usize / 2).saturating_sub(1);
    let wide: Vec<u16> = buf
        .chunks_exact(2)
        .take(wchar_count)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    let s = String::from_utf16_lossy(&wide);
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn clean_registry_string(raw: &str) -> String {
    if let Some(idx) = raw.rfind(';') {
        raw[idx + 1..].trim().to_string()
    } else {
        raw.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleans_registry_device_desc() {
        assert_eq!(
            clean_registry_string("@oem42.inf,%device%;Logitech USB Receiver"),
            "Logitech USB Receiver"
        );
    }

    #[test]
    fn parses_usb_interface_path() {
        let path = r"\\?\USB#VID_046D&PID_C52B#6&1a2b3c4d&0&1#{a5dcbF10-6530-11d2-901f-00c04fb951ed}";
        let (bus, hw, inst) = parse_interface_path(path).unwrap();
        assert_eq!(bus, "USB");
        assert_eq!(hw, "VID_046D&PID_C52B");
        assert_eq!(inst, "6&1a2b3c4d&0&1");
    }
}
