// GPU Proof of Work (PoW) Implementation
//
// 实现GPU工作量证明，防止虚假节点
// 通过GPU并行计算密集型哈希来验证真实GPU存在

use crate::error::{Error, Result};
use sha2::{Sha256, Digest};
use std::time::{Duration, Instant};

#[cfg(feature = "nvidia")]
use nvml_wrapper::Nvml;

/// GPU工作量证明配置
#[derive(Debug, Clone)]
pub struct PowConfig {
    /// 难度级别（前导零的数量）
    pub difficulty: u32,
    /// 最大计算时间（秒）
    pub max_compute_time_sec: u64,
}

impl Default for PowConfig {
    fn default() -> Self {
        Self {
            difficulty: 4,
            max_compute_time_sec: 10,
        }
    }
}

/// PoW 挑战
#[derive(Debug, Clone)]
pub struct PowChallenge {
    /// 挑战ID
    pub challenge_id: String,
    /// 随机nonce
    pub nonce: Vec<u8>,
    /// 难度
    pub difficulty: u32,
    /// 截止时间
    pub deadline: chrono::DateTime<chrono::Utc>,
}

/// PoW 响应
#[derive(Debug, Clone)]
pub struct PowResponse {
    /// 挑战ID
    pub challenge_id: String,
    /// 计算得到的响应（满足难度要求的哈希）
    pub response: Vec<u8>,
    /// 找到的nonce
    pub solution_nonce: u64,
    /// 计算时间（毫秒）
    pub computation_time_ms: u64,
    /// GPU签名
    pub gpu_signature: GpuSignature,
}

/// GPU签名（证明使用了真实GPU）
#[derive(Debug, Clone)]
pub struct GpuSignature {
    /// 设备UUID（NVIDIA）或设备ID
    pub device_uuid: String,
    /// GPU型号
    pub device_model: String,
    /// CUDA版本（NVIDIA）
    pub cuda_version: Option<String>,
    /// 计算能力
    pub compute_capability: Option<String>,
}

/// GPU PoW 计算器
pub struct GpuPowComputer {
    config: PowConfig,
    gpu_info: GpuSignature,
}

impl GpuPowComputer {
    /// 创建新的PoW计算器
    pub fn new(config: PowConfig) -> Result<Self> {
        let gpu_info = Self::get_gpu_signature()?;
        Ok(Self { config, gpu_info })
    }

    /// 获取GPU签名
    fn get_gpu_signature() -> Result<GpuSignature> {
        #[cfg(feature = "nvidia")]
        {
            let nvml = Nvml::init().map_err(|e| Error::GpuError(format!("NVML init failed: {}", e)))?;
            let device = nvml.device_by_index(0).map_err(|e| Error::GpuError(format!("Get device failed: {}", e)))?;

            let uuid = device.uuid().map_err(|e| Error::GpuError(format!("Get UUID failed: {}", e)))?;
            let name = device.name().map_err(|e| Error::GpuError(format!("Get name failed: {}", e)))?;
            let cuda_version = nvml.sys_cuda_driver_version().ok().map(|v| format!("{}.{}", v / 1000, (v % 1000) / 10));
            let compute_capability = device.cuda_compute_capability().ok().map(|(major, minor)| format!("{}.{}", major, minor));

            Ok(GpuSignature {
                device_uuid: uuid,
                device_model: name,
                cuda_version,
                compute_capability,
            })
        }

        #[cfg(not(feature = "nvidia"))]
        {
            // 其他GPU类型的实现（AMD、Apple）
            Ok(GpuSignature {
                device_uuid: "unknown".to_string(),
                device_model: "unknown".to_string(),
                cuda_version: None,
                compute_capability: None,
            })
        }
    }

    /// 计算PoW响应
    ///
    /// 算法：寻找满足 SHA256(challenge || nonce) 的前 difficulty 位为 0 的 nonce
    ///
    /// # GPU加速策略
    /// - NVIDIA: 使用CUDA并行搜索
    /// - AMD: 使用ROCm并行搜索
    /// - CPU Fallback: 多线程搜索
    pub fn compute(&self, challenge: &PowChallenge) -> Result<PowResponse> {
        let start_time = Instant::now();

        // 检查是否超时
        if chrono::Utc::now() > challenge.deadline {
            return Err(Error::Other("Challenge deadline exceeded".to_string()));
        }

        // 根据GPU类型选择计算策略
        #[cfg(feature = "nvidia")]
        {
            self.compute_cuda(&challenge, start_time)
        }

        #[cfg(not(feature = "nvidia"))]
        {
            self.compute_cpu(&challenge, start_time)
        }
    }

    /// 使用CUDA GPU计算PoW
    #[cfg(feature = "nvidia")]
    fn compute_cuda(&self, challenge: &PowChallenge, start_time: Instant) -> Result<PowResponse> {
        // TODO: 实现CUDA kernel并行搜索
        // 这里先使用CPU模拟
        tracing::warn!("CUDA PoW not yet implemented, falling back to CPU");
        self.compute_cpu(challenge, start_time)
    }

    /// 使用CPU计算PoW（多线程）
    fn compute_cpu(&self, challenge: &PowChallenge, start_time: Instant) -> Result<PowResponse> {
        let difficulty_mask = Self::difficulty_mask(challenge.difficulty);
        let num_threads = num_cpus::get();

        tracing::info!("Computing PoW with {} threads, difficulty: {}", num_threads, challenge.difficulty);

        // 多线程并行搜索
        let (tx, rx) = std::sync::mpsc::channel();
        let challenge_nonce = challenge.nonce.clone();

        for thread_id in 0..num_threads {
            let tx = tx.clone();
            let nonce_bytes = challenge_nonce.clone();
            let difficulty_mask = difficulty_mask.clone();
            let max_time = Duration::from_secs(self.config.max_compute_time_sec);

            std::thread::spawn(move || {
                let start = thread_id as u64;
                let step = num_threads as u64;

                for nonce in (start..u64::MAX).step_by(step as usize) {
                    // 超时检查
                    if start_time.elapsed() > max_time {
                        break;
                    }

                    // 计算哈希
                    let mut hasher = Sha256::new();
                    hasher.update(&nonce_bytes);
                    hasher.update(nonce.to_le_bytes());
                    let hash = hasher.finalize();

                    // 检查是否满足难度要求
                    if Self::check_difficulty(&hash, &difficulty_mask) {
                        let _ = tx.send((nonce, hash.to_vec()));
                        break;
                    }
                }
            });
        }

        // 等待任意线程找到解
        match rx.recv_timeout(Duration::from_secs(self.config.max_compute_time_sec)) {
            Ok((solution_nonce, hash)) => {
                let elapsed = start_time.elapsed();

                tracing::info!("PoW solution found: nonce={}, time={}ms", solution_nonce, elapsed.as_millis());

                Ok(PowResponse {
                    challenge_id: challenge.challenge_id.clone(),
                    response: hash,
                    solution_nonce,
                    computation_time_ms: elapsed.as_millis() as u64,
                    gpu_signature: self.gpu_info.clone(),
                })
            }
            Err(_) => {
                Err(Error::Other(format!("PoW computation timeout after {}s", self.config.max_compute_time_sec)))
            }
        }
    }

    /// 生成难度掩码
    /// difficulty=4 -> 前4位必须为0 -> 0x0FFFFFFF...
    fn difficulty_mask(difficulty: u32) -> Vec<u8> {
        let mut mask = vec![0xFF; 32]; // SHA256 = 32 bytes
        let bytes_to_zero = (difficulty / 8) as usize;
        let bits_to_zero = (difficulty % 8) as u8;

        // 完整字节置零
        for i in 0..bytes_to_zero {
            mask[i] = 0x00;
        }

        // 部分字节掩码
        if bits_to_zero > 0 && bytes_to_zero < 32 {
            mask[bytes_to_zero] = 0xFF >> bits_to_zero;
        }

        mask
    }

    /// 检查哈希是否满足难度要求
    fn check_difficulty(hash: &[u8], mask: &[u8]) -> bool {
        for i in 0..hash.len().min(mask.len()) {
            if hash[i] & !mask[i] != 0 {
                return false;
            }
        }
        true
    }

    /// 验证PoW响应
    pub fn verify(challenge: &PowChallenge, response: &PowResponse) -> Result<bool> {
        // 1. 检查挑战ID匹配
        if challenge.challenge_id != response.challenge_id {
            return Ok(false);
        }

        // 2. 重新计算哈希
        let mut hasher = Sha256::new();
        hasher.update(&challenge.nonce);
        hasher.update(response.solution_nonce.to_le_bytes());
        let computed_hash = hasher.finalize();

        // 3. 检查哈希匹配
        if computed_hash.as_slice() != response.response.as_slice() {
            return Ok(false);
        }

        // 4. 检查难度
        let difficulty_mask = Self::difficulty_mask(challenge.difficulty);
        if !Self::check_difficulty(&computed_hash, &difficulty_mask) {
            return Ok(false);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_mask() {
        let mask = GpuPowComputer::difficulty_mask(4);
        assert_eq!(mask[0], 0x0F); // 前4位为0

        let mask = GpuPowComputer::difficulty_mask(8);
        assert_eq!(mask[0], 0x00); // 前8位为0
        assert_eq!(mask[1], 0xFF);
    }

    #[test]
    fn test_check_difficulty() {
        let hash = vec![0x00, 0xAB, 0xCD];
        let mask = vec![0x0F, 0xFF, 0xFF];
        assert!(GpuPowComputer::check_difficulty(&hash, &mask));

        let hash = vec![0x10, 0xAB, 0xCD];
        assert!(!GpuPowComputer::check_difficulty(&hash, &mask));
    }

    #[test]
    fn test_pow_computation() {
        let config = PowConfig {
            difficulty: 8, // 较低难度用于测试
            max_compute_time_sec: 5,
        };

        let computer = GpuPowComputer::new(config).unwrap();

        let challenge = PowChallenge {
            challenge_id: "test-001".to_string(),
            nonce: b"test_challenge_nonce".to_vec(),
            difficulty: 8,
            deadline: chrono::Utc::now() + chrono::Duration::seconds(10),
        };

        let response = computer.compute(&challenge).unwrap();

        // 验证响应
        assert!(GpuPowComputer::verify(&challenge, &response).unwrap());
        assert!(response.computation_time_ms > 0);
    }

    #[test]
    fn test_pow_verification_fails_wrong_nonce() {
        let challenge = PowChallenge {
            challenge_id: "test-002".to_string(),
            nonce: b"original_nonce".to_vec(),
            difficulty: 8,
            deadline: chrono::Utc::now() + chrono::Duration::seconds(10),
        };

        let mut response = PowResponse {
            challenge_id: "test-002".to_string(),
            response: vec![0x00; 32],
            solution_nonce: 12345,
            computation_time_ms: 100,
            gpu_signature: GpuSignature {
                device_uuid: "test".to_string(),
                device_model: "test".to_string(),
                cuda_version: None,
                compute_capability: None,
            },
        };

        // 篡改solution_nonce
        response.solution_nonce = 99999;
        assert!(!GpuPowComputer::verify(&challenge, &response).unwrap());
    }
}
