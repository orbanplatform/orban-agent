//! GPU 偵測器
//!
//! 自動偵測系統中所有可用的 GPU

use super::GPUDevice;
use crate::{Error, Result};
use std::sync::Arc;

/// GPU 偵測器
///
/// 負責在系統啟動時偵測所有可用的 GPU
pub struct GPUDetector {
    devices: Vec<Arc<dyn GPUDevice>>,
}

impl GPUDetector {
    /// 創建新的 GPU 偵測器
    pub fn new() -> Result<Self> {
        Ok(Self {
            devices: Vec::new(),
        })
    }

    /// 偵測所有 GPU
    ///
    /// 這會嘗試偵測系統中所有支援的 GPU 類型
    pub fn detect_all(&self) -> Result<Vec<Arc<dyn GPUDevice>>> {
        let mut devices: Vec<Arc<dyn GPUDevice>> = Vec::new();

        // 偵測 NVIDIA GPU
        #[cfg(feature = "nvidia")]
        {
            match self.detect_nvidia() {
                Ok(mut nvidia_devices) => {
                    devices.append(&mut nvidia_devices);
                }
                Err(e) => {
                    tracing::debug!("No NVIDIA GPUs found: {}", e);
                }
            }
        }

        // 偵測 AMD GPU
        #[cfg(feature = "amd")]
        {
            match self.detect_amd() {
                Ok(mut amd_devices) => {
                    devices.append(&mut amd_devices);
                }
                Err(e) => {
                    tracing::debug!("No AMD GPUs found: {}", e);
                }
            }
        }

        // 偵測 Apple GPU
        #[cfg(all(target_os = "macos", feature = "apple"))]
        {
            match self.detect_apple() {
                Ok(mut apple_devices) => {
                    devices.append(&mut apple_devices);
                }
                Err(e) => {
                    tracing::debug!("No Apple GPUs found: {}", e);
                }
            }
        }

        if devices.is_empty() {
            return Err(Error::NoGpuFound);
        }

        Ok(devices)
    }

    /// 偵測 NVIDIA GPU
    #[cfg(feature = "nvidia")]
    fn detect_nvidia(&self) -> Result<Vec<Arc<dyn GPUDevice>>> {
        use super::nvidia::NvidiaGPU;

        let nvml = nvml_wrapper::Nvml::init()
            .map_err(|e| Error::Gpu(format!("Failed to initialize NVML: {}", e)))?;

        let device_count = nvml.device_count()
            .map_err(|e| Error::Gpu(format!("Failed to get device count: {}", e)))?;

        let mut devices = Vec::new();

        for i in 0..device_count {
            match nvml.device_by_index(i) {
                Ok(device) => {
                    devices.push(Arc::new(NvidiaGPU::new(device)) as Arc<dyn GPUDevice>);
                }
                Err(e) => {
                    tracing::warn!("Failed to get NVIDIA device {}: {}", i, e);
                }
            }
        }

        Ok(devices)
    }

    /// 偵測 AMD GPU
    #[cfg(feature = "amd")]
    fn detect_amd(&self) -> Result<Vec<Arc<dyn GPUDevice>>> {
        use super::amd::AmdGPU;

        // AMD ROCm SMI 初始化
        // 注意：這需要 AMD 驅動和 ROCm 支援
        let rocm = rocm_smi::RocmSmi::init()
            .map_err(|e| Error::Gpu(format!("Failed to initialize ROCm SMI: {}", e)))?;

        let mut devices = Vec::new();

        for device_id in rocm.get_device_ids() {
            devices.push(Arc::new(AmdGPU::new(device_id)) as Arc<dyn GPUDevice>);
        }

        Ok(devices)
    }

    /// 偵測 Apple GPU
    #[cfg(all(target_os = "macos", feature = "apple"))]
    fn detect_apple(&self) -> Result<Vec<Arc<dyn GPUDevice>>> {
        use super::apple::AppleGPU;

        // Apple Metal 偵測
        let device = AppleGPU::new()?;
        Ok(vec![Arc::new(device) as Arc<dyn GPUDevice>])
    }

    /// 取得所有 GPU 的狀態
    pub fn get_all_status(&self) -> Result<Vec<super::GPUStatus>> {
        let devices = self.detect_all()?;
        let mut statuses = Vec::new();

        for device in devices {
            match device.get_status() {
                Ok(status) => statuses.push(status),
                Err(e) => {
                    tracing::warn!("Failed to get status for {}: {}", device.name(), e);
                }
            }
        }

        Ok(statuses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = GPUDetector::new();
        assert!(detector.is_ok());
    }

    #[test]
    #[cfg(feature = "gpu-tests")]
    fn test_gpu_detection() {
        let detector = GPUDetector::new().unwrap();
        let devices = detector.detect_all();

        // 這個測試需要實際的 GPU
        if devices.is_ok() {
            let devices = devices.unwrap();
            assert!(!devices.is_empty());

            for device in devices {
                println!("Found: {} ({:.1} GB)",
                    device.name(),
                    device.total_memory_gb()
                );
            }
        }
    }
}
