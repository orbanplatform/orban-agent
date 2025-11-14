use crate::types::{GPUInfo, GPUStatus, GPUVendor, MemoryInfo};
use crate::error::Result;
use std::sync::Arc;

/// GPU 設備類型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    NVIDIA,
    AMD,
    Intel,
    Apple,
}

/// GPU 設備抽象接口
///
/// 所有 GPU 實現都必須實現這個 trait
pub trait GPUDevice: Send + Sync {
    /// 獲取設備索引
    fn index(&self) -> u32;

    /// 獲取 GPU 廠商
    fn vendor(&self) -> GPUVendor;

    /// 獲取 GPU 型號名稱
    fn name(&self) -> Result<String>;

    /// 獲取 VRAM 資訊
    fn memory_info(&self) -> Result<MemoryInfo>;

    /// 獲取 GPU 使用率 (0.0 - 1.0)
    fn utilization(&self) -> Result<f32>;

    /// 獲取溫度 (攝氏度)
    fn temperature(&self) -> Result<f32>;

    /// 獲取功耗 (瓦特)
    fn power_usage(&self) -> Result<f32>;

    /// 獲取風扇轉速 (0.0 - 1.0)
    fn fan_speed(&self) -> Result<f32>;

    /// 獲取計算能力版本 (如 "8.9" 表示 CUDA 8.9)
    fn compute_capability(&self) -> Result<String>;

    /// 獲取 CUDA 核心數量 (僅 NVIDIA)
    fn cuda_cores(&self) -> Option<u32> {
        None
    }

    /// 獲取 PCIe 頻寬 (GB/s)
    fn pcie_bandwidth(&self) -> Result<u32>;

    /// 獲取設備 UUID (用於唯一識別)
    fn uuid(&self) -> Result<String>;

    /// 獲取完整的 GPU 資訊
    fn get_info(&self) -> Result<GPUInfo> {
        let memory = self.memory_info()?;
        Ok(GPUInfo {
            index: self.index(),
            vendor: self.vendor(),
            model: self.name()?,
            vram_gb: (memory.total_gb().ceil() as u32),
            compute_capability: self.compute_capability()?,
            cuda_cores: self.cuda_cores(),
            pcie_bandwidth_gbps: self.pcie_bandwidth()?,
        })
    }

    /// 獲取即時狀態
    fn get_status(&self) -> Result<GPUStatus> {
        let memory = self.memory_info()?;
        Ok(GPUStatus {
            index: self.index(),
            utilization: self.utilization()?,
            memory_used_gb: memory.used_gb(),
            memory_total_gb: memory.total_gb(),
            temperature_c: self.temperature()?,
            power_draw_w: self.power_usage()?,
            fan_speed_percent: self.fan_speed()? * 100.0,
        })
    }

    /// 檢查是否滿足任務需求
    fn meets_requirements(&self, requirements: &TaskRequirements) -> Result<bool> {
        let info = self.get_info()?;
        let memory = self.memory_info()?;

        // 檢查 VRAM
        if info.vram_gb < requirements.min_vram_gb {
            return Ok(false);
        }

        // 檢查可用 VRAM (需要至少 80% 的需求量可用)
        let required_bytes = (requirements.min_vram_gb as u64) * 1024 * 1024 * 1024;
        if memory.free < (required_bytes * 8 / 10) {
            return Ok(false);
        }

        // 檢查計算能力
        let device_capability = self.parse_compute_capability(&info.compute_capability)?;
        let required_capability = self.parse_compute_capability(&requirements.min_compute_capability)?;
        if device_capability < required_capability {
            return Ok(false);
        }

        Ok(true)
    }

    /// 解析計算能力版本
    fn parse_compute_capability(&self, version: &str) -> Result<(u32, u32)> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 2 {
            return Ok((0, 0));
        }
        let major = parts[0].parse().unwrap_or(0);
        let minor = parts[1].parse().unwrap_or(0);
        Ok((major, minor))
    }

    /// 執行 GPU 工作證明計算
    fn compute_pow(&self, challenge: &[u8], difficulty: u32) -> Result<Vec<u8>>;
}

/// GPU 設備的線程安全包裝
pub type GPUDeviceRef = Arc<dyn GPUDevice>;
