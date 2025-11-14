// 收益追蹤器

use crate::types::{EarningRecord, EarningsData, EarningStatus};
use crate::network::orban_protocol::EarningsDetail;
use crate::error::Result;
use chrono::Utc;
use rust_decimal::Decimal;
use std::fs;
use std::path::PathBuf;
use tracing::info;

/// 收益追蹤器
pub struct EarningsTracker {
    data: EarningsData,
    storage_path: PathBuf,
}

impl EarningsTracker {
    /// 創建新的收益追蹤器
    pub fn new() -> Result<Self> {
        let storage_path = Self::get_storage_path()?;

        // 嘗試從文件加載
        let data = if storage_path.exists() {
            Self::load_from_file(&storage_path)?
        } else {
            EarningsData {
                total_earnings: Decimal::ZERO,
                today_earnings: Decimal::ZERO,
                pending_earnings: Decimal::ZERO,
                history: Vec::new(),
            }
        };

        Ok(Self {
            data,
            storage_path,
        })
    }

    /// 獲取存儲路徑
    fn get_storage_path() -> Result<PathBuf> {
        let mut path = dirs::data_local_dir()
            .ok_or_else(|| crate::error::Error::Unknown("Cannot find data directory".to_string()))?;

        path.push("orban-agent");
        fs::create_dir_all(&path)?;
        path.push("earnings.json");

        Ok(path)
    }

    /// 從文件加載
    fn load_from_file(path: &PathBuf) -> Result<EarningsData> {
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    /// 保存到文件
    fn save_to_file(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.storage_path, content)?;
        Ok(())
    }

    /// 記錄收益
    pub async fn record_earnings(&mut self, earnings: EarningsDetail) -> Result<()> {
        let record = EarningRecord {
            timestamp: Utc::now(),
            task_id: String::new(), // TODO: 從上下文獲取
            gpu_hours: earnings.gpu_hours,
            rate_per_hour: earnings.rate_usd_per_hour,
            amount: earnings.final_amount_usd,
            status: EarningStatus::Pending,
        };

        self.data.pending_earnings += record.amount;
        self.data.history.push(record);

        info!("Recorded earnings: ${}", earnings.final_amount_usd);

        self.save_to_file()?;
        Ok(())
    }

    /// 確認收益
    pub fn confirm_earnings(&mut self, task_id: &str) -> Result<()> {
        if let Some(record) = self.data.history.iter_mut().find(|r| r.task_id == task_id) {
            if record.status == EarningStatus::Pending {
                record.status = EarningStatus::Confirmed;
                self.data.pending_earnings -= record.amount;
                self.data.total_earnings += record.amount;

                self.save_to_file()?;
            }
        }

        Ok(())
    }

    /// 獲取收益數據
    pub fn get_data(&self) -> &EarningsData {
        &self.data
    }

    /// 更新今日收益
    pub fn update_today_earnings(&mut self) {
        let today = Utc::now().date_naive();

        let today_total: Decimal = self
            .data
            .history
            .iter()
            .filter(|r| r.timestamp.date_naive() == today)
            .map(|r| r.amount)
            .sum();

        self.data.today_earnings = today_total;
    }
}
