// 沙盒環境 - 隔離執行任務

use crate::error::{Error, Result};
use tracing::{info, warn};
use std::process::Command;

/// 沙盒
pub struct Sandbox {
    use_docker: bool,
}

impl Sandbox {
    /// 創建新的沙盒
    pub fn new() -> Result<Self> {
        // 檢查是否有 Docker
        let use_docker = Command::new("docker")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if use_docker {
            info!("Docker detected, will use containerized execution");
        } else {
            warn!("Docker not found, will use process isolation");
        }

        Ok(Self { use_docker })
    }

    /// 在沙盒中執行任務
    pub fn run_task(
        &self,
        model_path: &str,
        input_path: &str,
        config: &serde_json::Value,
    ) -> Result<String> {
        if self.use_docker {
            self.run_in_docker(model_path, input_path, config)
        } else {
            self.run_in_process(model_path, input_path, config)
        }
    }

    /// 使用 Docker 執行
    fn run_in_docker(
        &self,
        model_path: &str,
        input_path: &str,
        config: &serde_json::Value,
    ) -> Result<String> {
        // TODO: 實現 Docker 容器執行
        // docker run --gpus all -v /models:/models orban/runner python run.py
        info!("Running task in Docker container");
        warn!("Docker execution not yet implemented");

        Ok("/tmp/output.json".to_string())
    }

    /// 使用進程隔離執行
    fn run_in_process(
        &self,
        model_path: &str,
        input_path: &str,
        config: &serde_json::Value,
    ) -> Result<String> {
        // TODO: 實現進程隔離執行
        // 使用 setrlimit 限制資源
        info!("Running task in isolated process");
        warn!("Process execution not yet implemented");

        Ok("/tmp/output.json".to_string())
    }
}

impl Default for Sandbox {
    fn default() -> Self {
        Self::new().expect("Failed to create sandbox")
    }
}
