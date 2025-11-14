use super::device::GPUDevice;
use crate::types::{GPUVendor, MemoryInfo};
use crate::error::{Error, Result};

/// AMD GPU 設備
///
/// TODO: 實現 AMD ROCm 支援
pub struct AmdGPU {
    index: u32,
}

impl AmdGPU {
    pub fn new(index: u32) -> Self {
        Self { index }
    }
}

impl GPUDevice for AmdGPU {
    fn index(&self) -> u32 {
        self.index
    }

    fn vendor(&self) -> GPUVendor {
        GPUVendor::AMD
    }

    fn name(&self) -> Result<String> {
        // TODO: 使用 ROCm SMI 獲取 GPU 名稱
        Err(Error::GPUError("AMD GPU support not implemented".to_string()))
    }

    fn memory_info(&self) -> Result<MemoryInfo> {
        // TODO: 實現記憶體資訊查詢
        Err(Error::GPUError("AMD GPU support not implemented".to_string()))
    }

    fn utilization(&self) -> Result<f32> {
        // TODO: 實現使用率查詢
        Err(Error::GPUError("AMD GPU support not implemented".to_string()))
    }

    fn temperature(&self) -> Result<f32> {
        // TODO: 實現溫度查詢
        Err(Error::GPUError("AMD GPU support not implemented".to_string()))
    }

    fn power_usage(&self) -> Result<f32> {
        // TODO: 實現功耗查詢
        Err(Error::GPUError("AMD GPU support not implemented".to_string()))
    }

    fn fan_speed(&self) -> Result<f32> {
        // TODO: 實現風扇速度查詢
        Err(Error::GPUError("AMD GPU support not implemented".to_string()))
    }

    fn compute_capability(&self) -> Result<String> {
        // TODO: 返回 gfx 版本
        Err(Error::GPUError("AMD GPU support not implemented".to_string()))
    }

    fn pcie_bandwidth(&self) -> Result<u32> {
        // TODO: 實現 PCIe 頻寬查詢
        Err(Error::GPUError("AMD GPU support not implemented".to_string()))
    }

    fn uuid(&self) -> Result<String> {
        // TODO: 返回設備唯一 ID
        Err(Error::GPUError("AMD GPU support not implemented".to_string()))
    }

    fn compute_pow(&self, _challenge: &[u8], _difficulty: u32) -> Result<Vec<u8>> {
        // TODO: 使用 ROCm 實現 PoW
        Err(Error::GPUError("AMD GPU support not implemented".to_string()))
    }
}
