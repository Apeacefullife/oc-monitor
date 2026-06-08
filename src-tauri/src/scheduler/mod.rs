/// 定时刷新调度器
///
/// 使用 tokio 定时器，按设定间隔轮询 DeepSeek API
/// 将最新数据通过 Tauri 事件推送给前端

use std::sync::Arc;
use tokio::sync::Mutex;

/// 调度器状态
pub struct Scheduler {
    /// 是否正在运行
    running: Arc<Mutex<bool>>,
    /// 刷新间隔（秒）
    interval_secs: Arc<Mutex<u64>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            running: Arc::new(Mutex::new(false)),
            interval_secs: Arc::new(Mutex::new(60)),
        }
    }

    /// 开始调度
    #[allow(dead_code)]
    pub async fn start(&self) {
        let mut running = self.running.lock().await;
        *running = true;
        // TODO: 启动 tokio 定时任务
    }

    /// 停止调度
    #[allow(dead_code)]
    pub async fn stop(&self) {
        let mut running = self.running.lock().await;
        *running = false;
    }

    /// 设置刷新间隔
    #[allow(dead_code)]
    pub async fn set_interval(&self, secs: u64) {
        let mut interval = self.interval_secs.lock().await;
        *interval = secs;
    }
}
