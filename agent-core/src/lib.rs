// Orban Agent Core Library
//
// 這是 Orban Agent 的核心引擎，負責：
// - GPU 偵測與監控
// - 與 Orban Platform 通訊 (Orban Protocol)
// - 任務執行與沙盒隔離
// - 收益追蹤與統計

pub mod gpu;
pub mod network;
pub mod compute;
pub mod earnings;
pub mod types;
pub mod error;
pub mod config;
pub mod daemon;
pub mod cli;

// 重新導出常用類型
pub use error::{Error, Result};
pub use types::*;

use tracing::{info, warn, error};
use tokio::sync::mpsc;
use std::sync::Arc;

/// Orban Agent 主控制器
pub struct OrbanAgent {
    config: AgentConfig,
    gpu_detector: gpu::GPUDetector,
    network_client: network::OrbanClient,
    task_executor: compute::TaskExecutor,
    earnings_tracker: earnings::EarningsTracker,
}

impl OrbanAgent {
    /// 創建新的 Agent 實例
    pub async fn new(config: AgentConfig) -> Result<Self> {
        info!("Initializing Orban Agent v{}", env!("CARGO_PKG_VERSION"));

        // 偵測 GPU 硬體
        let gpu_detector = gpu::GPUDetector::detect_all()?;
        info!("Detected {} GPU(s)", gpu_detector.device_count());

        // 創建網路客戶端
        let network_client = network::OrbanClient::new(&config).await?;

        // 創建任務執行器
        let devices: Vec<_> = gpu_detector.get_all_devices().to_vec();
        let task_executor = compute::TaskExecutor::new(devices)?;

        // 創建收益追蹤器
        let earnings_tracker = earnings::EarningsTracker::new()?;

        Ok(Self {
            config,
            gpu_detector,
            network_client,
            task_executor,
            earnings_tracker,
        })
    }

    /// 啟動 Agent
    pub async fn start(&mut self) -> Result<()> {
        info!("Starting Orban Agent...");

        // 連接到 Orban Platform
        self.network_client.connect().await?;

        // 註冊 Agent
        self.register_agent().await?;

        // 啟動事件循環
        self.run_event_loop().await?;

        Ok(())
    }

    /// 註冊 Agent 到平台
    async fn register_agent(&mut self) -> Result<()> {
        info!("Registering agent with platform...");

        let hardware_info = self.gpu_detector.get_hardware_info();
        let capabilities = self.get_capabilities();
        let location = self.get_location();
        let availability = self.config.availability.clone();

        self.network_client
            .register(hardware_info, capabilities, location, availability)
            .await?;

        info!("Agent registered successfully");
        Ok(())
    }

    /// 主事件循環
    async fn run_event_loop(&mut self) -> Result<()> {
        info!("Entering event loop...");

        let (tx, mut rx) = mpsc::channel(100);
        let network_client = self.network_client.clone();

        // 心跳任務
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
                // TODO: 發送心跳
            }
        });

        // 接收任務
        loop {
            tokio::select! {
                Some(msg) = self.network_client.receive() => {
                    self.handle_message(msg).await?;
                }
                Some(event) = rx.recv() => {
                    self.handle_event(event).await?;
                }
            }
        }
    }

    /// 處理來自平台的訊息
    async fn handle_message(&mut self, msg: network::Message) -> Result<()> {
        use network::{MessageType, MessagePayload};

        match msg.message_type {
            MessageType::TaskAssign => {
                info!("Received task assignment");
                if let MessagePayload::TaskAssign(payload) = msg.payload {
                    self.handle_task_assign(payload).await?;
                }
            }
            MessageType::PowChallenge => {
                info!("Received PoW challenge");
                if let MessagePayload::PowChallenge(payload) = msg.payload {
                    self.handle_pow_challenge(payload).await?;
                }
            }
            MessageType::EarningsRecord => {
                info!("Received earnings record");
                if let MessagePayload::EarningsRecord(payload) = msg.payload {
                    self.handle_earnings_record(payload).await?;
                }
            }
            _ => {
                warn!("Unknown message type: {:?}", msg.message_type);
            }
        }

        Ok(())
    }

    /// 處理任務分配
    async fn handle_task_assign(&mut self, payload: network::TaskAssignPayload) -> Result<()> {
        // 檢查是否有足夠的資源
        if !self.can_accept_task(&payload.requirements) {
            // TODO: Implement reject_task
            // self.network_client.reject_task(&payload.task_id, "insufficient_resources").await?;
            return Ok(());
        }

        // 接受任務
        // TODO: Implement accept_task
        // self.network_client.accept_task(&payload.task_id).await?;

        // 執行任務
        // TODO: Convert payload to Task
        // let result = self.task_executor.execute(task).await?;

        // 上報完成
        // TODO: Implement complete_task
        // self.network_client.complete_task(&payload.task_id, result).await?;

        Ok(())
    }

    /// 處理工作證明挑戰
    async fn handle_pow_challenge(&mut self, payload: network::PowChallengePayload) -> Result<()> {
        let challenge = payload.nonce.as_bytes();
        let _response = self.gpu_detector.compute_pow(challenge)?;
        // TODO: Implement send_pow_response
        // self.network_client.send_pow_response(response).await?;
        Ok(())
    }

    /// 處理收益記錄
    async fn handle_earnings_record(&mut self, payload: network::EarningsRecordPayload) -> Result<()> {
        self.earnings_tracker.record_earnings(payload.earnings).await?;
        Ok(())
    }

    /// 處理內部事件
    async fn handle_event(&mut self, event: AgentEvent) -> Result<()> {
        // TODO: 實現事件處理
        Ok(())
    }

    /// 檢查是否可以接受任務
    fn can_accept_task(&self, requirements: &TaskRequirements) -> bool {
        self.gpu_detector.meets_requirements(requirements)
    }

    /// 獲取 Agent 能力
    fn get_capabilities(&self) -> Capabilities {
        Capabilities {
            supported_frameworks: vec![
                "pytorch".to_string(),
                "tensorflow".to_string(),
                "onnx".to_string(),
            ],
            max_batch_size: 32,
            fp16_support: true,
            int8_support: true,
        }
    }

    /// 獲取地理位置資訊
    fn get_location(&self) -> Location {
        // TODO: 實際偵測地理位置
        Location {
            country: "TW".to_string(),
            region: "asia-east1".to_string(),
            latency_to_platform_ms: 0,
        }
    }

    /// 停止 Agent
    pub async fn stop(&mut self) -> Result<()> {
        info!("Stopping Orban Agent...");
        self.network_client.disconnect().await?;
        Ok(())
    }
}

/// Agent 配置
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AgentConfig {
    pub agent_id: String,
    pub platform_url: String,
    pub private_key_path: String,
    pub availability: Availability,
}

/// Agent 事件
#[derive(Debug)]
pub enum AgentEvent {
    TaskCompleted(String),
    TaskFailed(String, String),
    GPUError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_creation() {
        // TODO: 添加測試
    }
}
