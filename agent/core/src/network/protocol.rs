//! 網路協議定義
//!
//! Agent 與 Platform 通訊的資料結構

use serde::{Deserialize, Serialize};
use crate::types::GpuInfo;
use rust_decimal::Decimal;

/// Agent 註冊請求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationRequest {
    /// 主機名稱
    pub hostname: String,

    /// GPU 資訊列表
    pub gpus: Vec<GpuInfo>,

    /// Agent 版本
    pub version: String,
}

/// Agent 註冊回應
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    /// 分配的 Agent ID
    pub agent_id: String,

    /// 註冊訊息
    pub message: String,
}

/// 收益資訊回應
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsResponse {
    /// 總收益（USD）
    pub total_usd: Decimal,

    /// 今日收益
    pub today_usd: Decimal,

    /// 待確認收益
    pub pending_usd: Decimal,

    /// 已完成任務數
    pub tasks_completed: u64,
}
