//! NVIDIA GPU 支援
//!
//! 使用 NVML (NVIDIA Management Library) 與 NVIDIA GPU 互動

use super::GPUDevice;
use crate::{Error, Result, types::GpuType};
use nvml_wrapper::Device;
use async_trait::async_trait;

/// NVIDIA GPU 裝置
pub struct NvidiaGPU {
    device: Device<'static>,
}

impl NvidiaGPU {
    /// 創建新的 NVIDIA GPU 實例
    pub fn new(device: Device<'static>) -> Self {
        Self { device }
    }

    /// 取得 UUID（用於硬體 ID）
    fn get_uuid(&self) -> String {
        self.device.uuid()
            .unwrap_or_else(|_| format!("nvidia-unknown-{}", self.device.index().unwrap_or(0)))
    }
}

#[async_trait]
impl GPUDevice for NvidiaGPU {
    fn name(&self) -> String {
        self.device.name()
            .unwrap_or_else(|_| "Unknown NVIDIA GPU".to_string())
    }

    fn gpu_type(&self) -> GpuType {
        GpuType::Nvidia
    }

    fn total_memory_gb(&self) -> f32 {
        self.device.memory_info()
            .map(|info| info.total as f32 / 1024.0 / 1024.0 / 1024.0)
            .unwrap_or(0.0)
    }

    fn used_memory_gb(&self) -> Result<f32> {
        self.device.memory_info()
            .map(|info| info.used as f32 / 1024.0 / 1024.0 / 1024.0)
            .map_err(|e| Error::Gpu(format!("Failed to get memory info: {}", e)))
    }

    fn utilization(&self) -> Result<f32> {
        self.device.utilization_rates()
            .map(|rates| rates.gpu as f32 / 100.0)
            .map_err(|e| Error::Gpu(format!("Failed to get utilization: {}", e)))
    }

    fn temperature(&self) -> Result<f32> {
        use nvml_wrapper::enum_wrappers::device::TemperatureSensor;

        self.device.temperature(TemperatureSensor::Gpu)
            .map(|temp| temp as f32)
            .map_err(|e| Error::Gpu(format!("Failed to get temperature: {}", e)))
    }

    fn power_usage(&self) -> Result<f32> {
        self.device.power_usage()
            .map(|milliwatts| milliwatts as f32 / 1000.0)
            .map_err(|e| Error::Gpu(format!("Failed to get power usage: {}", e)))
    }

    fn compute_capability(&self) -> Option<String> {
        self.device.cuda_compute_capability()
            .ok()
            .map(|cc| format!("{}.{}", cc.major, cc.minor))
    }

    fn driver_version(&self) -> Option<String> {
        // 驅動版本是全域的，不是每個裝置獨立的
        // 但我們可以嘗試從 NVML 取得
        None // 簡化實作
    }

    fn hardware_id(&self) -> String {
        self.get_uuid()
    }

    fn is_available(&self) -> bool {
        // 檢查 GPU 是否可用（沒有錯誤狀態）
        self.utilization().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "gpu-tests")]
    fn test_nvidia_gpu() {
        let nvml = nvml_wrapper::Nvml::init().unwrap();
        let device = nvml.device_by_index(0).unwrap();
        let gpu = NvidiaGPU::new(device);

        println!("Name: {}", gpu.name());
        println!("Memory: {:.1} GB", gpu.total_memory_gb());
        println!("UUID: {}", gpu.hardware_id());

        assert!(!gpu.name().is_empty());
        assert!(gpu.total_memory_gb() > 0.0);
    }
}
