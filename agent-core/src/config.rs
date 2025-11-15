//! 配置管理模塊

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::Result;

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 平台 URL
    pub platform_url: String,

    /// Agent 數據目錄
    pub data_dir: PathBuf,

    /// 日誌級別
    pub log_level: String,

    /// GPU 配置
    pub gpu: GpuConfig,

    /// 網路配置
    pub network: NetworkConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuConfig {
    /// 最大並發任務數
    pub max_concurrent_tasks: usize,

    /// 保留的 VRAM (GB)
    pub reserved_vram_gb: f32,

    /// 允許的 GPU 索引（None 表示所有）
    pub allowed_gpu_indices: Option<Vec<usize>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// 心跳間隔（秒）
    pub heartbeat_interval_secs: u64,

    /// 連接超時（秒）
    pub connection_timeout_secs: u64,

    /// 重試次數
    pub max_retries: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            platform_url: "https://platform.orban.ai".to_string(),
            data_dir: Self::default_data_dir(),
            log_level: "info".to_string(),
            gpu: GpuConfig::default(),
            network: NetworkConfig::default(),
        }
    }
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 1,
            reserved_vram_gb: 2.0,
            allowed_gpu_indices: None,
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval_secs: 30,
            connection_timeout_secs: 10,
            max_retries: 3,
        }
    }
}

impl Config {
    /// 載入配置
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path();

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)
                .map_err(|e| crate::Error::InvalidConfig(e.to_string()))?;
            Ok(config)
        } else {
            // 如果配置文件不存在，使用默認配置並保存
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// 保存配置
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_file_path();

        // 確保目錄存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::Error::InvalidConfig(e.to_string()))?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    /// 獲取配置文件路徑
    fn config_file_path() -> PathBuf {
        let data_dir = Self::default_data_dir();
        data_dir.join("config.toml")
    }

    /// 獲取默認數據目錄
    fn default_data_dir() -> PathBuf {
        directories::ProjectDirs::from("ai", "orban", "agent")
            .map(|dirs| dirs.data_dir().to_path_buf())
            .unwrap_or_else(|| {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".orban-agent")
            })
    }

    /// 獲取收益文件路徑
    pub fn earnings_file(&self) -> PathBuf {
        self.data_dir.join("earnings.json")
    }

    /// 獲取狀態文件路徑
    pub fn state_file(&self) -> PathBuf {
        self.data_dir.join("state.json")
    }

    /// 獲取日誌目錄
    pub fn log_dir(&self) -> PathBuf {
        self.data_dir.join("logs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.platform_url, "https://platform.orban.ai");
        assert_eq!(config.gpu.max_concurrent_tasks, 1);
        assert_eq!(config.network.heartbeat_interval_secs, 30);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(config.platform_url, deserialized.platform_url);
    }
}
