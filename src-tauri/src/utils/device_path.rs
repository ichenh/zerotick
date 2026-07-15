//! 设备接口路径解析与分类。

use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceCategory {
    Usb,
    Bluetooth,
    Unknown,
}

impl DeviceCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Usb => "usb",
            Self::Bluetooth => "bluetooth",
            Self::Unknown => "unknown",
        }
    }
}

pub fn parse_device_path(path: &str) -> (DeviceCategory, Option<String>) {
    let upper = path.to_uppercase();
    let category = if upper.contains("BTHENUM")
        || upper.contains("\\BTH\\")
        || upper.contains("BTHPORT")
        || upper.contains("BTHLE")
    {
        DeviceCategory::Bluetooth
    } else if upper.contains("VID_") || upper.contains("USB\\") {
        DeviceCategory::Usb
    } else {
        DeviceCategory::Unknown
    };
    (category, extract_vid_pid(&upper))
}

fn extract_vid_pid(upper_path: &str) -> Option<String> {
    let start = upper_path.find("VID_")?;
    let rest = &upper_path[start..];
    let pid_marker = rest.find("&PID_")?;
    let pid_start = pid_marker + 5;
    let pid_end = rest[pid_start..]
        .find(|c: char| !c.is_ascii_hexdigit())
        .map(|i| pid_start + i)
        .unwrap_or(rest.len());
    Some(rest[..pid_end].to_string())
}

pub fn is_transient_disconnect(elapsed: Duration, threshold: Duration) -> bool {
    elapsed < threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_usb_path() {
        let path =
            r"\\?\USB#VID_046D&PID_C52B#6&1a2b3c4d&0&1#{a5dcbF10-6530-11d2-901f-00c04fb951ed}";
        let (cat, vid_pid) = parse_device_path(path);
        assert_eq!(cat, DeviceCategory::Usb);
        assert_eq!(vid_pid.as_deref(), Some("VID_046D&PID_C52B"));
    }
}
