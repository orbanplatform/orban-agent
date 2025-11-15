use thiserror::Error;

/// Orban Agent 錯誤類型
#[derive(Error, Debug)]
pub enum Error {
    // GPU 相關錯誤
    #[error("No GPU found")]
    GPUNotFound,

    #[error("GPU error: {0}")]
    GPUError(String),

    #[error("Insufficient VRAM: required {required}GB, available {available}GB")]
    InsufficientVRAM { required: u32, available: u32 },

    #[cfg(feature = "nvidia")]
    #[error("NVML error: {0}")]
    NVMLError(#[from] nvml_wrapper::error::NvmlError),

    // 網路相關錯誤
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("HTTP error: {0}")]
    HTTPError(#[from] reqwest::Error),

    // 任務相關錯誤
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Upload failed: {0}")]
    UploadFailed(String),

    #[error("Task timeout")]
    TaskTimeout,

    #[error("Out of memory")]
    OutOfMemory,

    // 加密相關錯誤
    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    // 配置相關錯誤
    #[error("Configuration error: {0}")]
    ConfigError(#[from] config::ConfigError),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    // I/O 錯誤
    #[error("I/O error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("File not found: {0}")]
    FileNotFound(String),

    // 序列化錯誤
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    // Agent 狀態錯誤
    #[error("Agent not registered")]
    NotRegistered,

    // 其他錯誤
    #[error("Unknown error: {0}")]
    Unknown(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Result 類型別名
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// 判斷錯誤是否可恢復
    pub fn is_recoverable(&self) -> bool {
        match self {
            Error::ConnectionFailed(_) => true,
            Error::DownloadFailed(_) => true,
            Error::TaskTimeout => true,
            Error::GPUNotFound => false,
            Error::InsufficientVRAM { .. } => false,
            Error::AuthenticationFailed(_) => false,
            _ => false,
        }
    }

    /// 獲取錯誤碼
    pub fn error_code(&self) -> &'static str {
        match self {
            Error::GPUNotFound => "GPU_NOT_AVAILABLE",
            Error::InsufficientVRAM { .. } => "INSUFFICIENT_VRAM",
            Error::GPUError(_) => "GPU_ERROR",
            Error::ConnectionFailed(_) => "CONNECTION_FAILED",
            Error::AuthenticationFailed(_) => "AUTH_FAILED",
            Error::TaskExecutionFailed(_) => "TASK_EXECUTION_FAILED",
            Error::DownloadFailed(_) => "DOWNLOAD_FAILED",
            Error::UploadFailed(_) => "UPLOAD_FAILED",
            Error::TaskTimeout => "TIMEOUT",
            Error::OutOfMemory => "OOM_ERROR",
            Error::SignatureVerificationFailed => "VALIDATION_FAILED",
            _ => "UNKNOWN_ERROR",
        }
    }
}
