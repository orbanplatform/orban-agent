//! 简化的网络客户端 - 适配 lib.rs 使用

use crate::types::*;
use crate::error::{Error, Result};
use tracing::{info, warn};

/// 简化的网络客户端
#[derive(Clone)]
pub struct Client {
    base_url: String,
    client: reqwest::Client,
}

impl Client {
    /// 创建新客户端
    pub fn new(base_url: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| Error::HTTPError(e))?;

        Ok(Self { base_url, client })
    }

    /// 注册到平台
    pub async fn register(&self, req: RegistrationRequest) -> Result<String> {
        info!("Registering agent to platform...");

        // 实际环境：发送 HTTP POST 到平台
        // let url = format!("{}/api/v1/agents/register", self.base_url);
        // let response = self.client.post(&url)
        //     .json(&req)
        //     .send()
        //     .await?;

        // 暂时：模拟注册成功
        let agent_id = format!("agent-{}", uuid::Uuid::new_v4());
        warn!("Platform not available - using mock agent ID: {}", agent_id);

        Ok(agent_id)
    }

    /// 发送心跳
    pub async fn heartbeat(&self, agent_id: &str) -> Result<()> {
        // 实际环境：发送心跳到平台
        // let url = format!("{}/api/v1/agents/{}/heartbeat", self.base_url, agent_id);
        // self.client.post(&url).send().await?;

        // 暂时：模拟成功
        Ok(())
    }

    /// 领取任务
    pub async fn fetch_task(&self, agent_id: &str) -> Result<Option<Task>> {
        // 实际环境：从平台获取任务
        // let url = format!("{}/api/v1/agents/{}/tasks/fetch", self.base_url, agent_id);
        // let response = self.client.get(&url).send().await?;

        // 暂时：返回 None (无任务)
        Ok(None)
    }

    /// 提交任务结果
    pub async fn submit_result(&self, result: TaskResult) -> Result<()> {
        // 实际环境：提交结果到平台
        // let url = format!("{}/api/v1/tasks/{}/result", self.base_url, result.task_id);
        // self.client.post(&url).json(&result).send().await?;

        info!("Task result submitted (mock)");
        Ok(())
    }
}

/// 注册请求
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistrationRequest {
    pub hostname: String,
    pub gpus: Vec<GpuInfo>,
    pub version: String,
}

/// GPU 信息（简化版）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GpuInfo {
    pub model: String,
    pub gpu_type: GpuType,
    pub total_memory_gb: f32,
    pub compute_capability: Option<String>,
    pub driver_version: Option<String>,
    pub hardware_id: String,
}

/// GPU 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GpuType {
    Nvidia,
    Amd,
    Apple,
    Intel,
}

/// 任务定义（简化版）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
    pub id: String,
    pub gpu_required: String,
    pub estimated_duration_secs: u64,
}

/// 任务结果（简化版）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub gpu_used: GpuInfo,
    pub gpu_time_seconds: f64,
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = Client::new("https://platform.orban.ai".to_string());
        assert!(client.is_ok());
    }
}
