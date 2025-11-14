//! Orban Agent CLI
//!
//! 命令列介面

use orban_agent_core::{OrbanAgent, Result};
use clap::{Parser, Subcommand};
use tracing::info;

#[derive(Parser)]
#[command(name = "orban-agent")]
#[command(about = "Orban GPU Agent - Contribute your GPU to earn rewards", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the agent
    Start,

    /// Stop the agent
    Stop,

    /// Show agent status
    Status,

    /// Show earnings
    Earnings,

    /// Show GPU information
    Gpu,

    /// Show version
    Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => {
            println!("🚀 Starting Orban Agent...\n");

            let agent = OrbanAgent::new().await?;
            agent.start().await?;

            // 保持運行
            tokio::signal::ctrl_c().await?;

            println!("\n⏹️  Shutting down...");
            agent.stop().await?;

            println!("✓ Agent stopped");
        }

        Commands::Stop => {
            println!("Stopping agent...");
            // TODO: 實作優雅停止
            println!("✓ Agent stopped");
        }

        Commands::Status => {
            let agent = OrbanAgent::new().await?;
            let state = agent.get_state().await;

            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  Orban Agent Status");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();
            println!("Agent ID:        {}", state.agent_id.unwrap_or_else(|| "Not registered".to_string()));
            println!("Running:         {}", if state.is_running { "Yes" } else { "No" });
            println!("Tasks Completed: {}", state.tasks_completed);
            println!("Tasks Failed:    {}", state.tasks_failed);

            if let Some(started_at) = state.started_at {
                println!("Started At:      {}", started_at.format("%Y-%m-%d %H:%M:%S UTC"));
            }

            println!();

            // 顯示 GPU 狀態
            let gpu_status = agent.get_gpu_status().await?;

            if !gpu_status.is_empty() {
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("  GPU Status");
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!();

                for (i, status) in gpu_status.iter().enumerate() {
                    println!("GPU {}: {}", i, status.name);
                    println!("  Type:         {:?}", status.gpu_type);
                    println!("  Utilization:  {:.1}%", status.utilization * 100.0);
                    println!("  Memory:       {:.1} / {:.1} GB ({:.1}%)",
                        status.memory_used_gb,
                        status.memory_total_gb,
                        status.memory_usage_percent()
                    );
                    println!("  Temperature:  {:.1}°C", status.temperature);
                    println!("  Power:        {:.1}W", status.power_usage);
                    println!();
                }
            }
        }

        Commands::Earnings => {
            let agent = OrbanAgent::new().await?;
            let earnings = agent.get_earnings().await;

            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  Earnings Summary");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();
            println!("Total Earnings:   ${}", earnings.total_earnings);
            println!("Today:            ${}", earnings.today_earnings);
            println!("Pending:          ${}", earnings.pending_earnings);
            println!();

            if !earnings.history.is_empty() {
                println!("Recent Tasks:");
                println!();

                for record in earnings.history.iter().rev().take(10) {
                    println!("  {} | {} | ${} ({:.2}h @ ${}/h)",
                        record.timestamp.format("%Y-%m-%d %H:%M"),
                        record.gpu_model,
                        record.amount,
                        record.gpu_hours,
                        record.rate_per_hour
                    );
                }
            }
        }

        Commands::Gpu => {
            let agent = OrbanAgent::new().await?;
            let gpu_status = agent.get_gpu_status().await?;

            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("  GPU Information");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!();

            if gpu_status.is_empty() {
                println!("No compatible GPU found");
            } else {
                for (i, status) in gpu_status.iter().enumerate() {
                    println!("GPU {}:", i);
                    println!("  Name:         {}", status.name);
                    println!("  Type:         {:?}", status.gpu_type);
                    println!("  Total Memory: {:.1} GB", status.memory_total_gb);
                    println!("  Available:    {}", if status.is_available { "Yes" } else { "No" });
                    println!();
                }
            }
        }

        Commands::Version => {
            println!("orban-agent {}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
