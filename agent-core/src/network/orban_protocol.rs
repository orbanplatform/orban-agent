// Orban Protocol 實現
//
// 定義 Agent 與 Platform 之間的通訊協議

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::types::*;
use crate::error::Result;

/// 訊息類型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessageType {
    // 認證
    AuthChallenge,
    AuthResponse,
    AuthSuccess,

    // 註冊
    AgentRegister,
    RegisterAck,

    // 任務管理
    TaskAssign,
    TaskAccept,
    TaskReject,
    TaskProgress,
    TaskComplete,
    TaskFailed,

    // 監控
    Heartbeat,
    MetricsBatch,

    // 收益
    EarningsRecord,
    PayoutNotification,

    // 工作證明
    PowChallenge,
    PowResponse,

    // 錯誤
    Error,

    // 狀態同步
    StateSync,
}

/// Orban Protocol 訊息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message_id: String,
    pub timestamp: DateTime<Utc>,

    #[serde(rename = "type")]
    pub message_type: MessageType,

    #[serde(flatten)]
    pub payload: MessagePayload,
}

/// 訊息負載
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessagePayload {
    AuthChallenge(AuthChallengePayload),
    AuthResponse(AuthResponsePayload),
    AuthSuccess(AuthSuccessPayload),
    AgentRegister(AgentRegisterPayload),
    RegisterAck(RegisterAckPayload),
    TaskAssign(TaskAssignPayload),
    TaskAccept(TaskAcceptPayload),
    TaskReject(TaskRejectPayload),
    TaskProgress(TaskProgressPayload),
    TaskComplete(TaskCompletePayload),
    TaskFailed(TaskFailedPayload),
    Heartbeat(HeartbeatPayload),
    MetricsBatch(MetricsBatchPayload),
    EarningsRecord(EarningsRecordPayload),
    PayoutNotification(PayoutNotificationPayload),
    PowChallenge(PowChallengePayload),
    PowResponse(PowResponsePayload),
    Error(ErrorPayload),
    StateSync(StateSyncPayload),
}

// ==================== 認證訊息 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthChallengePayload {
    pub challenge: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponsePayload {
    pub agent_id: String,
    pub signature: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSuccessPayload {
    pub jwt_token: String,
    pub expires_in: u64,
}

// ==================== 註冊訊息 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegisterPayload {
    pub agent_id: String,
    pub hardware: HardwareInfo,
    pub capabilities: Capabilities,
    pub location: Location,
    pub availability: Availability,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAckPayload {
    pub agent_id: String,
    pub status: String,
    pub pricing: Pricing,
}

// ==================== 任務訊息 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignPayload {
    pub task_id: String,
    pub job_id: String,
    pub priority: u32,
    pub estimated_duration_sec: u32,
    pub requirements: TaskRequirements,
    pub payload: TaskPayload,
    pub pricing: Pricing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAcceptPayload {
    pub task_id: String,
    pub agent_id: String,
    pub gpu_allocated: u32,
    pub estimated_completion: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRejectPayload {
    pub task_id: String,
    pub reason: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgressPayload {
    pub task_id: String,
    pub progress: f32,
    pub stage: String,
    pub metrics: TaskMetrics,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub gpu_utilization: f32,
    pub memory_used_gb: f32,
    pub throughput_tokens_per_sec: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletePayload {
    pub task_id: String,
    pub result: TaskResult,
    pub proof_of_work: ProofOfWork,
    pub metrics: ExecutionMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFailedPayload {
    pub task_id: String,
    pub error: TaskErrorInfo,
    pub partial_results: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskErrorInfo {
    pub code: String,
    pub message: String,
    pub details: String,
}

// ==================== 監控訊息 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatPayload {
    pub agent_id: String,
    pub status: AgentStatus,
    pub current_task_id: Option<String>,
    pub gpu_status: Vec<GPUStatus>,
    pub uptime_sec: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Idle,
    Working,
    Error,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsBatchPayload {
    pub agent_id: String,
    pub time_range: TimeRange,
    pub aggregated_metrics: AggregatedMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub tasks_completed: u32,
    pub tasks_failed: u32,
    pub total_gpu_hours: f64,
    pub avg_gpu_utilization: f32,
    pub total_energy_kwh: f32,
    pub earnings_usd: rust_decimal::Decimal,
}

// ==================== 收益訊息 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsRecordPayload {
    pub task_id: String,
    pub earnings: EarningsDetail,
    pub status: EarningStatus,
    pub estimated_payout_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsDetail {
    pub gpu_hours: f64,
    pub rate_usd_per_hour: rust_decimal::Decimal,
    pub amount_usd: rust_decimal::Decimal,
    pub bonus_multiplier: f32,
    pub final_amount_usd: rust_decimal::Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutNotificationPayload {
    pub payout_id: String,
    pub period: PayoutPeriod,
    pub summary: PayoutSummary,
    pub payment_method: String,
    pub payment_address: String,
    pub status: PayoutStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutPeriod {
    pub start: String,
    pub end: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutSummary {
    pub total_tasks: u32,
    pub total_gpu_hours: f64,
    pub gross_amount_usd: rust_decimal::Decimal,
    pub platform_fee_usd: rust_decimal::Decimal,
    pub net_amount_usd: rust_decimal::Decimal,
}

// ==================== 工作證明訊息 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowChallengePayload {
    pub challenge_id: String,
    pub nonce: String,
    pub difficulty: u32,
    pub deadline: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowResponsePayload {
    pub challenge_id: String,
    pub response: String,
    pub computation_time_ms: u32,
    pub gpu_signature: GpuSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuSignature {
    pub device_uuid: String,
    pub cuda_version: Option<String>,
}

// ==================== 錯誤訊息 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
    pub context: Option<serde_json::Value>,
    pub recoverable: bool,
}

// ==================== 狀態同步訊息 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSyncPayload {
    pub agent_id: String,
    pub last_heartbeat: DateTime<Utc>,
    pub active_tasks: Vec<ActiveTaskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTaskInfo {
    pub task_id: String,
    pub progress: f32,
    pub started_at: DateTime<Utc>,
}

// ==================== 訊息構建器 ====================

impl Message {
    /// 創建新訊息
    pub fn new(message_type: MessageType, payload: MessagePayload) -> Self {
        Self {
            message_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            message_type,
            payload,
        }
    }

    /// 序列化為 JSON
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// 從 JSON 反序列化
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

// ==================== 便捷構建函數 ====================

/// 創建認證響應訊息
pub fn create_auth_response(
    agent_id: String,
    signature: String,
    public_key: String,
) -> Message {
    Message::new(
        MessageType::AuthResponse,
        MessagePayload::AuthResponse(AuthResponsePayload {
            agent_id,
            signature,
            public_key,
        }),
    )
}

/// 創建 Agent 註冊訊息
pub fn create_agent_register(
    agent_id: String,
    hardware: HardwareInfo,
    capabilities: Capabilities,
    location: Location,
    availability: Availability,
) -> Message {
    Message::new(
        MessageType::AgentRegister,
        MessagePayload::AgentRegister(AgentRegisterPayload {
            agent_id,
            hardware,
            capabilities,
            location,
            availability,
        }),
    )
}

/// 創建心跳訊息
pub fn create_heartbeat(
    agent_id: String,
    status: AgentStatus,
    current_task_id: Option<String>,
    gpu_status: Vec<GPUStatus>,
    uptime_sec: u64,
) -> Message {
    Message::new(
        MessageType::Heartbeat,
        MessagePayload::Heartbeat(HeartbeatPayload {
            agent_id,
            status,
            current_task_id,
            gpu_status,
            uptime_sec,
            timestamp: Utc::now(),
        }),
    )
}

/// 創建任務接受訊息
pub fn create_task_accept(
    task_id: String,
    agent_id: String,
    gpu_allocated: u32,
    estimated_duration_sec: u32,
) -> Message {
    let estimated_completion = Utc::now() + chrono::Duration::seconds(estimated_duration_sec as i64);

    Message::new(
        MessageType::TaskAccept,
        MessagePayload::TaskAccept(TaskAcceptPayload {
            task_id,
            agent_id,
            gpu_allocated,
            estimated_completion,
        }),
    )
}

/// 創建任務拒絕訊息
pub fn create_task_reject(task_id: String, reason: String, details: String) -> Message {
    Message::new(
        MessageType::TaskReject,
        MessagePayload::TaskReject(TaskRejectPayload {
            task_id,
            reason,
            details,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = create_heartbeat(
            "agent-test-001".to_string(),
            AgentStatus::Idle,
            None,
            vec![],
            3600,
        );

        let json = msg.to_json().unwrap();
        println!("Serialized: {}", json);

        let deserialized = Message::from_json(&json).unwrap();
        assert_eq!(deserialized.message_type, MessageType::Heartbeat);
    }
}
