//! Orban Agent CLI
//!
//! 命令行工具用於管理 Orban GPU Agent

use clap::{Parser, Subcommand};
use colored::Colorize;
use orban_agent_core::Result;
use std::process;

/// Orban Agent - GPU 算力貢獻工具
#[derive(Parser)]
#[command(name = "orban-agent")]
#[command(author = "Orban Team <dev@orban.ai>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Contribute your GPU, earn rewards", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 啟動 Agent (後台運行)
    Start {
        /// 前台運行模式（用於調試）
        #[arg(short, long)]
        foreground: bool,
    },

    /// 停止運行中的 Agent
    Stop,

    /// 顯示 Agent 運行狀態
    Status {
        /// 顯示詳細信息
        #[arg(short, long)]
        verbose: bool,
    },

    /// 顯示收益統計
    Earnings {
        /// 顯示歷史記錄
        #[arg(short, long)]
        history: bool,
    },

    /// 查看 Agent 日誌
    Logs {
        /// 持續追蹤日誌（類似 tail -f）
        #[arg(short, long)]
        follow: bool,

        /// 顯示最後 N 行
        #[arg(short, long, default_value = "50")]
        lines: usize,
    },

    /// 顯示版本信息
    Version,
}

#[tokio::main]
async fn main() {
    // 初始化日誌
    init_logging();

    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Start { foreground } => {
            orban_agent_core::cli::start::execute(foreground).await
        }
        Commands::Stop => {
            orban_agent_core::cli::stop::execute().await
        }
        Commands::Status { verbose } => {
            orban_agent_core::cli::status::execute(verbose).await
        }
        Commands::Earnings { history } => {
            orban_agent_core::cli::earnings::execute(history).await
        }
        Commands::Logs { follow, lines } => {
            orban_agent_core::cli::logs::execute(follow, lines).await
        }
        Commands::Version => {
            print_version();
            Ok(())
        }
    };

    match result {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("{} {}", "Error:".red().bold(), e);
            process::exit(1);
        }
    }
}

/// 初始化日誌系統
fn init_logging() {
    use tracing_subscriber::{fmt, EnvFilter};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .init();
}

/// 打印版本信息
fn print_version() {
    println!("{}", "╔════════════════════════════════════════╗".cyan());
    println!("{}", "║      Orban Agent - GPU Provider       ║".cyan());
    println!("{}", "╚════════════════════════════════════════╝".cyan());
    println!();
    println!("  {} {}", "Version:".bold(), env!("CARGO_PKG_VERSION"));
    println!("  {} {}", "Build:".bold(), get_build_info());
    println!("  {} {}", "Authors:".bold(), env!("CARGO_PKG_AUTHORS"));
    println!();
    println!("  {}", "https://orban.ai".dimmed());
}

/// 獲取構建信息
fn get_build_info() -> String {
    format!(
        "{} ({})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::ARCH
    )
}
