//! GPU 抽象層
//!
//! 從第一性原理思考 GPU：
//! - GPU 是什麼？並行計算處理器
//! - 我們需要知道什麼？型號、記憶體、當前狀態
//! - 如何跨平台？統一的 trait 抽象不同廠商
//!
//! 支援的 GPU：
//! - NVIDIA (CUDA)
//! - AMD (ROCm)
//! - Apple (Metal)
//! - Intel (oneAPI, 未來支援)

mod detector;
mod monitor;

#[cfg(feature = "nvidia")]
mod nvidia;

#[cfg(feature = "amd")]
mod amd;

#[cfg(all(target_os = "macos", feature = "apple"))]
mod apple;

pub use detector::GPUDetector;
pub use monitor::GPUMonitor;

use crate::{Result, types::GpuInfo};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// GPU 裝置的統一接口
///
/// 這個 trait 定義了所有 GPU 必須實作的基本功能
#[async_trait]
pub trait GPUDevice: Send + Sync {
    /// 取得 GPU 名稱
    fn name(&self) -> String;

    /// 取得 GPU 類型
    fn gpu_type(&self) -> crate::types::GpuType;

    /// 取得總記憶體（GB）
    fn total_memory_gb(&self) -> f32;

    /// 取得當前記憶體使用（GB）
    fn used_memory_gb(&self) -> Result<f32>;

    /// 取得 GPU 使用率 (0.0-1.0)
    fn utilization(&self) -> Result<f32>;

    /// 取得溫度（攝氏）
    fn temperature(&self) -> Result<f32>;

    /// 取得功耗（瓦特）
    fn power_usage(&self) -> Result<f32>;

    /// 取得計算能力（CUDA Compute Capability 或等效）
    fn compute_capability(&self) -> Option<String>;

    /// 取得驅動版本
    fn driver_version(&self) -> Option<String>;

    /// 取得硬體 ID（用於驗證）
    fn hardware_id(&self) -> String;

    /// 檢查是否可用
    fn is_available(&self) -> bool;

    /// 轉換為 GpuInfo
    fn to_info(&self) -> GpuInfo {
        GpuInfo {
            model: self.name(),
            gpu_type: self.gpu_type(),
            total_memory_gb: self.total_memory_gb(),
            compute_capability: self.compute_capability(),
            driver_version: self.driver_version(),
            hardware_id: self.hardware_id(),
        }
    }

    /// 取得當前狀態
    fn get_status(&self) -> Result<GPUStatus> {
        Ok(GPUStatus {
            name: self.name(),
            gpu_type: self.gpu_type(),
            utilization: self.utilization()?,
            memory_used_gb: self.used_memory_gb()?,
            memory_total_gb: self.total_memory_gb(),
            temperature: self.temperature()?,
            power_usage: self.power_usage()?,
            is_available: self.is_available(),
        })
    }
}

/// GPU 即時狀態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUStatus {
    /// GPU 名稱
    pub name: String,

    /// GPU 類型
    pub gpu_type: crate::types::GpuType,

    /// 使用率 (0.0-1.0)
    pub utilization: f32,

    /// 已使用記憶體（GB）
    pub memory_used_gb: f32,

    /// 總記憶體（GB）
    pub memory_total_gb: f32,

    /// 溫度（攝氏）
    pub temperature: f32,

    /// 功耗（瓦特）
    pub power_usage: f32,

    /// 是否可用
    pub is_available: bool,
}

impl GPUStatus {
    /// 記憶體使用百分比
    pub fn memory_usage_percent(&self) -> f32 {
        if self.memory_total_gb > 0.0 {
            (self.memory_used_gb / self.memory_total_gb) * 100.0
        } else {
            0.0
        }
    }

    /// 是否閒置（使用率低於 10%）
    pub fn is_idle(&self) -> bool {
        self.utilization < 0.1
    }

    /// 是否過熱（超過 80°C）
    pub fn is_overheating(&self) -> bool {
        self.temperature > 80.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_status_calculations() {
        let status = GPUStatus {
            name: "Test GPU".to_string(),
            gpu_type: crate::types::GpuType::Nvidia,
            utilization: 0.05,
            memory_used_gb: 2.0,
            memory_total_gb: 8.0,
            temperature: 65.0,
            power_usage: 150.0,
            is_available: true,
        };

        assert_eq!(status.memory_usage_percent(), 25.0);
        assert!(status.is_idle());
        assert!(!status.is_overheating());
    }
}
