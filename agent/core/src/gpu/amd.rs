//! AMD GPU 支援
//!
//! 使用 ROCm SMI 與 AMD GPU 互動

use super::GPUDevice;
use crate::{Error, Result, types::GpuType};
use async_trait::async_trait;

/// AMD GPU 裝置
pub struct AmdGPU {
    device_id: u32,
}

impl AmdGPU {
    /// 創建新的 AMD GPU 實例
    pub fn new(device_id: u32) -> Self {
        Self { device_id }
    }
}

#[async_trait]
impl GPUDevice for AmdGPU {
    fn name(&self) -> String {
        // 實際實作需要從 ROCm SMI 查詢
        format!("AMD GPU {}", self.device_id)
    }

    fn gpu_type(&self) -> GpuType {
        GpuType::Amd
    }

    fn total_memory_gb(&self) -> f32 {
        // TODO: 實作 ROCm SMI 查詢
        0.0
    }

    fn used_memory_gb(&self) -> Result<f32> {
        // TODO: 實作 ROCm SMI 查詢
        Ok(0.0)
    }

    fn utilization(&self) -> Result<f32> {
        // TODO: 實作 ROCm SMI 查詢
        Ok(0.0)
    }

    fn temperature(&self) -> Result<f32> {
        // TODO: 實作 ROCm SMI 查詢
        Ok(0.0)
    }

    fn power_usage(&self) -> Result<f32> {
        // TODO: 實作 ROCm SMI 查詢
        Ok(0.0)
    }

    fn compute_capability(&self) -> Option<String> {
        // AMD 使用 gfx 版本
        None
    }

    fn driver_version(&self) -> Option<String> {
        None
    }

    fn hardware_id(&self) -> String {
        format!("amd-{}", self.device_id)
    }

    fn is_available(&self) -> bool {
        true
    }
}
