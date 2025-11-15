//! 简化的任务执行器 - 适配 lib.rs 使用

use crate::error::{Error, Result};
use crate::gpu::GPUDevice;
use crate::network::{Task, TaskResult, GpuInfo, GpuType};
use crate::types::GPUVendor;
use std::sync::Arc;
use std::time::Instant;
use tracing::{info, warn};

/// 简化的任务执行器
pub struct TaskExecutor {
    devices: Vec<Arc<dyn GPUDevice>>,
}

impl TaskExecutor {
    /// 创建新的任务执行器
    pub fn new(devices: Vec<Arc<dyn GPUDevice>>) -> Result<Self> {
        Ok(Self { devices })
    }

    /// 执行任务
    pub async fn execute(&mut self, task: Task) -> Result<TaskResult> {
        info!("🔧 Executing task: {}", task.id);
        let start_time = Instant::now();

        // 选择一个可用的 GPU
        let device = self.select_gpu(&task)?;

        // 模拟任务执行
        info!("  ├─ Selected GPU: {}", device.name().unwrap_or_else(|_| "Unknown GPU".to_string()));
        info!("  ├─ Estimated duration: {}s", task.estimated_duration_secs);

        // 模拟工作（实际环境会执行真实的 AI 推理）
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let gpu_time_seconds = start_time.elapsed().as_secs_f64();

        info!("  └─ ✓ Task completed in {:.2}s", gpu_time_seconds);

        // 獲取 GPU 信息
        let gpu_name = device.name().unwrap_or_else(|_| "Unknown GPU".to_string());
        let gpu_vendor = device.vendor();
        let gpu_type = match gpu_vendor {
            GPUVendor::NVIDIA => GpuType::Nvidia,
            GPUVendor::AMD => GpuType::Amd,
            GPUVendor::Apple => GpuType::Apple,
            GPUVendor::Intel => GpuType::Intel,
        };

        let gpu_info = GpuInfo {
            model: gpu_name,
            gpu_type,
            total_memory_gb: device.memory_info().map(|m| m.total_gb()).unwrap_or(0.0),
            compute_capability: device.compute_capability().ok(),
            driver_version: None,
            hardware_id: format!("gpu-{}", device.index()),
        };

        Ok(TaskResult {
            task_id: task.id.clone(),
            gpu_used: gpu_info,
            gpu_time_seconds,
            completed_at: chrono::Utc::now(),
        })
    }

    /// 选择 GPU
    fn select_gpu(&self, task: &Task) -> Result<Arc<dyn GPUDevice>> {
        // 简单策略：选择第一个可用的 GPU
        self.devices
            .first()
            .cloned()
            .ok_or_else(|| Error::GPUError("No GPU available".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_creation() {
        let devices: Vec<Arc<dyn GPUDevice>> = vec![];
        let executor = TaskExecutor::new(devices);
        assert!(executor.is_ok());
    }
}
