use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

// ==================== 硬體資訊 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub gpus: Vec<GPUInfo>,
    pub cpu: CPUInfo,
    pub memory_gb: u32,
    pub storage_available_gb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUInfo {
    pub index: u32,
    pub vendor: GPUVendor,
    pub model: String,
    pub vram_gb: u32,
    pub compute_capability: String,
    pub cuda_cores: Option<u32>,
    pub pcie_bandwidth_gbps: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GPUVendor {
    NVIDIA,
    AMD,
    Intel,
    Apple,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPUInfo {
    pub model: String,
    pub cores: u32,
    pub threads: u32,
}

// ==================== 能力與可用性 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub supported_frameworks: Vec<String>,
    pub max_batch_size: u32,
    pub fp16_support: bool,
    pub int8_support: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Availability {
    pub hours_per_day: u32,
    pub reliability_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub country: String,
    pub region: String,
    pub latency_to_platform_ms: u32,
}

// ==================== 任務定義 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_id: String,
    pub job_id: String,
    pub priority: u32,
    pub estimated_duration_sec: u32,
    pub requirements: TaskRequirements,
    pub payload: TaskPayload,
    pub pricing: Pricing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    pub min_vram_gb: u32,
    pub min_compute_capability: String,
    pub framework: String,
    pub fp16: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPayload {
    pub model_url: String,
    pub model_hash: String,
    pub input_data_url: String,
    pub output_url: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pricing {
    pub base_rate_usd_per_hour: Decimal,
    pub gpu_multiplier: Decimal,
    pub effective_rate: Decimal,
}

// ==================== 任務結果 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub output_url: String,
    pub output_hash: String,
    pub execution_time_sec: u32,
    pub gpu_time_sec: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfWork {
    pub method: String,
    pub challenge_id: String,
    pub response: Vec<u8>,
    pub gpu_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    pub avg_gpu_utilization: f32,
    pub peak_memory_gb: f32,
    pub energy_kwh: f32,
}

// ==================== GPU 狀態 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GPUStatus {
    pub index: u32,
    pub utilization: f32,
    pub memory_used_gb: f32,
    pub memory_total_gb: f32,
    pub temperature_c: f32,
    pub power_draw_w: f32,
    pub fan_speed_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub free: u64,
    pub used: u64,
}

impl MemoryInfo {
    pub fn total_gb(&self) -> f32 {
        self.total as f32 / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn used_gb(&self) -> f32 {
        self.used as f32 / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn utilization(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.used as f32 / self.total as f32
        }
    }
}

// ==================== 收益相關 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningRecord {
    pub timestamp: DateTime<Utc>,
    pub task_id: String,
    pub gpu_hours: f64,
    pub rate_per_hour: Decimal,
    pub amount: Decimal,
    pub status: EarningStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EarningStatus {
    Pending,
    Confirmed,
    Paid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsData {
    pub total_earnings: Decimal,
    pub today_earnings: Decimal,
    pub pending_earnings: Decimal,
    pub history: Vec<EarningRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutInfo {
    pub payout_id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_tasks: u32,
    pub total_gpu_hours: f64,
    pub gross_amount: Decimal,
    pub platform_fee: Decimal,
    pub net_amount: Decimal,
    pub payment_method: String,
    pub payment_address: String,
    pub status: PayoutStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayoutStatus {
    Processing,
    Completed,
    Failed,
}

// ==================== Dashboard 資料 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub earnings: EarningsData,
    pub gpu_status: Vec<GPUStatus>,
    pub is_running: bool,
    pub uptime_sec: u64,
    pub tasks_completed_today: u32,
}
