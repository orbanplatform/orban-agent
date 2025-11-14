//! GPU 監控器
//!
//! 即時監控 GPU 狀態，提供歷史資料和趨勢分析

use super::{GPUDevice, GPUStatus};
use std::sync::Arc;
use std::collections::VecDeque;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// GPU 監控器
///
/// 定期收集 GPU 狀態資料，並提供歷史查詢
pub struct GPUMonitor {
    devices: Vec<Arc<dyn GPUDevice>>,
    history: Arc<RwLock<VecDeque<GPUSnapshot>>>,
    max_history_size: usize,
}

/// GPU 快照（某個時間點的狀態）
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GPUSnapshot {
    pub timestamp: DateTime<Utc>,
    pub statuses: Vec<GPUStatus>,
}

impl GPUMonitor {
    /// 創建新的監控器
    pub fn new(devices: Vec<Arc<dyn GPUDevice>>) -> Self {
        Self {
            devices,
            history: Arc::new(RwLock::new(VecDeque::new())),
            max_history_size: 1000, // 保留最近 1000 個快照
        }
    }

    /// 開始監控（背景任務）
    ///
    /// 每隔 interval 秒收集一次 GPU 狀態
    pub fn start_monitoring(&self, interval_secs: u64) {
        let devices = self.devices.clone();
        let history = self.history.clone();
        let max_size = self.max_history_size;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                std::time::Duration::from_secs(interval_secs)
            );

            loop {
                interval.tick().await;

                // 收集當前狀態
                let mut statuses = Vec::new();
                for device in &devices {
                    if let Ok(status) = device.get_status() {
                        statuses.push(status);
                    }
                }

                let snapshot = GPUSnapshot {
                    timestamp: Utc::now(),
                    statuses,
                };

                // 加入歷史記錄
                let mut hist = history.write().await;
                hist.push_back(snapshot);

                // 限制歷史大小
                while hist.len() > max_size {
                    hist.pop_front();
                }
            }
        });
    }

    /// 取得當前狀態
    pub async fn current_status(&self) -> Vec<GPUStatus> {
        let mut statuses = Vec::new();
        for device in &self.devices {
            if let Ok(status) = device.get_status() {
                statuses.push(status);
            }
        }
        statuses
    }

    /// 取得歷史快照
    pub async fn get_history(&self, limit: usize) -> Vec<GPUSnapshot> {
        let hist = self.history.read().await;
        hist.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// 計算平均使用率（最近 N 個快照）
    pub async fn average_utilization(&self, last_n: usize) -> f32 {
        let hist = self.history.read().await;
        let snapshots: Vec<_> = hist.iter().rev().take(last_n).collect();

        if snapshots.is_empty() {
            return 0.0;
        }

        let mut total = 0.0;
        let mut count = 0;

        for snapshot in snapshots {
            for status in &snapshot.statuses {
                total += status.utilization;
                count += 1;
            }
        }

        if count > 0 {
            total / count as f32
        } else {
            0.0
        }
    }

    /// 檢查是否有任何 GPU 過熱
    pub async fn is_any_overheating(&self) -> bool {
        let statuses = self.current_status().await;
        statuses.iter().any(|s| s.is_overheating())
    }

    /// 檢查所有 GPU 是否都閒置
    pub async fn are_all_idle(&self) -> bool {
        let statuses = self.current_status().await;
        !statuses.is_empty() && statuses.iter().all(|s| s.is_idle())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitor_creation() {
        let devices: Vec<Arc<dyn GPUDevice>> = Vec::new();
        let monitor = GPUMonitor::new(devices);
        assert_eq!(monitor.max_history_size, 1000);
    }
}
