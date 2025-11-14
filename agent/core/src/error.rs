//! 錯誤類型定義
//!
//! 使用 thiserror 提供清晰的錯誤訊息和類型

use thiserror::Error;

/// Agent 核心錯誤類型
#[derive(Error, Debug)]
pub enum Error {
    /// GPU 相關錯誤
    #[error("GPU error: {0}")]
    Gpu(String),

    /// GPU 未找到
    #[error("No compatible GPU found")]
    NoGpuFound,

    /// 網路錯誤
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// WebSocket 錯誤
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// 序列化錯誤
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO 錯誤
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 配置錯誤
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    /// 任務執行錯誤
    #[error("Task execution failed: {0}")]
    TaskExecution(String),

    /// 驗證錯誤
    #[error("Verification failed: {0}")]
    Verification(String),

    /// 未註冊到平台
    #[error("Agent not registered to platform")]
    NotRegistered,

    /// 無效的任務
    #[error("Invalid task: {0}")]
    InvalidTask(String),

    /// 容器運行時錯誤
    #[cfg(feature = "docker")]
    #[error("Container runtime error: {0}")]
    Container(#[from] bollard::errors::Error),

    /// 其他錯誤
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Result 類型別名
pub type Result<T> = std::result::Result<T, Error>;
