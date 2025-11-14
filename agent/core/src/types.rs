//! 共用類型定義
//!
//! 定義整個系統中使用的核心資料結構

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// 任務類型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    /// 推論任務
    Inference,
    /// 訓練任務
    Training,
    /// 微調任務
    FineTuning,
}

/// 任務狀態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    /// 等待中
    Pending,
    /// 已分配
    Assigned,
    /// 下載中
    Downloading,
    /// 執行中
    Running,
    /// 已完成
    Completed,
    /// 失敗
    Failed,
}

/// 任務定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任務 ID
    pub id: String,

    /// 任務類型
    pub task_type: TaskType,

    /// 任務需求
    pub requirements: TaskRequirements,

    /// 模型資訊
    pub model: ModelInfo,

    /// 資料集資訊（如果需要）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset: Option<DatasetInfo>,

    /// 執行參數
    pub params: serde_json::Value,

    /// 建立時間
    pub created_at: DateTime<Utc>,

    /// 超時時間（秒）
    pub timeout_seconds: u64,
}

/// 任務需求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    /// 需要的 VRAM (GB)
    pub vram_gb: f32,

    /// 最低計算能力（CUDA Compute Capability）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_compute_capability: Option<String>,

    /// GPU 類型偏好
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_gpu_type: Option<GpuType>,
}

/// GPU 類型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GpuType {
    Nvidia,
    Amd,
    Intel,
    Apple,
}

/// 模型資訊
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// 模型名稱
    pub name: String,

    /// 模型版本
    pub version: String,

    /// 下載 URL 或 HuggingFace ID
    pub source: String,

    /// 模型大小（bytes）
    pub size_bytes: u64,

    /// 校驗和（SHA256）
    pub checksum: String,
}

/// 資料集資訊
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    /// 資料集名稱
    pub name: String,

    /// 下載 URL
    pub source: String,

    /// 大小（bytes）
    pub size_bytes: u64,

    /// 校驗和
    pub checksum: String,
}

/// 任務結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// 任務 ID
    pub task_id: String,

    /// Agent ID
    pub agent_id: String,

    /// 執行狀態
    pub status: TaskStatus,

    /// 開始時間
    pub started_at: DateTime<Utc>,

    /// 完成時間
    pub completed_at: DateTime<Utc>,

    /// GPU 執行時間（秒）
    pub gpu_time_seconds: f64,

    /// 使用的 GPU 資訊
    pub gpu_used: GpuInfo,

    /// 結果資料（URL 或 inline data）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_data: Option<ResultData>,

    /// 工作證明
    pub proof: ProofOfWork,

    /// 錯誤訊息（如果失敗）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 結果資料
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResultData {
    /// URL 引用
    Url {
        url: String,
        size_bytes: u64,
        checksum: String,
    },
    /// Inline 資料（小結果）
    Inline {
        data: String,
    },
}

/// 工作證明
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfWork {
    /// 挑戰值（由平台提供）
    pub challenge: String,

    /// 回應（GPU 計算的結果）
    pub response: String,

    /// GPU 硬體簽名
    pub gpu_signature: String,

    /// 時間戳
    pub timestamp: DateTime<Utc>,

    /// 額外的驗證資料
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// GPU 資訊
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU 型號
    pub model: String,

    /// GPU 類型
    pub gpu_type: GpuType,

    /// 總記憶體（GB）
    pub total_memory_gb: f32,

    /// 計算能力
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_capability: Option<String>,

    /// 驅動版本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver_version: Option<String>,

    /// 硬體 ID（用於驗證）
    pub hardware_id: String,
}

/// 收益狀態
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EarningStatus {
    /// 待確認
    Pending,
    /// 已確認
    Confirmed,
    /// 已支付
    Paid,
}

/// 系統指標
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// CPU 使用率 (0-100)
    pub cpu_usage: f32,

    /// RAM 使用率 (0-100)
    pub ram_usage: f32,

    /// 網路下載速度 (Mbps)
    pub network_download_mbps: f32,

    /// 網路上傳速度 (Mbps)
    pub network_upload_mbps: f32,

    /// 系統溫度
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_temp_celsius: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_serialization() {
        let task = Task {
            id: "task-123".to_string(),
            task_type: TaskType::Inference,
            requirements: TaskRequirements {
                vram_gb: 8.0,
                min_compute_capability: Some("7.5".to_string()),
                preferred_gpu_type: Some(GpuType::Nvidia),
            },
            model: ModelInfo {
                name: "llama-2-7b".to_string(),
                version: "1.0".to_string(),
                source: "meta-llama/Llama-2-7b-hf".to_string(),
                size_bytes: 13_000_000_000,
                checksum: "abc123".to_string(),
            },
            dataset: None,
            params: serde_json::json!({}),
            created_at: Utc::now(),
            timeout_seconds: 3600,
        };

        let json = serde_json::to_string(&task).unwrap();
        let deserialized: Task = serde_json::from_str(&json).unwrap();

        assert_eq!(task.id, deserialized.id);
        assert_eq!(task.task_type, deserialized.task_type);
    }
}
