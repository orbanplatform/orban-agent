//! 工作證明生成器
//!
//! 從第一性原理思考：如何證明確實用 GPU 執行了計算？
//!
//! 策略：
//! 1. GPU 特有的計算：只能用 GPU 高效完成的任務
//! 2. 硬體簽名：GPU 的唯一標識
//! 3. 時間戳：防止重放攻擊
//! 4. 結果哈希：綁定計算結果

use crate::{
    Result, Error,
    gpu::GPUDevice,
    types::{Task, ProofOfWork, ResultData},
};
use chrono::Utc;
use sha2::{Sha256, Digest};

/// 工作證明生成器
pub struct ProofGenerator {
    // 可以添加配置選項
}

impl ProofGenerator {
    /// 創建新的證明生成器
    pub fn new() -> Self {
        Self {}
    }

    /// 生成工作證明
    pub fn generate(
        &self,
        task: &Task,
        result: &ResultData,
        gpu: &Arc<dyn GPUDevice>,
        gpu_time_seconds: f64,
    ) -> Result<ProofOfWork> {
        // 1. 取得挑戰值（應該從平台獲取，這裡簡化）
        let challenge = self.get_challenge(task);

        // 2. 計算回應（GPU 密集型計算）
        let response = self.compute_response(&challenge, result, gpu)?;

        // 3. 生成 GPU 簽名
        let gpu_signature = self.generate_gpu_signature(gpu, gpu_time_seconds);

        // 4. 創建元數據
        let metadata = serde_json::json!({
            "gpu_time_seconds": gpu_time_seconds,
            "gpu_model": gpu.name(),
            "gpu_memory_gb": gpu.total_memory_gb(),
        });

        Ok(ProofOfWork {
            challenge,
            response,
            gpu_signature,
            timestamp: Utc::now(),
            metadata: Some(metadata),
        })
    }

    /// 取得挑戰值
    fn get_challenge(&self, task: &Task) -> String {
        // 實際應該從平台獲取隨機挑戰值
        // 這裡簡化為任務 ID 的哈希
        let mut hasher = Sha256::new();
        hasher.update(task.id.as_bytes());
        hasher.update(task.created_at.to_rfc3339().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 計算回應
    ///
    /// 這應該是只能用 GPU 高效計算的函數
    fn compute_response(
        &self,
        challenge: &str,
        result: &ResultData,
        gpu: &Arc<dyn GPUDevice>,
    ) -> Result<String> {
        // 簡化實作：計算 challenge + result 的哈希
        // 實際應該執行 GPU 密集型計算（如特定的哈希算法）

        let result_str = match result {
            ResultData::Url { url, checksum, .. } => {
                format!("{}-{}", url, checksum)
            }
            ResultData::Inline { data } => data.clone(),
        };

        let mut hasher = Sha256::new();
        hasher.update(challenge.as_bytes());
        hasher.update(result_str.as_bytes());
        hasher.update(gpu.hardware_id().as_bytes());

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// 生成 GPU 簽名
    ///
    /// 結合 GPU 硬體 ID 和執行資訊
    fn generate_gpu_signature(
        &self,
        gpu: &Arc<dyn GPUDevice>,
        gpu_time_seconds: f64,
    ) -> String {
        let mut hasher = Sha256::new();
        hasher.update(gpu.hardware_id().as_bytes());
        hasher.update(gpu.name().as_bytes());
        hasher.update(gpu_time_seconds.to_string().as_bytes());

        format!("{:x}", hasher.finalize())
    }
}

use std::sync::Arc;

impl Default for ProofGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proof_generator_creation() {
        let generator = ProofGenerator::new();
        assert!(true); // 基本創建測試
    }
}
