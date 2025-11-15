//! Stop 命令實現

use crate::{Result, daemon::DaemonManager};
use colored::Colorize;

/// 執行 stop 命令
pub async fn execute() -> Result<()> {
    let daemon = DaemonManager::new()?;

    println!("{}", "Stopping Orban Agent...".cyan().bold());
    println!();

    // 檢查是否在運行
    if !daemon.is_running() {
        println!("{} Agent is not running", "ℹ".blue());
        return Ok(());
    }

    let pid = daemon.read_pid()?;
    println!("  Found agent process (PID: {})", pid);

    // 停止 Agent
    print!("  Sending shutdown signal...");
    std::io::Write::flush(&mut std::io::stdout()).ok();

    match daemon.stop() {
        Ok(_) => {
            println!(" {}", "✓".green());
            println!();
            println!("{} Agent stopped successfully!", "✓".green());
        }
        Err(e) => {
            println!(" {}", "✗".red());
            println!();
            println!("{} Failed to stop agent: {}", "✗".red(), e);
            println!();
            println!("You may need to manually kill the process:");
            println!("  kill {}", pid);
            return Err(e);
        }
    }

    Ok(())
}
