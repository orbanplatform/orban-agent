//! 網路客戶端
//!
//! 與 Orban Platform 的 HTTP/WebSocket 通訊

use super::protocol::*;
use crate::{Result, Error, types::{Task, TaskResult}};
use tracing::{info, debug};

/// Orban Platform 客戶端
pub struct Client {
    base_url: String,
    http_client: reqwest::Client,
}

impl Client {
    /// 創建新的客戶端
    pub fn new(base_url: String) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            base_url,
            http_client,
        })
    }

    /// 註冊 Agent
    pub async fn register(&self, req: RegistrationRequest) -> Result<String> {
        info!("Registering agent to platform...");

        let url = format!("{}/api/v1/agents/register", self.base_url);

        let response = self.http_client
            .post(&url)
            .json(&req)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Network(
                response.error_for_status().unwrap_err()
            ));
        }

        let resp: RegistrationResponse = response.json().await?;

        info!("Agent registered: {}", resp.agent_id);

        Ok(resp.agent_id)
    }

    /// 發送心跳
    pub async fn heartbeat(&self, agent_id: &str) -> Result<()> {
        debug!("Sending heartbeat...");

        let url = format!("{}/api/v1/agents/{}/heartbeat", self.base_url, agent_id);

        let response = self.http_client
            .post(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Network(
                response.error_for_status().unwrap_err()
            ));
        }

        Ok(())
    }

    /// 領取任務
    pub async fn fetch_task(&self, agent_id: &str) -> Result<Option<Task>> {
        debug!("Fetching task...");

        let url = format!("{}/api/v1/agents/{}/tasks/fetch", self.base_url, agent_id);

        let response = self.http_client
            .get(&url)
            .send()
            .await?;

        if response.status().as_u16() == 404 {
            // 沒有可用任務
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(Error::Network(
                response.error_for_status().unwrap_err()
            ));
        }

        let task: Task = response.json().await?;

        Ok(Some(task))
    }

    /// 提交結果
    pub async fn submit_result(&self, result: TaskResult) -> Result<()> {
        info!("Submitting result for task: {}", result.task_id);

        let url = format!("{}/api/v1/tasks/{}/result", self.base_url, result.task_id);

        let response = self.http_client
            .post(&url)
            .json(&result)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Network(
                response.error_for_status().unwrap_err()
            ));
        }

        info!("Result submitted successfully");

        Ok(())
    }

    /// 取得收益資訊
    pub async fn get_earnings(&self, agent_id: &str) -> Result<EarningsResponse> {
        let url = format!("{}/api/v1/agents/{}/earnings", self.base_url, agent_id);

        let response = self.http_client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Network(
                response.error_for_status().unwrap_err()
            ));
        }

        let earnings: EarningsResponse = response.json().await?;

        Ok(earnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Client::new("https://api.orban.ai".to_string());
        assert!(client.is_ok());
    }
}
