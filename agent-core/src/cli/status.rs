//! Status 命令實現

use crate::{Result, daemon::DaemonManager, earnings::EarningsTracker, gpu::GPUDetector};
use colored::Colorize;
use chrono::Utc;

/// 執行 status 命令
pub async fn execute(verbose: bool) -> Result<()> {
    let daemon = DaemonManager::new()?;

    println!("{}", "╔════════════════════════════════════════╗".cyan());
    println!("{}", "║      Orban Agent Status Report        ║".cyan());
    println!("{}", "╚════════════════════════════════════════╝".cyan());
    println!();

    // 運行狀態
    let is_running = daemon.is_running();
    print_section("Agent Status");

    if is_running {
        let pid = daemon.read_pid()?;
        let state = daemon.load_state().ok();

        println!("  {} {}", "Status:".bold(), "Running".green());
        println!("  {} {}", "PID:".bold(), pid);

        if let Some(state) = state {
            let uptime = Utc::now()
                .signed_duration_since(state.started_at)
                .num_seconds();
            println!("  {} {}", "Uptime:".bold(), format_duration(uptime as u64));
            println!("  {} {}", "Started:".bold(), state.started_at.format("%Y-%m-%d %H:%M:%S UTC"));

            if verbose {
                println!("  {} {}", "Tasks Completed:".bold(), state.tasks_completed);
            }
        }
    } else {
        println!("  {} {}", "Status:".bold(), "Stopped".red());
        println!();
        println!("  Start with: {}", "orban-agent start".cyan());
    }

    println!();

    // GPU 信息
    print_section("GPU Information");
    print_gpu_info(verbose)?;
    println!();

    // 收益摘要
    if let Ok(tracker) = EarningsTracker::new() {
        print_section("Earnings Summary");

        let data = tracker.get_data();
        println!("  {} ${:.4}", "Total Earnings:".bold(), data.total_earnings);
        println!("  {} ${:.4}", "Today:".bold(), data.today_earnings);
        println!("  {} ${:.4}", "Pending:".bold(), data.pending_earnings);

        if verbose {
            println!("  {} {}", "Records:".bold(), data.history.len());
        }

        println!();
        println!("  View details: {}", "orban-agent earnings".cyan());
    }

    Ok(())
}

/// 打印章節標題
fn print_section(title: &str) {
    println!("{}", format!("─── {} ───", title).dimmed());
    println!();
}

/// 格式化時長
fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// 打印 GPU 信息
fn print_gpu_info(verbose: bool) -> Result<()> {
    match GPUDetector::detect_all() {
        Ok(detector) => {
            let device_count = detector.device_count();

            if device_count == 0 {
                println!("  {} No GPU detected", "⚠".yellow());
                return Ok(());
            }

            println!("  {} {} GPU(s) detected", "✓".green(), device_count);
            println!();

            // 獲取所有設備並打印信息
            for device in detector.get_all_devices() {
                let index = device.index();
                let name = device.name().unwrap_or_else(|_| "Unknown GPU".to_string());
                let vendor = device.vendor();

                println!("  {} {} ({:?})",
                    format!("GPU {}:", index).bold().cyan(),
                    name,
                    vendor
                );

                // 打印 VRAM
                if let Ok(memory) = device.memory_info() {
                    println!("    {} {:.1} GB", "VRAM:".dimmed(), memory.total_gb());
                }

                // 如果 verbose 模式，顯示更多詳細信息
                if verbose {
                    if let Ok(status) = device.get_status() {
                        println!("    {} {:.0}%", "Utilization:".dimmed(), status.utilization);
                        println!("    {} {:.1} / {:.1} GB",
                            "Memory Used:".dimmed(),
                            status.memory_used_gb,
                            status.memory_total_gb
                        );
                        println!("    {} {:.1}°C", "Temperature:".dimmed(), status.temperature_c);
                        println!("    {} {:.1} W", "Power Draw:".dimmed(), status.power_draw_w);

                        if status.fan_speed_percent > 0.0 {
                            println!("    {} {:.0}%", "Fan Speed:".dimmed(), status.fan_speed_percent);
                        }
                    }

                    // 打印計算能力
                    if let Ok(compute_cap) = device.compute_capability() {
                        println!("    {} {}", "Compute:".dimmed(), compute_cap);
                    }

                    // 打印 PCIe 頻寬
                    if let Ok(pcie_bw) = device.pcie_bandwidth() {
                        println!("    {} {} GB/s", "PCIe:".dimmed(), pcie_bw);
                    }

                    // 打印 CUDA 核心數 (如果有)
                    if let Some(cuda_cores) = device.cuda_cores() {
                        println!("    {} {}", "CUDA Cores:".dimmed(), cuda_cores);
                    }
                }

                println!();
            }
        }
        Err(e) => {
            println!("  {} Failed to detect GPUs: {}", "✗".red(), e);
            println!("  {}", "Make sure GPU drivers are installed".dimmed());
        }
    }

    Ok(())
}
