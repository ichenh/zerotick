//! 检测当前进程是否以管理员权限运行，并请求 UAC 提升重启

use std::os::windows::ffi::OsStrExt;
use windows::core::PCWSTR;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOW;

/// 当前进程是否已提升（管理员 / UAC 已授权）
pub fn is_elevated() -> bool {
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut ret_len = 0u32;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some((&mut elevation as *mut TOKEN_ELEVATION).cast()),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut ret_len,
        )
        .is_ok();

        ok && elevation.TokenIsElevated != 0
    }
}

/// 以管理员身份重新启动当前可执行文件（触发 UAC）
pub fn relaunch_as_admin(background: bool) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("无法获取程序路径: {e}"))?;
    let wide: Vec<u16> = exe
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let background_arg: Vec<u16> = "--background\0".encode_utf16().collect();
    let parameters = if background {
        PCWSTR(background_arg.as_ptr())
    } else {
        PCWSTR::null()
    };

    unsafe {
        let result = ShellExecuteW(
            None,
            windows::core::w!("runas"),
            PCWSTR(wide.as_ptr()),
            parameters,
            PCWSTR::null(),
            SW_SHOW,
        );
        if result.0 as isize <= 32 {
            return Err("用户取消了 UAC 提升或未获得管理员权限".into());
        }
    }
    Ok(())
}
