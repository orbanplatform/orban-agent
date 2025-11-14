//! 安全沙盒
//!
//! 在隔離環境中執行任務，防止惡意代碼傷害主機系統
//!
//! 支援的隔離方式：
//! - Docker 容器
//! - Podman（無需 root）
//! - Native 執行（最低安全性）

use crate::{Error, Result, types::Task};
use tracing::{info, warn};

/// 沙盒類型
#[derive(Debug, Clone)]
pub enum SandboxType {
    /// Docker 容器
    #[cfg(feature = "docker")]
    Docker,

    /// Native 執行（無隔離）
    Native,
}

/// 沙盒執行環境
pub struct Sandbox {
    sandbox_type: SandboxType,

    #[cfg(feature = "docker")]
    docker: Option<bollard::Docker>,
}

impl Sandbox {
    /// 創建新的沙盒
    pub fn new() -> Result<Self> {
        #[cfg(feature = "docker")]
        {
            // 嘗試連接 Docker
            match bollard::Docker::connect_with_local_defaults() {
                Ok(docker) => {
                    info!("Using Docker sandbox");
                    return Ok(Self {
                        sandbox_type: SandboxType::Docker,
                        docker: Some(docker),
                    });
                }
                Err(e) => {
                    warn!("Docker not available: {}", e);
                }
            }
        }

        // 降級到 Native 執行
        warn!("Using Native sandbox (less secure)");
        Ok(Self {
            sandbox_type: SandboxType::Native,
            #[cfg(feature = "docker")]
            docker: None,
        })
    }

    /// 執行推論任務
    pub async fn run_inference(&self, task: &Task) -> Result<()> {
        match self.sandbox_type {
            #[cfg(feature = "docker")]
            SandboxType::Docker => {
                self.run_in_docker(task, "inference").await
            }
            SandboxType::Native => {
                self.run_native(task, "inference").await
            }
        }
    }

    /// 執行訓練任務
    pub async fn run_training(&self, task: &Task) -> Result<()> {
        match self.sandbox_type {
            #[cfg(feature = "docker")]
            SandboxType::Docker => {
                self.run_in_docker(task, "training").await
            }
            SandboxType::Native => {
                self.run_native(task, "training").await
            }
        }
    }

    /// 在 Docker 中執行
    #[cfg(feature = "docker")]
    async fn run_in_docker(&self, task: &Task, task_type: &str) -> Result<()> {
        use bollard::container::{Config, CreateContainerOptions};
        use bollard::models::HostConfig;

        let docker = self.docker.as_ref()
            .ok_or_else(|| Error::Other(anyhow::anyhow!("Docker not available")))?;

        // 創建容器配置
        let config = Config {
            image: Some("orban-agent-runtime:latest"),
            cmd: Some(vec![task_type, &task.id]),
            host_config: Some(HostConfig {
                // GPU 訪問
                device_requests: Some(vec![bollard::models::DeviceRequest {
                    driver: Some("nvidia".to_string()),
                    count: Some(-1), // 所有 GPU
                    device_ids: None,
                    capabilities: Some(vec![vec!["gpu".to_string()]]),
                    options: None,
                }]),
                // 資源限制
                memory: Some(16 * 1024 * 1024 * 1024), // 16GB
                cpu_quota: Some(100000),
                ..Default::default()
            }),
            ..Default::default()
        };

        // 創建並啟動容器
        let container_name = format!("orban-task-{}", task.id);

        info!("Creating Docker container: {}", container_name);

        let _container = docker.create_container(
            Some(CreateContainerOptions {
                name: &container_name,
                platform: None,
            }),
            config,
        ).await.map_err(|e| Error::Container(e))?;

        // TODO: 啟動容器並等待完成

        Ok(())
    }

    /// Native 執行（直接在主機上運行）
    async fn run_native(&self, task: &Task, task_type: &str) -> Result<()> {
        info!("Running task natively: {} ({})", task.id, task_type);

        // TODO: 實作 native 執行邏輯
        // 警告：這不安全，應該只用於測試或信任的環境

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let result = Sandbox::new();
        assert!(result.is_ok());
    }
}
