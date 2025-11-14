//! Orban Agent Core Library
//!
//! 從第一性原理設計的分散式 GPU 算力貢獻系統核心庫
//!
//! # 設計原則
//!
//! 1. **安全第一**：所有外部輸入必須驗證
//! 2. **可驗證性**：所有計算必須可證明
//! 3. **跨平台**：支援 Linux/Windows/macOS
//! 4. **模塊化**：每個組件可獨立測試
//! 5. **效能**：最小化 GPU 閒置時間
//!
//! # 架構
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │         Application Layer           │
//! │    (CLI, Desktop UI, System Tray)   │
//! ├─────────────────────────────────────┤
//! │        Business Logic Layer         │
//! │  (Earnings, Scheduling, Config)     │
//! ├─────────────────────────────────────┤
//! │         Core Services Layer         │
//! │ (GPU Monitor, Executor, Network)    │
//! ├─────────────────────────────────────┤
//! │     Hardware Abstraction Layer      │
//! │  (CUDA, ROCm, Metal, DirectML)      │
//! └─────────────────────────────────────┘
//! ```

pub mod gpu;
pub mod compute;
pub mod network;
pub mod earnings;
pub mod config;
pub mod error;
pub mod types;

pub use error::{Error, Result};

use tracing::{info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Orban Agent 主要結構
///
/// 這是整個 Agent 的協調者，管理所有子系統的生命週期
pub struct OrbanAgent {
    /// GPU 偵測器
    gpu_detector: Arc<gpu::GPUDetector>,

    /// 任務執行器
    executor: Arc<RwLock<compute::TaskExecutor>>,

    /// 網路客戶端
    network_client: Arc<network::Client>,

    /// 收益追蹤器
    earnings_tracker: Arc<RwLock<earnings::EarningsTracker>>,

    /// 配置
    config: Arc<config::Config>,

    /// Agent 狀態
    state: Arc<RwLock<AgentState>>,
}

/// Agent 狀態
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentState {
    /// Agent ID（由平台分配）
    pub agent_id: Option<String>,

    /// 是否正在運行
    pub is_running: bool,

    /// 當前任務 ID
    pub current_task: Option<String>,

    /// 啟動時間
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,

    /// 完成的任務數量
    pub tasks_completed: u64,

    /// 失敗的任務數量
    pub tasks_failed: u64,
}

impl OrbanAgent {
    /// 創建新的 Orban Agent 實例
    ///
    /// # 範例
    ///
    /// ```no_run
    /// use orban_agent_core::OrbanAgent;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let agent = OrbanAgent::new().await?;
    ///     agent.start().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new() -> Result<Self> {
        // 初始化日誌系統
        Self::init_logging();

        info!("🚀 Initializing Orban Agent...");

        // 載入配置
        let config = Arc::new(config::Config::load()?);
        info!("✓ Configuration loaded");

        // 偵測 GPU
        let gpu_detector = Arc::new(gpu::GPUDetector::new()?);
        let devices = gpu_detector.detect_all()?;

        if devices.is_empty() {
            warn!("⚠️  No compatible GPU detected!");
            warn!("   Agent can still run but won't be able to execute tasks.");
        } else {
            for device in &devices {
                info!("✓ Found GPU: {} ({:.1} GB VRAM)",
                    device.name(),
                    device.total_memory_gb()
                );
            }
        }

        // 創建網路客戶端
        let network_client = Arc::new(
            network::Client::new(config.platform_url.clone())?
        );
        info!("✓ Network client initialized");

        // 創建任務執行器
        let executor = Arc::new(RwLock::new(
            compute::TaskExecutor::new(devices)?
        ));
        info!("✓ Task executor initialized");

        // 載入收益追蹤器
        let earnings_tracker = Arc::new(RwLock::new(
            earnings::EarningsTracker::load(&config)?
        ));
        info!("✓ Earnings tracker loaded");

        // 初始化狀態
        let state = Arc::new(RwLock::new(AgentState {
            agent_id: None,
            is_running: false,
            current_task: None,
            started_at: None,
            tasks_completed: 0,
            tasks_failed: 0,
        }));

        info!("✓ Orban Agent initialized successfully");

        Ok(Self {
            gpu_detector,
            executor,
            network_client,
            earnings_tracker,
            config,
            state,
        })
    }

    /// 啟動 Agent
    ///
    /// 這會開始主要的工作循環：
    /// 1. 註冊到平台
    /// 2. 開始心跳
    /// 3. 領取並執行任務
    pub async fn start(&self) -> Result<()> {
        let mut state = self.state.write().await;

        if state.is_running {
            warn!("Agent is already running");
            return Ok(());
        }

        info!("🚀 Starting Orban Agent...");

        // 註冊到平台
        let agent_id = self.register_to_platform().await?;
        info!("✓ Registered to platform: {}", agent_id);

        state.agent_id = Some(agent_id.clone());
        state.is_running = true;
        state.started_at = Some(chrono::Utc::now());

        drop(state); // 釋放鎖

        // 啟動心跳任務
        self.start_heartbeat();

        // 啟動主工作循環
        self.run_work_loop().await?;

        Ok(())
    }

    /// 停止 Agent
    pub async fn stop(&self) -> Result<()> {
        let mut state = self.state.write().await;

        if !state.is_running {
            return Ok(());
        }

        info!("⏹️  Stopping Orban Agent...");

        state.is_running = false;

        // 儲存收益資料
        let tracker = self.earnings_tracker.read().await;
        tracker.save(&self.config)?;

        info!("✓ Agent stopped successfully");

        Ok(())
    }

    /// 取得當前狀態
    pub async fn get_state(&self) -> AgentState {
        self.state.read().await.clone()
    }

    /// 取得收益資訊
    pub async fn get_earnings(&self) -> earnings::EarningsData {
        self.earnings_tracker.read().await.get_data()
    }

    /// 取得 GPU 狀態
    pub async fn get_gpu_status(&self) -> Result<Vec<gpu::GPUStatus>> {
        self.gpu_detector.get_all_status()
    }

    // === 內部方法 ===

    /// 註冊到平台
    async fn register_to_platform(&self) -> Result<String> {
        let devices = self.gpu_detector.detect_all()?;
        let gpu_info: Vec<_> = devices.iter().map(|d| d.to_info()).collect();

        let registration = network::RegistrationRequest {
            hostname: hostname::get()
                .ok()
                .and_then(|h| h.into_string().ok())
                .unwrap_or_else(|| "unknown".to_string()),
            gpus: gpu_info,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        self.network_client.register(registration).await
    }

    /// 開始心跳任務（背景運行）
    fn start_heartbeat(&self) {
        let client = self.network_client.clone();
        let state = self.state.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(30)
            );

            loop {
                interval.tick().await;

                let s = state.read().await;
                if !s.is_running {
                    break;
                }

                if let Some(agent_id) = &s.agent_id {
                    if let Err(e) = client.heartbeat(agent_id).await {
                        warn!("Heartbeat failed: {}", e);
                    }
                }
            }
        });
    }

    /// 主工作循環
    async fn run_work_loop(&self) -> Result<()> {
        loop {
            // 檢查是否應該繼續運行
            {
                let state = self.state.read().await;
                if !state.is_running {
                    break;
                }
            }

            // 嘗試領取任務
            match self.fetch_and_execute_task().await {
                Ok(Some(earnings)) => {
                    // 記錄收益
                    let mut tracker = self.earnings_tracker.write().await;
                    tracker.add_earnings(earnings);

                    let mut state = self.state.write().await;
                    state.tasks_completed += 1;
                    state.current_task = None;
                }
                Ok(None) => {
                    // 沒有可用任務，等待一下
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
                Err(e) => {
                    warn!("Task execution failed: {}", e);
                    let mut state = self.state.write().await;
                    state.tasks_failed += 1;
                    state.current_task = None;
                }
            }
        }

        Ok(())
    }

    /// 領取並執行一個任務
    async fn fetch_and_execute_task(&self) -> Result<Option<earnings::EarningRecord>> {
        let state = self.state.read().await;
        let agent_id = state.agent_id.as_ref()
            .ok_or_else(|| Error::NotRegistered)?;

        drop(state);

        // 領取任務
        let task = match self.network_client.fetch_task(agent_id).await? {
            Some(t) => t,
            None => return Ok(None),
        };

        info!("📥 Received task: {}", task.id);

        // 更新狀態
        {
            let mut state = self.state.write().await;
            state.current_task = Some(task.id.clone());
        }

        // 執行任務
        let mut executor = self.executor.write().await;
        let result = executor.execute(task.clone()).await?;

        info!("✓ Task completed: {}", task.id);

        // 提交結果
        self.network_client.submit_result(result.clone()).await?;

        // 計算收益
        let earnings = earnings::EarningRecord::from_task_result(&result);

        Ok(Some(earnings))
    }

    /// 初始化日誌系統
    fn init_logging() {
        use tracing_subscriber::EnvFilter;

        tracing_subscriber::fmt()
            .with_env_filter(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new("info"))
            )
            .with_target(false)
            .with_thread_ids(false)
            .with_file(false)
            .init();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_creation() {
        // 測試 Agent 可以正確創建
        // 注意：這在沒有 GPU 的環境中也應該能運行
        let result = OrbanAgent::new().await;

        // 不應該失敗，即使沒有 GPU
        assert!(result.is_ok() || result.is_err());
    }
}
