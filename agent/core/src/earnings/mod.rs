//! 收益追蹤模塊
//!
//! 追蹤使用者的 GPU 貢獻收益

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Datelike};
use rust_decimal::Decimal;
use std::collections::HashMap;
use crate::{Result, config::Config, types::{TaskResult, EarningStatus}};

/// 收益記錄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningRecord {
    /// 任務 ID
    pub task_id: String,

    /// GPU 型號
    pub gpu_model: String,

    /// GPU 執行時間（小時）
    pub gpu_hours: f64,

    /// 費率（USD per hour）
    pub rate_per_hour: Decimal,

    /// 收益金額（USD）
    pub amount: Decimal,

    /// 狀態
    pub status: EarningStatus,

    /// 時間戳
    pub timestamp: DateTime<Utc>,
}

impl EarningRecord {
    /// 從任務結果創建收益記錄
    pub fn from_task_result(result: &TaskResult) -> Self {
        let gpu_hours = result.gpu_time_seconds / 3600.0;

        // 根據 GPU 型號決定費率
        let rate_per_hour = Self::calculate_rate(&result.gpu_used.model);

        let amount = rate_per_hour * Decimal::from_f64_retain(gpu_hours).unwrap_or_default();

        Self {
            task_id: result.task_id.clone(),
            gpu_model: result.gpu_used.model.clone(),
            gpu_hours,
            rate_per_hour,
            amount,
            status: EarningStatus::Pending,
            timestamp: result.completed_at,
        }
    }

    /// 計算費率
    fn calculate_rate(gpu_model: &str) -> Decimal {
        // 基礎費率：每 GPU 小時 $0.01
        let base_rate = Decimal::from_str_exact("0.01").unwrap();

        // 根據 GPU 型號調整倍數
        let multiplier = if gpu_model.contains("4090") {
            Decimal::from_str_exact("2.5").unwrap()
        } else if gpu_model.contains("3090") {
            Decimal::from_str_exact("1.8").unwrap()
        } else if gpu_model.contains("A100") {
            Decimal::from_str_exact("5.0").unwrap()
        } else if gpu_model.contains("H100") {
            Decimal::from_str_exact("8.0").unwrap()
        } else {
            Decimal::from_str_exact("1.0").unwrap()
        };

        base_rate * multiplier
    }
}

/// 收益資料（用於 UI 顯示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsData {
    /// 總收益
    pub total_earnings: Decimal,

    /// 今日收益
    pub today_earnings: Decimal,

    /// 待確認收益
    pub pending_earnings: Decimal,

    /// 歷史記錄
    pub history: Vec<EarningRecord>,

    /// 每日統計（日期 -> 收益）
    pub daily_stats: HashMap<String, Decimal>,
}

/// 收益追蹤器
pub struct EarningsTracker {
    records: Vec<EarningRecord>,
}

impl EarningsTracker {
    /// 創建新的追蹤器
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// 載入收益資料
    pub fn load(config: &Config) -> Result<Self> {
        let file_path = config.earnings_file();

        if !file_path.exists() {
            return Ok(Self::new());
        }

        let contents = std::fs::read_to_string(&file_path)?;
        let records: Vec<EarningRecord> = serde_json::from_str(&contents)?;

        Ok(Self { records })
    }

    /// 儲存收益資料
    pub fn save(&self, config: &Config) -> Result<()> {
        let file_path = config.earnings_file();
        let contents = serde_json::to_string_pretty(&self.records)?;
        std::fs::write(file_path, contents)?;

        Ok(())
    }

    /// 添加收益記錄
    pub fn add_earnings(&mut self, record: EarningRecord) {
        self.records.push(record);
    }

    /// 取得收益資料
    pub fn get_data(&self) -> EarningsData {
        let total_earnings: Decimal = self.records.iter()
            .map(|r| r.amount)
            .sum();

        let today = Utc::now().date_naive();
        let today_earnings: Decimal = self.records.iter()
            .filter(|r| r.timestamp.date_naive() == today)
            .map(|r| r.amount)
            .sum();

        let pending_earnings: Decimal = self.records.iter()
            .filter(|r| r.status == EarningStatus::Pending)
            .map(|r| r.amount)
            .sum();

        // 計算每日統計
        let mut daily_stats: HashMap<String, Decimal> = HashMap::new();
        for record in &self.records {
            let date = record.timestamp.date_naive().to_string();
            *daily_stats.entry(date).or_insert(Decimal::ZERO) += record.amount;
        }

        EarningsData {
            total_earnings,
            today_earnings,
            pending_earnings,
            history: self.records.clone(),
            daily_stats,
        }
    }

    /// 取得最近 N 筆記錄
    pub fn get_recent(&self, limit: usize) -> Vec<EarningRecord> {
        self.records.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}

impl Default for EarningsTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_earning_record_creation() {
        let gpu_hours = 1.5;
        let rate = Decimal::from_str_exact("0.025").unwrap();
        let expected_amount = rate * Decimal::from_f64_retain(gpu_hours).unwrap();

        assert!(expected_amount > Decimal::ZERO);
    }

    #[test]
    fn test_earnings_tracker() {
        let mut tracker = EarningsTracker::new();
        assert_eq!(tracker.records.len(), 0);

        let record = EarningRecord {
            task_id: "task-1".to_string(),
            gpu_model: "RTX 4090".to_string(),
            gpu_hours: 1.0,
            rate_per_hour: Decimal::from_str_exact("0.025").unwrap(),
            amount: Decimal::from_str_exact("0.025").unwrap(),
            status: EarningStatus::Pending,
            timestamp: Utc::now(),
        };

        tracker.add_earnings(record);
        assert_eq!(tracker.records.len(), 1);

        let data = tracker.get_data();
        assert!(data.total_earnings > Decimal::ZERO);
    }
}
