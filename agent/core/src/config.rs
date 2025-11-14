//! 配置管理模塊
//!
//! 從第一性原理思考：配置的本質是「可變的系統參數」
//!
//! 配置來源優先級：
//! 1. 環境變數（最高優先級）
//! 2. 配置檔案
//! 3. 預設值（最低優先級）

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::{Error, Result};

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Orban Platform URL
    pub platform_url: String,

    /// Agent 資料目錄
    pub data_dir: PathBuf,

    /// 最大併發任務數
    pub max_concurrent_tasks: usize,

    /// GPU 配置
    pub gpu: GpuConfig,

    /// 網路配置
    pub network: NetworkConfig,

    /// 資源限制
    pub limits: ResourceLimits,

    /// 收益配置
    pub earnings: EarningsConfig,
}

/// GPU 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    /// 是否自動偵測 GPU
    pub auto_detect: bool,

    /// 只使用指定的 GPU（device index）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_devices: Option<Vec<usize>>,

    /// 最大 GPU 使用率 (0.0-1.0)
    pub max_utilization: f32,

    /// 最大 VRAM 使用率 (0.0-1.0)
    pub max_vram_usage: f32,

    /// GPU 溫度上限（攝氏）
    pub max_temperature: f32,
}

/// 網路配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// 連接超時（秒）
    pub connect_timeout_secs: u64,

    /// 請求超時（秒）
    pub request_timeout_secs: u64,

    /// 心跳間隔（秒）
    pub heartbeat_interval_secs: u64,

    /// 最大重試次數
    pub max_retries: usize,

    /// 是否啟用 P2P
    pub enable_p2p: bool,
}

/// 資源限制
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// 最大磁碟使用量（GB）
    pub max_disk_usage_gb: f32,

    /// 最大記憶體使用量（GB）
    pub max_memory_gb: f32,

    /// 最大網路頻寬（Mbps）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_bandwidth_mbps: Option<f32>,

    /// 閒置時才執行任務
    pub idle_only: bool,
}

/// 收益配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsConfig {
    /// 最小支付門檻（USD）
    pub min_payout_threshold: f32,

    /// 支付地址（錢包地址或帳號）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payout_address: Option<String>,

    /// 是否自動複合收益
    pub auto_compound: bool,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = directories::ProjectDirs::from("ai", "orban", "agent")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".orban-agent"));

        Self {
            platform_url: "https://api.orban.ai".to_string(),
            data_dir,
            max_concurrent_tasks: 1,
            gpu: GpuConfig {
                auto_detect: true,
                use_devices: None,
                max_utilization: 0.9,
                max_vram_usage: 0.9,
                max_temperature: 85.0,
            },
            network: NetworkConfig {
                connect_timeout_secs: 30,
                request_timeout_secs: 300,
                heartbeat_interval_secs: 30,
                max_retries: 3,
                enable_p2p: true,
            },
            limits: ResourceLimits {
                max_disk_usage_gb: 100.0,
                max_memory_gb: 16.0,
                max_bandwidth_mbps: None,
                idle_only: false,
            },
            earnings: EarningsConfig {
                min_payout_threshold: 10.0,
                payout_address: None,
                auto_compound: false,
            },
        }
    }
}

impl Config {
    /// 載入配置
    ///
    /// 優先級：環境變數 > 配置檔案 > 預設值
    pub fn load() -> Result<Self> {
        let mut config = Config::default();

        // 確保資料目錄存在
        std::fs::create_dir_all(&config.data_dir)?;

        let config_file = config.data_dir.join("config.json");

        // 如果配置檔案存在，載入它
        if config_file.exists() {
            let contents = std::fs::read_to_string(&config_file)?;
            config = serde_json::from_str(&contents)?;
        } else {
            // 創建預設配置檔案
            config.save()?;
        }

        // 環境變數覆蓋
        if let Ok(url) = std::env::var("ORBAN_PLATFORM_URL") {
            config.platform_url = url;
        }

        if let Ok(data_dir) = std::env::var("ORBAN_DATA_DIR") {
            config.data_dir = PathBuf::from(data_dir);
        }

        Ok(config)
    }

    /// 儲存配置
    pub fn save(&self) -> Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;

        let config_file = self.data_dir.join("config.json");
        let contents = serde_json::to_string_pretty(self)?;
        std::fs::write(config_file, contents)?;

        Ok(())
    }

    /// 取得模型快取目錄
    pub fn model_cache_dir(&self) -> PathBuf {
        self.data_dir.join("models")
    }

    /// 取得資料集快取目錄
    pub fn dataset_cache_dir(&self) -> PathBuf {
        self.data_dir.join("datasets")
    }

    /// 取得結果輸出目錄
    pub fn output_dir(&self) -> PathBuf {
        self.data_dir.join("outputs")
    }

    /// 取得日誌目錄
    pub fn logs_dir(&self) -> PathBuf {
        self.data_dir.join("logs")
    }

    /// 取得收益資料檔案路徑
    pub fn earnings_file(&self) -> PathBuf {
        self.data_dir.join("earnings.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.max_concurrent_tasks, 1);
        assert!(config.gpu.auto_detect);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(config.platform_url, parsed.platform_url);
    }
}
