//! Earnings 命令實現

use crate::{Result, earnings::EarningsTracker, types::EarningStatus};
use colored::Colorize;
use chrono::Utc;

/// 執行 earnings 命令
pub async fn execute(show_history: bool) -> Result<()> {
    let mut tracker = EarningsTracker::new()?;

    // 更新今日收益
    tracker.update_today_earnings();

    let data = tracker.get_data();

    println!("{}", "╔════════════════════════════════════════╗".cyan());
    println!("{}", "║        Earnings Dashboard             ║".cyan());
    println!("{}", "╚════════════════════════════════════════╝".cyan());
    println!();

    // 摘要統計
    print_summary(&data.total_earnings.to_string(), &data.today_earnings.to_string(), &data.pending_earnings.to_string());
    println!();

    // 歷史記錄
    if show_history {
        print_history(&data.history);
    } else {
        // 只顯示最近的記錄
        if !data.history.is_empty() {
            print_section("Recent Earnings");
            let recent: Vec<_> = data.history.iter().rev().take(10).collect();
            print_earnings_table(&recent);
            println!();

            if data.history.len() > 10 {
                println!("  {} Showing 10 of {} records",
                    "ℹ".blue(),
                    data.history.len()
                );
                println!("  Use {} to view all records",
                    "orban-agent earnings --history".cyan()
                );
            }
        } else {
            println!("  {} No earnings yet", "ℹ".blue());
            println!("  Start the agent to begin earning rewards");
        }
    }

    Ok(())
}

/// 打印摘要統計
fn print_summary(total: &str, today: &str, pending: &str) {
    print_section("Summary");

    // Total Earnings
    println!("  {} {}",
        "Total Earnings:".bold(),
        format!("${}", total).green().bold()
    );

    // Today's Earnings
    println!("  {} {}",
        "Today:".bold(),
        format!("${}", today).cyan()
    );

    // Pending
    println!("  {} {}",
        "Pending:".bold(),
        format!("${}", pending).yellow()
    );
}

/// 打印歷史記錄
fn print_history(records: &[crate::types::EarningRecord]) {
    if records.is_empty() {
        println!("  {} No earnings history", "ℹ".blue());
        return;
    }

    print_section(&format!("All Earnings ({} records)", records.len()));

    // 按日期分組
    let mut by_date: std::collections::HashMap<String, Vec<&crate::types::EarningRecord>> =
        std::collections::HashMap::new();

    for record in records.iter().rev() {
        let date = record.timestamp.format("%Y-%m-%d").to_string();
        by_date.entry(date).or_insert_with(Vec::new).push(record);
    }

    // 按日期排序
    let mut dates: Vec<String> = by_date.keys().cloned().collect();
    dates.sort_by(|a, b| b.cmp(a)); // 降序

    for date in dates {
        let day_records = &by_date[&date];
        let day_total: rust_decimal::Decimal = day_records
            .iter()
            .map(|r| r.amount)
            .sum();

        println!();
        println!("  {} {} ({})",
            "📅".dimmed(),
            date.bold(),
            format!("${:.4}", day_total).green()
        );
        println!("  {}", "─".repeat(50).dimmed());

        print_earnings_table(day_records);
    }
}

/// 打印收益表格
fn print_earnings_table(records: &[&crate::types::EarningRecord]) {
    for record in records {
        let time = record.timestamp.format("%H:%M:%S");
        let status_str = format_status(record.status);
        let amount = format!("${:.4}", record.amount);

        println!("    {} {} {} {} {}",
            time.to_string().dimmed(),
            format!("Task: {}", truncate(&record.task_id, 8)).cyan(),
            format!("{:.2}h", record.gpu_hours).dimmed(),
            status_str,
            amount.green()
        );
    }
}

/// 格式化狀態
fn format_status(status: EarningStatus) -> colored::ColoredString {
    match status {
        EarningStatus::Pending => "PENDING".yellow(),
        EarningStatus::Confirmed => "CONFIRMED".green(),
        EarningStatus::Paid => "PAID".bright_green().bold(),
    }
}

/// 截斷字符串
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

/// 打印章節標題
fn print_section(title: &str) {
    println!("{}", format!("─── {} ───", title).dimmed());
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 5), "hello...");
    }
}
