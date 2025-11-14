use super::device::GPUDevice;
use crate::types::{GPUVendor, MemoryInfo};
use crate::error::{Error, Result};
use nvml_wrapper::{Device, Nvml};
use std::sync::Arc;
use sha2::{Sha256, Digest};

/// NVIDIA GPU 設備
pub struct NvidiaGPU {
    device: Device<'static>,
    index: u32,
    nvml: Arc<Nvml>,
}

// SAFETY: NVML Device 可以安全地在線程間傳遞
unsafe impl Send for NvidiaGPU {}
unsafe impl Sync for NvidiaGPU {}

impl NvidiaGPU {
    pub fn new(device: Device<'static>) -> Self {
        let index = device.index().unwrap_or(0);

        // 保持 NVML 實例的所有權
        // 注意：這裡我們假設 NVML 已經初始化
        // 實際使用時需要更好的生命週期管理
        let nvml = Arc::new(Nvml::init().expect("Failed to initialize NVML"));

        Self {
            device,
            index,
            nvml,
        }
    }

    /// 獲取 CUDA 核心數量（基於架構推算）
    fn estimate_cuda_cores(&self) -> Option<u32> {
        let name = self.name().ok()?;

        // 根據型號名稱估算 CUDA 核心數
        // 這是一個簡化的估算，實際應該查詢完整的規格資料庫
        if name.contains("4090") {
            Some(16384)
        } else if name.contains("4080") {
            Some(9728)
        } else if name.contains("3090") {
            Some(10496)
        } else if name.contains("3080") {
            Some(8704)
        } else if name.contains("A100") {
            Some(6912)
        } else if name.contains("V100") {
            Some(5120)
        } else {
            None
        }
    }
}

impl GPUDevice for NvidiaGPU {
    fn index(&self) -> u32 {
        self.index
    }

    fn vendor(&self) -> GPUVendor {
        GPUVendor::NVIDIA
    }

    fn name(&self) -> Result<String> {
        Ok(self.device.name()?)
    }

    fn memory_info(&self) -> Result<MemoryInfo> {
        let mem_info = self.device.memory_info()?;
        Ok(MemoryInfo {
            total: mem_info.total,
            free: mem_info.free,
            used: mem_info.used,
        })
    }

    fn utilization(&self) -> Result<f32> {
        let util = self.device.utilization_rates()?;
        Ok(util.gpu as f32 / 100.0)
    }

    fn temperature(&self) -> Result<f32> {
        use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
        let temp = self.device.temperature(TemperatureSensor::Gpu)?;
        Ok(temp as f32)
    }

    fn power_usage(&self) -> Result<f32> {
        let power = self.device.power_usage()?;
        Ok(power as f32 / 1000.0) // mW to W
    }

    fn fan_speed(&self) -> Result<f32> {
        match self.device.fan_speed(0) {
            Ok(speed) => Ok(speed as f32 / 100.0),
            Err(_) => Ok(0.0), // 有些 GPU 可能沒有風扇感測器
        }
    }

    fn compute_capability(&self) -> Result<String> {
        let (major, minor) = self.device.cuda_compute_capability()?;
        Ok(format!("{}.{}", major, minor))
    }

    fn cuda_cores(&self) -> Option<u32> {
        self.estimate_cuda_cores()
    }

    fn pcie_bandwidth(&self) -> Result<u32> {
        // 獲取 PCIe 世代和寬度
        use nvml_wrapper::enum_wrappers::device::PcieLinkMaxSpeed;

        let max_link_gen = self.device.max_pcie_link_gen()?;
        let max_link_width = self.device.max_pcie_link_width()?;

        // PCIe Gen3 x16 = 16 GB/s, Gen4 x16 = 32 GB/s, Gen5 x16 = 64 GB/s
        let bandwidth_per_lane = match max_link_gen {
            3 => 1,   // 1 GB/s per lane
            4 => 2,   // 2 GB/s per lane
            5 => 4,   // 4 GB/s per lane
            _ => 1,
        };

        Ok(bandwidth_per_lane * max_link_width)
    }

    fn uuid(&self) -> Result<String> {
        Ok(self.device.uuid()?)
    }

    fn compute_pow(&self, challenge: &[u8], difficulty: u32) -> Result<Vec<u8>> {
        // 簡化的 PoW 實現
        // 實際應該使用 CUDA kernel 進行並行計算

        // TODO: 使用 cudarc 實現 GPU 並行哈希搜索
        // 這裡先用 CPU 實現作為示例

        let mut nonce: u64 = 0;
        loop {
            let mut hasher = Sha256::new();
            hasher.update(challenge);
            hasher.update(&nonce.to_le_bytes());
            let hash = hasher.finalize();

            // 檢查前 difficulty 位是否為 0
            let leading_zeros = hash.iter().take_while(|&&b| b == 0).count();
            if leading_zeros >= difficulty as usize {
                return Ok(hash.to_vec());
            }

            nonce += 1;
            if nonce > 10_000_000 {
                return Err(Error::TaskTimeout);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // 僅在有 NVIDIA GPU 的系統上運行
    fn test_nvidia_gpu() {
        let nvml = Nvml::init().unwrap();
        let device_count = nvml.device_count().unwrap();

        if device_count > 0 {
            let device = nvml.device_by_index(0).unwrap();
            let gpu = NvidiaGPU::new(device);

            println!("GPU Name: {}", gpu.name().unwrap());
            println!("Memory: {:?}", gpu.memory_info().unwrap());
            println!("Utilization: {:.2}%", gpu.utilization().unwrap() * 100.0);
            println!("Temperature: {:.1}°C", gpu.temperature().unwrap());
        }
    }
}
