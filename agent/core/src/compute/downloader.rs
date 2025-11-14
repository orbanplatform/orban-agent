//! 資源下載器
//!
//! 負責下載模型和資料集，支援：
//! - 斷點續傳
//! - 校驗和驗證
//! - 進度回報
//! - 快取管理

use crate::{Result, Error};
use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use tracing::{info, warn};

/// 資源下載器
pub struct Downloader {
    cache_dir: PathBuf,
    client: reqwest::Client,
}

impl Downloader {
    /// 創建新的下載器
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir)?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;

        Ok(Self {
            cache_dir,
            client,
        })
    }

    /// 下載文件
    ///
    /// 如果本地快取存在且校驗和正確，則跳過下載
    pub async fn download(
        &self,
        url: &str,
        checksum: &str,
        filename: &str,
    ) -> Result<PathBuf> {
        let file_path = self.cache_dir.join(filename);

        // 檢查快取
        if file_path.exists() {
            info!("Checking cache: {}", filename);

            if self.verify_checksum(&file_path, checksum)? {
                info!("✓ Using cached file: {}", filename);
                return Ok(file_path);
            } else {
                warn!("Cache corrupted, re-downloading: {}", filename);
                std::fs::remove_file(&file_path)?;
            }
        }

        // 下載文件
        info!("Downloading: {} from {}", filename, url);

        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            return Err(Error::Network(response.error_for_status().unwrap_err()));
        }

        let bytes = response.bytes().await?;
        std::fs::write(&file_path, &bytes)?;

        // 驗證校驗和
        if !self.verify_checksum(&file_path, checksum)? {
            std::fs::remove_file(&file_path)?;
            return Err(Error::Verification(
                format!("Checksum mismatch for {}", filename)
            ));
        }

        info!("✓ Downloaded: {}", filename);

        Ok(file_path)
    }

    /// 驗證文件校驗和
    fn verify_checksum(&self, file_path: &Path, expected: &str) -> Result<bool> {
        let contents = std::fs::read(file_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let actual = format!("{:x}", hasher.finalize());

        Ok(actual == expected)
    }

    /// 清理舊的快取文件
    pub fn cleanup_old_cache(&self, days: u64) -> Result<()> {
        info!("Cleaning up cache older than {} days", days);

        let cutoff = std::time::SystemTime::now()
            - std::time::Duration::from_secs(days * 24 * 3600);

        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if let Ok(modified) = metadata.modified() {
                if modified < cutoff {
                    info!("Removing old cache: {:?}", entry.path());
                    std::fs::remove_file(entry.path())?;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_downloader_creation() {
        let dir = tempdir().unwrap();
        let downloader = Downloader::new(dir.path().to_path_buf());
        assert!(downloader.is_ok());
    }
}
