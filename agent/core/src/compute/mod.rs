//! 計算引擎模塊
//!
//! 從第一性原理思考任務執行：
//! 1. 資源準備：下載模型、資料
//! 2. 環境隔離：沙盒執行，防止惡意代碼
//! 3. GPU 計算：執行實際的推論或訓練
//! 4. 結果驗證：生成工作證明
//! 5. 清理資源：釋放 GPU 記憶體

mod executor;
mod sandbox;
mod verifier;
mod downloader;

pub use executor::TaskExecutor;
pub use sandbox::Sandbox;
pub use verifier::ProofGenerator;

use crate::Result;

/// 任務執行狀態
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ExecutionStatus {
    /// 準備中（下載資源）
    Preparing,
    /// 執行中
    Running,
    /// 已完成
    Completed,
    /// 失敗
    Failed(String),
}
