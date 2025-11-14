// 斷線重連策略

use tokio::time::{Duration, sleep};
use tracing::{info, warn};

/// 重連策略
pub struct ReconnectStrategy {
    max_retries: u32,
    base_delay_secs: u64,
    max_delay_secs: u64,
    current_attempt: u32,
}

impl ReconnectStrategy {
    /// 創建新的重連策略
    pub fn new() -> Self {
        Self {
            max_retries: 10,
            base_delay_secs: 1,
            max_delay_secs: 300, // 最多 5 分鐘
            current_attempt: 0,
        }
    }

    /// 重置重連計數
    pub fn reset(&mut self) {
        self.current_attempt = 0;
    }

    /// 獲取下一次重連的延遲時間
    pub fn next_delay(&mut self) -> Option<Duration> {
        if self.current_attempt >= self.max_retries {
            return None;
        }

        // 指數退避: delay = min(base * 2^attempt, max_delay)
        let delay_secs = (self.base_delay_secs * (2_u64.pow(self.current_attempt)))
            .min(self.max_delay_secs);

        self.current_attempt += 1;

        info!(
            "Reconnection attempt {}/{}, waiting {} seconds",
            self.current_attempt, self.max_retries, delay_secs
        );

        Some(Duration::from_secs(delay_secs))
    }

    /// 執行重連邏輯
    pub async fn retry<F, Fut, T, E>(&mut self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        loop {
            match operation().await {
                Ok(result) => {
                    self.reset();
                    return Ok(result);
                }
                Err(e) => {
                    warn!("Operation failed: {}", e);

                    if let Some(delay) = self.next_delay() {
                        sleep(delay).await;
                    } else {
                        warn!("Max retry attempts reached");
                        return Err(e);
                    }
                }
            }
        }
    }
}

impl Default for ReconnectStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reconnect_strategy() {
        let mut strategy = ReconnectStrategy::new();

        // 測試指數退避
        assert_eq!(strategy.next_delay(), Some(Duration::from_secs(1)));
        assert_eq!(strategy.next_delay(), Some(Duration::from_secs(2)));
        assert_eq!(strategy.next_delay(), Some(Duration::from_secs(4)));
        assert_eq!(strategy.next_delay(), Some(Duration::from_secs(8)));

        // 重置
        strategy.reset();
        assert_eq!(strategy.next_delay(), Some(Duration::from_secs(1)));
    }
}
