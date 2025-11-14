//! Apple GPU 支援
//!
//! 使用 Metal 框架與 Apple Silicon GPU 互動

use super::GPUDevice;
use crate::{Error, Result, types::GpuType};
use async_trait::async_trait;

/// Apple GPU 裝置
pub struct AppleGPU {
    name: String,
}

impl AppleGPU {
    /// 創建新的 Apple GPU 實例
    pub fn new() -> Result<Self> {
        // TODO: 使用 Metal 框架檢測 GPU
        Ok(Self {
            name: "Apple GPU".to_string(),
        })
    }
}

#[async_trait]
impl GPUDevice for AppleGPU {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn gpu_type(&self) -> GpuType {
        GpuType::Apple
    }

    fn total_memory_gb(&self) -> f32 {
        // Apple Silicon 使用統一記憶體
        // TODO: 從系統查詢
        0.0
    }

    fn used_memory_gb(&self) -> Result<f32> {
        // TODO: 實作
        Ok(0.0)
    }

    fn utilization(&self) -> Result<f32> {
        // TODO: 使用 Metal Performance HUD 或系統 API
        Ok(0.0)
    }

    fn temperature(&self) -> Result<f32> {
        // Apple 不直接暴露 GPU 溫度
        Ok(0.0)
    }

    fn power_usage(&self) -> Result<f32> {
        // TODO: 使用 IOKit 或 powermetrics
        Ok(0.0)
    }

    fn compute_capability(&self) -> Option<String> {
        // Apple 使用不同的術語
        Some("Metal".to_string())
    }

    fn driver_version(&self) -> Option<String> {
        None
    }

    fn hardware_id(&self) -> String {
        // TODO: 取得真實的硬體序號
        "apple-gpu-0".to_string()
    }

    fn is_available(&self) -> bool {
        true
    }
}
