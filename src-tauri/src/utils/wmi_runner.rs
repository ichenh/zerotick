//! 持久 WMI 工作线程 — COM 按 wmi crate 要求在同一线程单例初始化，避免 0x80010106

use std::sync::mpsc::{self, SyncSender};
use std::sync::OnceLock;
use std::thread::{self, JoinHandle};
use wmi::{COMLibrary, WMIConnection, WMIError};

type WmiJob = Box<dyn FnOnce(&WMIConnection) + Send>;

struct WmiWorker {
    tx: SyncSender<WmiJob>,
    _handle: JoinHandle<()>,
}

static WORKER: OnceLock<WmiWorker> = OnceLock::new();

fn worker_main(rx: mpsc::Receiver<WmiJob>) {
    let com = match COMLibrary::without_security() {
        Ok(c) => c,
        Err(e) => {
            crate::utils::logging::error(format!("WMI COM 初始化失败: {e}"));
            return;
        }
    };
    let wmi = match WMIConnection::new(com) {
        Ok(w) => w,
        Err(e) => {
            crate::utils::logging::error(format!("WMI 连接失败: {e}"));
            return;
        }
    };

    while let Ok(job) = rx.recv() {
        job(&wmi);
    }
}

fn worker() -> &'static WmiWorker {
    WORKER.get_or_init(|| {
        let (tx, rx) = mpsc::sync_channel::<WmiJob>(32);
        let handle = thread::Builder::new()
            .name("zerotick-wmi".into())
            .spawn(move || worker_main(rx))
            .expect("spawn zerotick-wmi thread");
        WmiWorker {
            tx,
            _handle: handle,
        }
    })
}

/// 在专用 WMI 线程执行查询（COM 仅初始化一次）
pub fn run<T, F>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&WMIConnection) -> Result<T, WMIError> + Send + 'static,
{
    let (reply_tx, reply_rx) = mpsc::channel();
    worker()
        .tx
        .send(Box::new(move |wmi| {
            let result = f(wmi).map_err(|e| format!("{e}"));
            let _ = reply_tx.send(result);
        }))
        .map_err(|_| "WMI 工作线程已退出".to_string())?;

    reply_rx
        .recv()
        .map_err(|_| "WMI 工作线程无响应".to_string())?
}
