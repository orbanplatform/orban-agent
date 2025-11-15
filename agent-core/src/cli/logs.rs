//! Logs 命令實現

use crate::{Result, Error, config::Config};
use colored::Colorize;
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;

/// 執行 logs 命令
pub async fn execute(follow: bool, lines: usize) -> Result<()> {
    let config = Config::load()?;
    let log_file = config.log_dir().join("agent.log");

    if !log_file.exists() {
        println!("{} Log file not found", "ℹ".blue());
        println!("  {}", "Start the agent to generate logs".dimmed());
        println!("  Location: {}", log_file.display().to_string().dimmed());
        return Ok(());
    }

    println!("{} Orban Agent Logs", "📋".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();

    if follow {
        follow_logs(&log_file).await
    } else {
        show_last_lines(&log_file, lines)
    }
}

/// 顯示最後 N 行日誌
fn show_last_lines(log_file: &PathBuf, n: usize) -> Result<()> {
    let file = File::open(log_file)?;
    let reader = BufReader::new(file);

    let lines: Vec<String> = reader
        .lines()
        .filter_map(|line| line.ok())
        .collect();

    let start = if lines.len() > n {
        lines.len() - n
    } else {
        0
    };

    for line in &lines[start..] {
        print_log_line(line);
    }

    println!();
    println!("{}", format!("Showing last {} lines", lines.len() - start).dimmed());
    println!("Use {} to follow logs in real-time",
        "orban-agent logs --follow".cyan()
    );

    Ok(())
}

/// 追蹤日誌（類似 tail -f）
async fn follow_logs(log_file: &PathBuf) -> Result<()> {
    println!("{} Following logs (Ctrl+C to stop)...", "ℹ".blue());
    println!();

    let mut file = File::open(log_file)?;

    // 跳到文件末尾
    file.seek(SeekFrom::End(0))?;

    let mut reader = BufReader::new(file);
    let mut line = String::new();

    loop {
        match reader.read_line(&mut line) {
            Ok(0) => {
                // 沒有新內容，等待
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                // 檢查文件是否被輪替
                if !log_file.exists() {
                    println!("{} Log file rotated, reopening...", "⚠".yellow());
                    file = File::open(log_file)?;
                    reader = BufReader::new(file);
                }
            }
            Ok(_) => {
                print_log_line(&line);
                line.clear();
            }
            Err(e) => {
                return Err(Error::Unknown(format!("Failed to read log: {}", e)));
            }
        }
    }
}

/// 打印並格式化日誌行
fn print_log_line(line: &str) {
    let line = line.trim();

    if line.is_empty() {
        return;
    }

    // 嘗試解析日誌級別並著色
    if line.contains("ERROR") || line.contains("error") {
        println!("{}", line.red());
    } else if line.contains("WARN") || line.contains("warn") {
        println!("{}", line.yellow());
    } else if line.contains("INFO") || line.contains("info") {
        println!("{}", line);
    } else if line.contains("DEBUG") || line.contains("debug") {
        println!("{}", line.dimmed());
    } else if line.contains("TRACE") || line.contains("trace") {
        println!("{}", line.bright_black());
    } else {
        // 默認樣式
        println!("{}", line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_log_line() {
        // 這個測試只是確保函數不會崩潰
        print_log_line("INFO: Test message");
        print_log_line("ERROR: Test error");
        print_log_line("");
    }
}
