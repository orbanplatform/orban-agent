// 網路通訊模組

mod client;
mod simple_client;
mod orban_protocol;
mod auth;
mod reconnect;

pub use client::OrbanClient;
pub use simple_client::{Client, RegistrationRequest, GpuInfo, GpuType, Task, TaskResult};
pub use orban_protocol::{
    Message, MessageType, MessagePayload,
    TaskAssignPayload, EarningsRecordPayload, EarningsDetail,
    PowChallengePayload, AgentStatus
};
pub use auth::Authenticator;

use crate::error::Result;
