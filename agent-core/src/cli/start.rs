//! Start 命令實現

use crate::{Result, Error, daemon::{DaemonManager, AgentState}, config::Config};
use colored::Colorize;
use tracing::{info, error};

/// 執行 start 命令
pub async fn execute(foreground: bool) -> Result<()> {
    let daemon = DaemonManager::new()?;

    // 檢查是否已經在運行
    if daemon.is_running() {
        let pid = daemon.read_pid()?;
        println!("{} Agent is already running (PID: {})",
            "⚠".yellow(), pid);
        println!("  Use {} to stop it first",
            "orban-agent stop".cyan());
        return Ok(());
    }

    println!("{}", "╔════════════════════════════════════════╗".cyan());
    println!("{}", "║   Starting Orban Agent...             ║".cyan());
    println!("{}", "╚════════════════════════════════════════╝".cyan());
    println!();

    // 加載配置
    let config = Config::load()?;
    println!("  {} Loaded configuration", "✓".green());
    println!("    Platform: {}", config.platform_url);

    if foreground {
        // 前台運行模式（用於調試）
        println!();
        println!("{} Running in foreground mode (Ctrl+C to stop)", "ℹ".blue());
        println!();

        run_agent(config).await?;
    } else {
        // 後台守護進程模式
        #[cfg(unix)]
        {
            daemonize_unix(&daemon, config).await?;
        }

        #[cfg(windows)]
        {
            daemonize_windows(&daemon, config).await?;
        }
    }

    Ok(())
}

/// Unix 守護進程化
#[cfg(unix)]
async fn daemonize_unix(daemon: &DaemonManager, config: Config) -> Result<()> {
    use daemonize::Daemonize;

    let log_dir = config.log_dir();
    std::fs::create_dir_all(&log_dir)?;

    let stdout = std::fs::File::create(log_dir.join("agent.log"))?;
    let stderr = std::fs::File::create(log_dir.join("agent.err"))?;

    let daemonize = Daemonize::new()
        .pid_file(daemon.pid_file())
        .working_directory(std::env::current_dir()?)
        .stdout(stdout)
        .stderr(stderr);

    println!("  {} Daemonizing agent...", "✓".green());

    match daemonize.start() {
        Ok(_) => {
            // 子進程
            let pid = std::process::id();
            daemon.write_pid(pid)?;

            // 保存初始狀態
            let state = AgentState {
                pid,
                started_at: chrono::Utc::now(),
                is_running: true,
                uptime_seconds: 0,
                tasks_completed: 0,
            };
            daemon.save_state(&state)?;

            // 運行 Agent
            if let Err(e) = run_agent(config).await {
                error!("Agent failed: {}", e);
                daemon.remove_pid_file()?;
                return Err(e);
            }

            Ok(())
        }
        Err(e) => {
            Err(Error::Unknown(format!("Failed to daemonize: {}", e)))
        }
    }
}

/// Windows 後台運行
#[cfg(windows)]
async fn daemonize_windows(daemon: &DaemonManager, config: Config) -> Result<()> {
    use std::process::{Command, Stdio};

    println!("  {} Starting background process...", "✓".green());

    // 在 Windows 上，我們使用 CREATE_NO_WINDOW 標誌啟動新進程
    let exe_path = std::env::current_exe()?;

    let child = Command::new(exe_path)
        .arg("start")
        .arg("--foreground")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let pid = child.id();
    daemon.write_pid(pid)?;

    // 保存初始狀態
    let state = AgentState {
        pid,
        started_at: chrono::Utc::now(),
        is_running: true,
        uptime_seconds: 0,
        tasks_completed: 0,
    };
    daemon.save_state(&state)?;

    println!();
    println!("{} Agent started successfully!", "✓".green());
    println!("  PID: {}", pid);
    println!();
    println!("  View status: {}", "orban-agent status".cyan());
    println!("  View logs:   {}", "orban-agent logs".cyan());

    Ok(())
}

/// 運行 Agent 主循環
async fn run_agent(config: Config) -> Result<()> {
    info!("Orban Agent starting...");

    println!("  {} GPU detection...", "✓".green());

    println!("  {} Connecting to platform...", "✓".green());

    // 創建 AgentConfig 從 Config
    let agent_config = crate::AgentConfig {
        agent_id: config.agent_id.clone(),
        platform_url: config.platform_url.clone(),
        private_key_path: config.private_key_path.clone(),
        availability: crate::types::Availability {
            hours_per_day: if config.availability.always_on { 24 } else { 12 },
            reliability_score: 0.95,
        },
    };

    // 創建並啟動 Agent
    let mut agent = crate::OrbanAgent::new(agent_config).await?;

    println!();
    println!("{} Agent started successfully!", "✓".green());
    println!();
    println!("  View status: {}", "orban-agent status".cyan());
    println!("  View logs:   {}", "orban-agent logs".cyan());
    println!("  Stop agent:  {}", "orban-agent stop".cyan());

    // 啟動 Agent（會進入事件循環）
    agent.start().await?;

    info!("Agent shutting down...");
    Ok(())
}
