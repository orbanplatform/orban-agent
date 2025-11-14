use super::device::{GPUDevice, GPUDeviceRef};
use crate::types::{HardwareInfo, GPUInfo, GPUStatus, CPUInfo, TaskRequirements};
use crate::error::{Error, Result};
use std::sync::Arc;
use tracing::{info, warn};
use sysinfo::System;

/// GPU 偵測器
///
/// 自動偵測系統中所有可用的 GPU 設備
#[derive(Clone)]
pub struct GPUDetector {
    devices: Vec<GPUDeviceRef>,
    system_info: Arc<System>,
}

impl GPUDetector {
    /// 偵測所有可用的 GPU
    pub fn detect_all() -> Result<Self> {
        info!("Detecting GPU devices...");
        let mut devices: Vec<GPUDeviceRef> = Vec::new();

        // 偵測 NVIDIA GPU
        #[cfg(feature = "nvidia")]
        {
            match Self::detect_nvidia() {
                Ok(nvidia_devices) => {
                    info!("Found {} NVIDIA GPU(s)", nvidia_devices.len());
                    devices.extend(nvidia_devices);
                }
                Err(e) => {
                    warn!("Failed to detect NVIDIA GPUs: {}", e);
                }
            }
        }

        // 偵測 AMD GPU
        #[cfg(feature = "amd")]
        {
            match Self::detect_amd() {
                Ok(amd_devices) => {
                    info!("Found {} AMD GPU(s)", amd_devices.len());
                    devices.extend(amd_devices);
                }
                Err(e) => {
                    warn!("Failed to detect AMD GPUs: {}", e);
                }
            }
        }

        // 偵測 Apple GPU
        #[cfg(target_os = "macos")]
        {
            match Self::detect_apple() {
                Ok(apple_devices) => {
                    info!("Found {} Apple GPU(s)", apple_devices.len());
                    devices.extend(apple_devices);
                }
                Err(e) => {
                    warn!("Failed to detect Apple GPUs: {}", e);
                }
            }
        }

        if devices.is_empty() {
            return Err(Error::GPUNotFound);
        }

        info!("Total GPUs detected: {}", devices.len());

        let mut system_info = System::new_all();
        system_info.refresh_all();

        Ok(Self {
            devices,
            system_info: Arc::new(system_info),
        })
    }

    /// 偵測 NVIDIA GPU
    #[cfg(feature = "nvidia")]
    fn detect_nvidia() -> Result<Vec<GPUDeviceRef>> {
        use super::nvidia::NvidiaGPU;

        let nvml = nvml_wrapper::Nvml::init()?;
        let device_count = nvml.device_count()?;

        let mut devices = Vec::new();
        for i in 0..device_count {
            let device = nvml.device_by_index(i)?;
            devices.push(Arc::new(NvidiaGPU::new(device)) as GPUDeviceRef);
        }

        Ok(devices)
    }

    /// 偵測 AMD GPU
    #[cfg(feature = "amd")]
    fn detect_amd() -> Result<Vec<GPUDeviceRef>> {
        use super::amd::AmdGPU;

        // TODO: 實現 AMD GPU 偵測
        // 需要使用 ROCm SMI 或類似的 API
        Ok(Vec::new())
    }

    /// 偵測 Apple GPU
    #[cfg(target_os = "macos")]
    fn detect_apple() -> Result<Vec<GPUDeviceRef>> {
        use super::apple::AppleGPU;

        // TODO: 實現 Apple Metal GPU 偵測
        Ok(Vec::new())
    }

    /// 獲取 GPU 數量
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// 獲取指定索引的 GPU
    pub fn get_device(&self, index: usize) -> Option<&GPUDeviceRef> {
        self.devices.get(index)
    }

    /// 獲取所有 GPU
    pub fn get_all_devices(&self) -> &[GPUDeviceRef] {
        &self.devices
    }

    /// 獲取完整的硬體資訊
    pub fn get_hardware_info(&self) -> HardwareInfo {
        let gpus: Vec<GPUInfo> = self
            .devices
            .iter()
            .filter_map(|device| device.get_info().ok())
            .collect();

        let cpu = self.get_cpu_info();

        let total_memory_kb = self.system_info.total_memory();
        let memory_gb = (total_memory_kb / 1024 / 1024) as u32;

        // 估算可用存儲空間 (TODO: 更精確的計算)
        let storage_available_gb = 500; // 假設預留 500GB

        HardwareInfo {
            gpus,
            cpu,
            memory_gb,
            storage_available_gb,
        }
    }

    /// 獲取 CPU 資訊
    fn get_cpu_info(&self) -> CPUInfo {
        let cpus = self.system_info.cpus();
        let model = if !cpus.is_empty() {
            cpus[0].brand().to_string()
        } else {
            "Unknown CPU".to_string()
        };

        let cores = self.system_info.physical_core_count().unwrap_or(0) as u32;
        let threads = cpus.len() as u32;

        CPUInfo {
            model,
            cores,
            threads,
        }
    }

    /// 獲取所有 GPU 的即時狀態
    pub fn get_all_status(&self) -> Result<Vec<GPUStatus>> {
        self.devices
            .iter()
            .map(|device| device.get_status())
            .collect()
    }

    /// 檢查是否滿足任務需求
    pub fn meets_requirements(&self, requirements: &TaskRequirements) -> bool {
        self.devices
            .iter()
            .any(|device| device.meets_requirements(requirements).unwrap_or(false))
    }

    /// 選擇最適合的 GPU 執行任務
    pub fn select_best_gpu(&self, requirements: &TaskRequirements) -> Option<&GPUDeviceRef> {
        let mut suitable_devices: Vec<&GPUDeviceRef> = self
            .devices
            .iter()
            .filter(|device| device.meets_requirements(requirements).unwrap_or(false))
            .collect();

        if suitable_devices.is_empty() {
            return None;
        }

        // 按可用記憶體排序，選擇可用記憶體最多的
        suitable_devices.sort_by(|a, b| {
            let a_mem = a.memory_info().map(|m| m.free).unwrap_or(0);
            let b_mem = b.memory_info().map(|m| m.free).unwrap_or(0);
            b_mem.cmp(&a_mem)
        });

        suitable_devices.first().copied()
    }

    /// 計算工作證明 (使用第一個 GPU)
    pub fn compute_pow(&self, challenge: &[u8]) -> Result<Vec<u8>> {
        if let Some(device) = self.devices.first() {
            device.compute_pow(challenge, 4)
        } else {
            Err(Error::GPUNotFound)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_detection() {
        // 這個測試可能會失敗，如果系統沒有 GPU
        match GPUDetector::detect_all() {
            Ok(detector) => {
                println!("Detected {} GPU(s)", detector.device_count());
                assert!(detector.device_count() > 0);
            }
            Err(e) => {
                println!("No GPU detected: {}", e);
            }
        }
    }
}
