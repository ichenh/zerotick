//! 登录自启：普通模式使用当前用户启动项，管理员模式使用最高权限计划任务。

use crate::utils::{elevated, logging};
use std::sync::{Mutex, OnceLock};
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;
use windows::core::{Interface, BSTR};
use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, RPC_E_CHANGED_MODE, VARIANT_BOOL};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::System::TaskScheduler::{
    IExecAction, ILogonTrigger, ITaskFolder, ITaskService, TaskScheduler, TASK_ACTION_EXEC,
    TASK_CREATE_OR_UPDATE, TASK_INSTANCES_IGNORE_NEW, TASK_LOGON_INTERACTIVE_TOKEN,
    TASK_RUNLEVEL_HIGHEST, TASK_TRIGGER_LOGON,
};
use windows::Win32::System::Variant::VARIANT;

const ELEVATED_TASK_PREFIX: &str = "ZeroTick Elevated Startup";
const BACKGROUND_ARG: &str = "--background";

static LAST_ERROR: OnceLock<Mutex<Option<String>>> = OnceLock::new();

/// 启动后同步自启状态。错误保留给设置页读取，不弹终端或系统脚本窗口。
pub fn sync_on_startup(app: &AppHandle, enabled: bool, run_as_admin: bool) {
    let result = sync(app, enabled, run_as_admin);
    if let Err(error) = &result {
        logging::warn(format!("登录自启同步失败: {error}"));
    }
    set_last_error(result.err());
}

/// 用户保存设置时同步。管理员自启必须由已提升进程创建，以免形成每次登录都弹 UAC 的循环。
pub fn sync(app: &AppHandle, enabled: bool, run_as_admin: bool) -> Result<(), String> {
    if enabled && run_as_admin {
        if !elevated::is_elevated() {
            return Err(
                "AUTOSTART_ADMIN_REQUIRED: 需要先确认一次管理员权限，才能建立静默的管理员自启任务"
                    .into(),
            );
        }
        enable_elevated_startup(app)
    } else if enabled {
        disable_elevated_task()?;
        enable_standard_startup(app)
    } else {
        disable_standard_startup(app)?;
        disable_elevated_task()
    }
}

pub fn take_last_error() -> Option<String> {
    LAST_ERROR
        .get_or_init(|| Mutex::new(None))
        .lock()
        .ok()
        .and_then(|mut error| error.take())
}

fn set_last_error(error: Option<String>) {
    if let Ok(mut slot) = LAST_ERROR.get_or_init(|| Mutex::new(None)).lock() {
        *slot = error;
    }
}

fn enable_elevated_startup(app: &AppHandle) -> Result<(), String> {
    register_elevated_task().map_err(|error| {
        format!("AUTOSTART_TASK_FAILED: Windows 无法建立最高权限登录任务: {error}")
    })?;

    if let Err(error) = disable_standard_startup(app) {
        let _ = disable_elevated_task();
        return Err(format!(
            "AUTOSTART_TASK_FAILED: 无法移除旧的普通自启项，已回滚管理员自启任务: {error}"
        ));
    }
    Ok(())
}

fn enable_standard_startup(app: &AppHandle) -> Result<(), String> {
    app.autolaunch().enable().map_err(|error| {
        format!("AUTOSTART_STANDARD_FAILED: Windows 无法建立当前用户启动项: {error}")
    })
}

fn disable_standard_startup(app: &AppHandle) -> Result<(), String> {
    let manager = app.autolaunch();
    match manager.is_enabled() {
        Ok(true) => match manager.disable() {
            Ok(()) => Ok(()),
            Err(error) if is_not_found_message(&error.to_string()) => {
                logging::warn("检测到失效的普通自启项，已按未启用处理");
                Ok(())
            }
            Err(error) => Err(format!(
                "AUTOSTART_STANDARD_FAILED: Windows 无法移除当前用户启动项: {error}"
            )),
        },
        Ok(false) => Ok(()),
        Err(error) => Err(format!(
            "AUTOSTART_STANDARD_FAILED: Windows 无法读取当前用户启动项: {error}"
        )),
    }
}

fn is_not_found_message(message: &str) -> bool {
    let lower = message.to_lowercase();
    message.contains("找不到") || lower.contains("not found") || lower.contains("cannot find")
}

fn with_task_scheduler<T>(
    operation: impl FnOnce(&ITaskService, &ITaskFolder) -> windows::core::Result<T>,
) -> Result<T, String> {
    unsafe {
        let init = CoInitializeEx(None, COINIT_MULTITHREADED);
        let uninitialize = if init.is_ok() {
            true
        } else if init == RPC_E_CHANGED_MODE {
            false
        } else {
            return Err(format!("COM 初始化失败: {init:?}"));
        };

        let result = (|| {
            let service: ITaskService =
                CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;
            let empty = VARIANT::default();
            service.Connect(&empty, &empty, &empty, &empty)?;
            let root = service.GetFolder(&BSTR::from("\\"))?;
            operation(&service, &root)
        })()
        .map_err(|error| error.to_string());

        if uninitialize {
            CoUninitialize();
        }
        result
    }
}

fn current_task_user(service: &ITaskService) -> windows::core::Result<String> {
    unsafe {
        let user = service.ConnectedUser()?.to_string();
        let domain = service.ConnectedDomain()?.to_string();
        if domain.is_empty() || user.contains('\\') {
            Ok(user)
        } else {
            Ok(format!("{domain}\\{user}"))
        }
    }
}

fn current_user_task_name(service: &ITaskService) -> windows::core::Result<String> {
    let user = current_task_user(service)?;
    let safe_user: String = user
        .chars()
        .map(|character| match character {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect();
    Ok(format!("{ELEVATED_TASK_PREFIX} - {safe_user}"))
}

fn register_elevated_task() -> Result<(), String> {
    let executable =
        std::env::current_exe().map_err(|error| format!("无法读取 ZeroTick 程序路径: {error}"))?;
    let executable = executable.to_string_lossy().into_owned();
    let working_directory = std::path::Path::new(&executable)
        .parent()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default();

    with_task_scheduler(|service, root| unsafe {
        let user = current_task_user(service)?;
        let task_name = current_user_task_name(service)?;
        let definition = service.NewTask(0)?;

        let registration = definition.RegistrationInfo()?;
        registration.SetAuthor(&BSTR::from("ZeroTick"))?;
        registration.SetDescription(&BSTR::from(
            "Starts ZeroTick silently with administrator rights when this user signs in.",
        ))?;

        let principal = definition.Principal()?;
        principal.SetUserId(&BSTR::from(user.as_str()))?;
        principal.SetLogonType(TASK_LOGON_INTERACTIVE_TOKEN)?;
        principal.SetRunLevel(TASK_RUNLEVEL_HIGHEST)?;

        let settings = definition.Settings()?;
        settings.SetEnabled(VARIANT_BOOL(-1))?;
        settings.SetAllowDemandStart(VARIANT_BOOL(-1))?;
        settings.SetStartWhenAvailable(VARIANT_BOOL(-1))?;
        settings.SetDisallowStartIfOnBatteries(VARIANT_BOOL(0))?;
        settings.SetStopIfGoingOnBatteries(VARIANT_BOOL(0))?;
        settings.SetMultipleInstances(TASK_INSTANCES_IGNORE_NEW)?;
        // ZeroTick 是常驻托盘程序，不能使用任务计划程序默认的 72 小时上限。
        settings.SetExecutionTimeLimit(&BSTR::from("PT0S"))?;

        let trigger = definition.Triggers()?.Create(TASK_TRIGGER_LOGON)?;
        let logon_trigger: ILogonTrigger = trigger.cast()?;
        logon_trigger.SetUserId(&BSTR::from(user.as_str()))?;
        logon_trigger.SetDelay(&BSTR::from("PT5S"))?;

        let action = definition.Actions()?.Create(TASK_ACTION_EXEC)?;
        let exec_action: IExecAction = action.cast()?;
        exec_action.SetPath(&BSTR::from(executable.as_str()))?;
        exec_action.SetArguments(&BSTR::from(BACKGROUND_ARG))?;
        if !working_directory.is_empty() {
            exec_action.SetWorkingDirectory(&BSTR::from(working_directory.as_str()))?;
        }

        let empty = VARIANT::default();
        root.RegisterTaskDefinition(
            &BSTR::from(task_name.as_str()),
            &definition,
            TASK_CREATE_OR_UPDATE.0,
            &empty,
            &empty,
            TASK_LOGON_INTERACTIVE_TOKEN,
            &empty,
        )?;
        Ok(())
    })
}

fn disable_elevated_task() -> Result<(), String> {
    with_task_scheduler(|service, root| unsafe {
        let task_name = current_user_task_name(service)?;
        let name = BSTR::from(task_name.as_str());
        match root.GetTask(&name) {
            Ok(_) => root.DeleteTask(&name, 0),
            Err(error)
                if error.code() == windows::core::HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0)
                    || error.code() == windows::core::HRESULT(0x8004_130f_u32 as i32) =>
            {
                Ok(())
            }
            Err(error) => Err(error),
        }
    })
    .map_err(|error| format!("AUTOSTART_TASK_FAILED: Windows 无法移除最高权限登录任务: {error}"))
}
