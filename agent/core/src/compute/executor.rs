//! 任務執行器
//!
//! 負責協調整個任務執行流程

use super::{Sandbox, ProofGenerator};
use crate::{
    Error, Result,
    gpu::GPUDevice,
    types::{Task, TaskResult, TaskStatus, TaskType, ProofOfWork},
};
use std::sync::Arc;
use chrono::Utc;
use tracing::{info, warn};

/// 任務執行器
pub struct TaskExecutor {
    /// 可用的 GPU 裝置
    gpus: Vec<Arc<dyn GPUDevice>>,

    /// 沙盒執行環境
    sandbox: Sandbox,

    /// 工作證明生成器
    proof_generator: ProofGenerator,
}

impl TaskExecutor {
    /// 創建新的任務執行器
    pub fn new(gpus: Vec<Arc<dyn GPUDevice>>) -> Result<Self> {
        Ok(Self {
            gpus,
            sandbox: Sandbox::new()?,
            proof_generator: ProofGenerator::new(),
        })
    }

    /// 執行任務
    ///
    /// 這是主要的任務執行流程
    pub async fn execute(&mut self, task: Task) -> Result<TaskResult> {
        info!("Starting task execution: {}", task.id);

        let started_at = Utc::now();

        // 1. 選擇合適的 GPU
        let gpu = self.select_gpu(&task)?;
        info!("Selected GPU: {}", gpu.name());

        // 2. 驗證任務要求
        self.validate_task(&task, &gpu)?;

        // 3. 下載資源（模型、資料集）
        info!("Downloading resources...");
        self.download_resources(&task).await?;

        // 4. 執行任務
        info!("Executing task...");
        let result_data = match task.task_type {
            TaskType::Inference => {
                self.execute_inference(&task, &gpu).await?
            }
            TaskType::Training => {
                self.execute_training(&task, &gpu).await?
            }
            TaskType::FineTuning => {
                self.execute_finetuning(&task, &gpu).await?
            }
        };

        let completed_at = Utc::now();
        let gpu_time_seconds = (completed_at - started_at).num_milliseconds() as f64 / 1000.0;

        // 5. 生成工作證明
        info!("Generating proof of work...");
        let proof = self.proof_generator.generate(
            &task,
            &result_data,
            &gpu,
            gpu_time_seconds
        )?;

        // 6. 清理資源
        self.cleanup(&task).await?;

        info!("Task completed successfully: {}", task.id);

        Ok(TaskResult {
            task_id: task.id.clone(),
            agent_id: "temp-agent-id".to_string(), // 會由外部填入
            status: TaskStatus::Completed,
            started_at,
            completed_at,
            gpu_time_seconds,
            gpu_used: gpu.to_info(),
            result_data: Some(result_data),
            proof,
            error: None,
        })
    }

    /// 選擇合適的 GPU
    fn select_gpu(&self, task: &Task) -> Result<Arc<dyn GPUDevice>> {
        if self.gpus.is_empty() {
            return Err(Error::NoGpuFound);
        }

        // 簡單策略：選擇第一個可用且符合需求的 GPU
        for gpu in &self.gpus {
            // 檢查 VRAM 是否足夠
            if gpu.total_memory_gb() >= task.requirements.vram_gb {
                // 檢查 GPU 類型偏好
                if let Some(preferred_type) = &task.requirements.preferred_gpu_type {
                    if &gpu.gpu_type() == preferred_type {
                        return Ok(gpu.clone());
                    }
                } else {
                    return Ok(gpu.clone());
                }
            }
        }

        // 如果沒有完全符合的，選第一個
        Ok(self.gpus[0].clone())
    }

    /// 驗證任務要求
    fn validate_task(&self, task: &Task, gpu: &Arc<dyn GPUDevice>) -> Result<()> {
        // 檢查 VRAM 是否足夠
        if gpu.total_memory_gb() < task.requirements.vram_gb {
            return Err(Error::InvalidTask(
                format!("Insufficient VRAM: required {:.1} GB, available {:.1} GB",
                    task.requirements.vram_gb,
                    gpu.total_memory_gb()
                )
            ));
        }

        // 檢查計算能力
        if let Some(min_cc) = &task.requirements.min_compute_capability {
            if let Some(cc) = gpu.compute_capability() {
                // 簡單的字串比較（實際應該做版本比較）
                if cc < *min_cc {
                    return Err(Error::InvalidTask(
                        format!("Insufficient compute capability: required {}, available {}",
                            min_cc, cc
                        )
                    ));
                }
            }
        }

        Ok(())
    }

    /// 下載資源
    async fn download_resources(&self, task: &Task) -> Result<()> {
        // TODO: 實作模型和資料集下載
        // 應該包括：
        // 1. 檢查本地快取
        // 2. 下載（支援斷點續傳）
        // 3. 驗證校驗和
        // 4. 解壓縮

        info!("Model: {} ({})", task.model.name, task.model.version);

        if let Some(dataset) = &task.dataset {
            info!("Dataset: {}", dataset.name);
        }

        Ok(())
    }

    /// 執行推論任務
    async fn execute_inference(
        &self,
        task: &Task,
        gpu: &Arc<dyn GPUDevice>,
    ) -> Result<crate::types::ResultData> {
        info!("Running inference on {}", gpu.name());

        // 在沙盒中執行
        self.sandbox.run_inference(task).await?;

        // TODO: 實際的推論邏輯
        // 應該包括：
        // 1. 載入模型
        // 2. 預處理輸入
        // 3. 執行推論
        // 4. 後處理輸出

        Ok(crate::types::ResultData::Inline {
            data: serde_json::json!({
                "status": "completed",
                "predictions": []
            }).to_string(),
        })
    }

    /// 執行訓練任務
    async fn execute_training(
        &self,
        task: &Task,
        gpu: &Arc<dyn GPUDevice>,
    ) -> Result<crate::types::ResultData> {
        info!("Running training on {}", gpu.name());

        // TODO: 實作訓練邏輯
        self.sandbox.run_training(task).await?;

        Ok(crate::types::ResultData::Inline {
            data: serde_json::json!({
                "status": "completed",
                "metrics": {
                    "loss": 0.123,
                    "accuracy": 0.95
                }
            }).to_string(),
        })
    }

    /// 執行微調任務
    async fn execute_finetuning(
        &self,
        task: &Task,
        gpu: &Arc<dyn GPUDevice>,
    ) -> Result<crate::types::ResultData> {
        info!("Running fine-tuning on {}", gpu.name());

        // TODO: 實作微調邏輯
        self.sandbox.run_training(task).await?;

        Ok(crate::types::ResultData::Inline {
            data: serde_json::json!({
                "status": "completed",
                "checkpoint": "checkpoint-1000"
            }).to_string(),
        })
    }

    /// 清理資源
    async fn cleanup(&self, task: &Task) -> Result<()> {
        info!("Cleaning up resources for task: {}", task.id);

        // TODO: 實作清理邏輯
        // 1. 釋放 GPU 記憶體
        // 2. 刪除臨時檔案
        // 3. 清理容器

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let gpus: Vec<Arc<dyn GPUDevice>> = Vec::new();
        let result = TaskExecutor::new(gpus);
        assert!(result.is_ok());
    }
}
