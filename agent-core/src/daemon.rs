//! 守護進程管理模塊

use crate::{Result, Error};
use std::fs;
use std::path::PathBuf;
use std::io::{Read, Write};
use tracing::{info, warn, error};

/// 守護進程管理器
pub struct DaemonManager {
    pid_file: PathBuf,
    state_file: PathBuf,
}

/// Agent 運行狀態
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentState {
    pub pid: u32,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub is_running: bool,
    pub uptime_seconds: u64,
    pub tasks_completed: u32,
}

impl DaemonManager {
    /// 創建新的守護進程管理器
    pub fn new() -> Result<Self> {
        let data_dir = crate::config::Config::default().data_dir;
        fs::create_dir_all(&data_dir)?;

        Ok(Self {
            pid_file: data_dir.join("orban-agent.pid"),
            state_file: data_dir.join("state.json"),
        })
    }

    /// 獲取 PID 文件路徑
    pub fn pid_file(&self) -> &PathBuf {
        &self.pid_file
    }

    /// 獲取狀態文件路徑
    pub fn state_file(&self) -> &PathBuf {
        &self.state_file
    }

    /// 寫入 PID 文件
    pub fn write_pid(&self, pid: u32) -> Result<()> {
        let mut file = fs::File::create(&self.pid_file)?;
        file.write_all(pid.to_string().as_bytes())?;
        info!("PID {} written to {:?}", pid, self.pid_file);
        Ok(())
    }

    /// 讀取 PID 文件
    pub fn read_pid(&self) -> Result<u32> {
        if !self.pid_file.exists() {
            return Err(Error::Unknown("PID file not found".to_string()));
        }

        let mut file = fs::File::open(&self.pid_file)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        content.trim().parse::<u32>()
            .map_err(|e| Error::Unknown(format!("Invalid PID: {}", e)))
    }

    /// 刪除 PID 文件
    pub fn remove_pid_file(&self) -> Result<()> {
        if self.pid_file.exists() {
            fs::remove_file(&self.pid_file)?;
            info!("PID file removed");
        }
        Ok(())
    }

    /// 檢查進程是否在運行
    pub fn is_running(&self) -> bool {
        match self.read_pid() {
            Ok(pid) => self.is_process_running(pid),
            Err(_) => false,
        }
    }

    /// 檢查指定 PID 的進程是否在運行
    #[cfg(unix)]
    fn is_process_running(&self, pid: u32) -> bool {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        match kill(Pid::from_raw(pid as i32), None) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    #[cfg(windows)]
    fn is_process_running(&self, pid: u32) -> bool {
        // Windows 實現
        use std::process::Command;

        Command::new("tasklist")
            .args(&["/FI", &format!("PID eq {}", pid)])
            .output()
            .map(|output| {
                String::from_utf8_lossy(&output.stdout)
                    .contains(&pid.to_string())
            })
            .unwrap_or(false)
    }

    /// 停止守護進程
    #[cfg(unix)]
    pub fn stop(&self) -> Result<()> {
        let pid = self.read_pid()?;

        if !self.is_process_running(pid) {
            warn!("Process {} is not running", pid);
            self.remove_pid_file()?;
            return Ok(());
        }

        info!("Sending SIGTERM to process {}", pid);

        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        kill(Pid::from_raw(pid as i32), Signal::SIGTERM)
            .map_err(|e| Error::Unknown(format!("Failed to send signal: {}", e)))?;

        // 等待進程退出
        for _ in 0..30 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if !self.is_process_running(pid) {
                info!("Process {} stopped successfully", pid);
                self.remove_pid_file()?;
                return Ok(());
            }
        }

        // 如果還沒停止，發送 SIGKILL
        warn!("Process {} did not stop gracefully, sending SIGKILL", pid);
        kill(Pid::from_raw(pid as i32), Signal::SIGKILL)
            .map_err(|e| Error::Unknown(format!("Failed to kill process: {}", e)))?;

        self.remove_pid_file()?;
        Ok(())
    }

    #[cfg(windows)]
    pub fn stop(&self) -> Result<()> {
        let pid = self.read_pid()?;

        if !self.is_process_running(pid) {
            warn!("Process {} is not running", pid);
            self.remove_pid_file()?;
            return Ok(());
        }

        info!("Terminating process {}", pid);

        use std::process::Command;
        Command::new("taskkill")
            .args(&["/PID", &pid.to_string(), "/F"])
            .output()
            .map_err(|e| Error::Unknown(format!("Failed to kill process: {}", e)))?;

        self.remove_pid_file()?;
        Ok(())
    }

    /// 保存 Agent 狀態
    pub fn save_state(&self, state: &AgentState) -> Result<()> {
        let content = serde_json::to_string_pretty(state)?;
        fs::write(&self.state_file, content)?;
        Ok(())
    }

    /// 加載 Agent 狀態
    pub fn load_state(&self) -> Result<AgentState> {
        if !self.state_file.exists() {
            return Err(Error::Unknown("State file not found".to_string()));
        }

        let content = fs::read_to_string(&self.state_file)?;
        let state: AgentState = serde_json::from_str(&content)?;
        Ok(state)
    }

    /// 更新狀態（如果存在）
    pub fn update_state<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut AgentState),
    {
        let mut state = self.load_state().unwrap_or_else(|_| AgentState {
            pid: std::process::id(),
            started_at: chrono::Utc::now(),
            is_running: true,
            uptime_seconds: 0,
            tasks_completed: 0,
        });

        f(&mut state);
        self.save_state(&state)?;
        Ok(())
    }
}

impl Default for DaemonManager {
    fn default() -> Self {
        Self::new().expect("Failed to create DaemonManager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_manager_creation() {
        let manager = DaemonManager::new().unwrap();
        assert!(manager.pid_file.ends_with("orban-agent.pid"));
    }

    #[test]
    fn test_is_process_running_self() {
        let manager = DaemonManager::new().unwrap();
        let self_pid = std::process::id();
        assert!(manager.is_process_running(self_pid));
    }

    #[test]
    fn test_is_process_running_invalid() {
        let manager = DaemonManager::new().unwrap();
        // PID 1 unlikely to be our process, but might exist
        // Use a very high PID that's unlikely to exist
        assert!(!manager.is_process_running(9999999));
    }
}
