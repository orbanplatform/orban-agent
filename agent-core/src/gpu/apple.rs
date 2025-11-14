use super::device::GPUDevice;
use crate::types::{GPUVendor, MemoryInfo};
use crate::error::{Error, Result};

/// Apple GPU 設備 (Apple Silicon)
///
/// TODO: 實現 Metal 支援
pub struct AppleGPU {
    index: u32,
}

impl AppleGPU {
    pub fn new(index: u32) -> Self {
        Self { index }
    }
}

impl GPUDevice for AppleGPU {
    fn index(&self) -> u32 {
        self.index
    }

    fn vendor(&self) -> GPUVendor {
        GPUVendor::Apple
    }

    fn name(&self) -> Result<String> {
        // TODO: 使用 Metal API 獲取 GPU 名稱
        Err(Error::GPUError("Apple GPU support not implemented".to_string()))
    }

    fn memory_info(&self) -> Result<MemoryInfo> {
        // TODO: 實現記憶體資訊查詢 (Metal shared memory)
        Err(Error::GPUError("Apple GPU support not implemented".to_string()))
    }

    fn utilization(&self) -> Result<f32> {
        // TODO: 實現使用率查詢
        Err(Error::GPUError("Apple GPU support not implemented".to_string()))
    }

    fn temperature(&self) -> Result<f32> {
        // TODO: 使用 IOKit 查詢溫度
        Err(Error::GPUError("Apple GPU support not implemented".to_string()))
    }

    fn power_usage(&self) -> Result<f32> {
        // TODO: 實現功耗查詢
        Err(Error::GPUError("Apple GPU support not implemented".to_string()))
    }

    fn fan_speed(&self) -> Result<f32> {
        // Apple Silicon 通常沒有風扇或不可查詢
        Ok(0.0)
    }

    fn compute_capability(&self) -> Result<String> {
        // TODO: 返回 Metal 版本
        Err(Error::GPUError("Apple GPU support not implemented".to_string()))
    }

    fn pcie_bandwidth(&self) -> Result<u32> {
        // Apple Silicon 使用統一記憶體架構，沒有 PCIe
        Ok(0)
    }

    fn uuid(&self) -> Result<String> {
        // TODO: 返回設備唯一 ID
        Err(Error::GPUError("Apple GPU support not implemented".to_string()))
    }

    fn compute_pow(&self, _challenge: &[u8], _difficulty: u32) -> Result<Vec<u8>> {
        // TODO: 使用 Metal Compute 實現 PoW
        Err(Error::GPUError("Apple GPU support not implemented".to_string()))
    }
}
