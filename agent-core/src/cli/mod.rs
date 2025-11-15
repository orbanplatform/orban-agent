//! CLI 命令模塊

pub mod start;
pub mod stop;
pub mod status;
pub mod earnings;
pub mod logs;

use crate::Result;

/// CLI 命令執行結果
pub type CommandResult = Result<()>;
