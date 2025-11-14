// 任務執行引擎

use super::Sandbox;
use crate::gpu::GPUDetector;
use crate::types::{TaskPayload, TaskResult};
use crate::error::{Error, Result};
use tracing::{info, warn};
use std::time::Instant;

/// 任務執行器
pub struct TaskExecutor {
    gpu_detector: GPUDetector,
    sandbox: Sandbox,
}

impl TaskExecutor {
    /// 創建新的任務執行器
    pub fn new(gpu_detector: GPUDetector) -> Result<Self> {
        let sandbox = Sandbox::new()?;

        Ok(Self {
            gpu_detector,
            sandbox,
        })
    }

    /// 執行任務
    pub async fn execute(&self, payload: TaskPayload) -> Result<TaskResult> {
        info!("Starting task execution");
        let start_time = Instant::now();

        // 1. 下載模型和資料
        info!("Downloading model from {}", payload.model_url);
        let model_path = self.download_file(&payload.model_url, &payload.model_hash).await?;

        info!("Downloading input data from {}", payload.input_data_url);
        let input_path = self.download_file(&payload.input_data_url, "").await?;

        // 2. 在沙盒中執行
        info!("Executing task in sandbox");
        let output_path = self.sandbox.run_task(&model_path, &input_path, &payload.config)?;

        // 3. 上傳結果
        info!("Uploading results to {}", payload.output_url);
        let output_hash = self.upload_file(&output_path, &payload.output_url).await?;

        let execution_time = start_time.elapsed();

        info!(
            "Task completed in {:.2} seconds",
            execution_time.as_secs_f64()
        );

        Ok(TaskResult {
            output_url: payload.output_url,
            output_hash,
            execution_time_sec: execution_time.as_secs() as u32,
            gpu_time_sec: execution_time.as_secs() as u32, // TODO: 精確計算 GPU 時間
        })
    }

    /// 下載文件
    async fn download_file(&self, url: &str, expected_hash: &str) -> Result<String> {
        // TODO: 實現文件下載
        // 1. 使用 reqwest 下載
        // 2. 驗證哈希
        // 3. 保存到臨時目錄
        warn!("File download not yet implemented: {}", url);
        Ok("/tmp/model.bin".to_string())
    }

    /// 上傳文件
    async fn upload_file(&self, path: &str, url: &str) -> Result<String> {
        // TODO: 實現文件上傳
        // 1. 讀取文件
        // 2. 計算哈希
        // 3. 上傳到 S3
        warn!("File upload not yet implemented: {}", path);
        Ok("sha256:abc123".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_task_execution() {
        // TODO: 添加任務執行測試
    }
}
