//! 持久 WMI 工作线程 — COM 按 wmi crate 要求在同一线程单例初始化，避免 0x80010106

use std::sync::mpsc::{self, RecvTimeoutError, SyncSender, TrySendError};
use std::sync::OnceLock;
use std::thread::{self, JoinHandle};
use std::time::Duration;
use wmi::{WMIConnection, WMIError};

type WmiJob = Box<dyn FnOnce(&WMIConnection) + Send>;

struct WmiWorker {
    tx: SyncSender<WmiJob>,
    _handle: JoinHandle<()>,
}

static WORKER: OnceLock<WmiWorker> = OnceLock::new();

fn worker_main(rx: mpsc::Receiver<WmiJob>) {
    let wmi = match WMIConnection::new() {
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
        .try_send(Box::new(move |wmi| {
            let result = f(wmi).map_err(|e| format!("{e}"));
            let _ = reply_tx.send(result);
        }))
        .map_err(|error| match error {
            TrySendError::Full(_) => "WMI query queue is full".to_string(),
            TrySendError::Disconnected(_) => "WMI worker has stopped".to_string(),
        })?;

    let timeout = Duration::from_secs(crate::settings::get().system_query_timeout_secs);
    match reply_rx.recv_timeout(timeout) {
        Ok(result) => result,
        Err(RecvTimeoutError::Timeout) => Err(format!(
            "WMI query timed out after {} seconds",
            timeout.as_secs()
        )),
        Err(RecvTimeoutError::Disconnected) => Err("WMI worker did not respond".to_string()),
    }
}
