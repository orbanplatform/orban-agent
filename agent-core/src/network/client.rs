// Orban WebSocket 客戶端

use super::auth::Authenticator;
use super::orban_protocol::{Message, MessageType, MessagePayload, AgentStatus};
use super::reconnect::ReconnectStrategy;
use crate::types::*;
use crate::error::{Error, Result};
use crate::AgentConfig;

use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use futures::{StreamExt, SinkExt};
use tracing::{info, warn, error};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Orban 客戶端
#[derive(Clone)]
pub struct OrbanClient {
    config: Arc<AgentConfig>,
    authenticator: Arc<Authenticator>,
    ws: Arc<Mutex<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    reconnect_strategy: Arc<Mutex<ReconnectStrategy>>,
    jwt_token: Arc<Mutex<Option<String>>>,
}

impl OrbanClient {
    /// 創建新的客戶端
    pub async fn new(config: &AgentConfig) -> Result<Self> {
        let authenticator = Authenticator::from_private_key_file(
            &config.private_key_path,
            config.agent_id.clone(),
        )?;

        Ok(Self {
            config: Arc::new(config.clone()),
            authenticator: Arc::new(authenticator),
            ws: Arc::new(Mutex::new(None)),
            reconnect_strategy: Arc::new(Mutex::new(ReconnectStrategy::new())),
            jwt_token: Arc::new(Mutex::new(None)),
        })
    }

    /// 連接到平台
    pub async fn connect(&self) -> Result<()> {
        info!("Connecting to Orban Platform at {}", self.config.platform_url);

        let url = format!("{}/agent/v1/connect", self.config.platform_url);

        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| Error::ConnectionFailed(e.to_string()))?;

        info!("WebSocket connection established");

        *self.ws.lock().await = Some(ws_stream);

        // 執行認證
        self.authenticate().await?;

        // 重置重連策略
        self.reconnect_strategy.lock().await.reset();

        Ok(())
    }

    /// 認證
    async fn authenticate(&self) -> Result<()> {
        info!("Authenticating with platform...");

        // 接收認證挑戰
        let challenge_msg = self.receive_message().await?;

        if let MessagePayload::AuthChallenge(challenge) = challenge_msg.payload {
            // 響應挑戰
            let (signature, public_key) = self
                .authenticator
                .respond_to_challenge(&challenge.challenge)?;

            let response = super::orban_protocol::create_auth_response(
                self.authenticator.agent_id().to_string(),
                signature,
                public_key,
            );

            self.send_message(&response).await?;

            // 接收認證成功
            let auth_success = self.receive_message().await?;

            if let MessagePayload::AuthSuccess(success) = auth_success.payload {
                *self.jwt_token.lock().await = Some(success.jwt_token);
                info!("Authentication successful");
                Ok(())
            } else {
                Err(Error::AuthenticationFailed("Invalid response".to_string()))
            }
        } else {
            Err(Error::AuthenticationFailed(
                "Expected auth challenge".to_string(),
            ))
        }
    }

    /// 註冊 Agent
    pub async fn register(
        &self,
        hardware: HardwareInfo,
        capabilities: Capabilities,
        location: Location,
        availability: Availability,
    ) -> Result<()> {
        info!("Registering agent...");

        let msg = super::orban_protocol::create_agent_register(
            self.authenticator.agent_id().to_string(),
            hardware,
            capabilities,
            location,
            availability,
        );

        self.send_message(&msg).await?;

        // 接收註冊確認
        let ack = self.receive_message().await?;

        if matches!(ack.message_type, MessageType::RegisterAck) {
            info!("Agent registered successfully");
            Ok(())
        } else {
            Err(Error::Unknown("Registration failed".to_string()))
        }
    }

    /// 發送訊息
    pub async fn send_message(&self, msg: &Message) -> Result<()> {
        let json = msg.to_json()?;

        if let Some(ws) = self.ws.lock().await.as_mut() {
            ws.send(WsMessage::Text(json))
                .await
                .map_err(|e| Error::WebSocketError(e))?;
            Ok(())
        } else {
            Err(Error::ConnectionFailed("Not connected".to_string()))
        }
    }

    /// 接收訊息
    pub async fn receive(&self) -> Option<Message> {
        self.receive_message().await.ok()
    }

    /// 接收訊息 (內部)
    async fn receive_message(&self) -> Result<Message> {
        if let Some(ws) = self.ws.lock().await.as_mut() {
            while let Some(msg) = ws.next().await {
                match msg {
                    Ok(WsMessage::Text(text)) => {
                        return Message::from_json(&text);
                    }
                    Ok(WsMessage::Binary(data)) => {
                        // TODO: 支援 protobuf
                        warn!("Received binary message (protobuf not yet supported)");
                    }
                    Ok(WsMessage::Close(_)) => {
                        warn!("WebSocket closed by server");
                        return Err(Error::ConnectionFailed("Connection closed".to_string()));
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        return Err(Error::WebSocketError(e));
                    }
                    _ => {}
                }
            }
        }

        Err(Error::ConnectionFailed("Not connected".to_string()))
    }

    /// 發送心跳
    pub async fn send_heartbeat(
        &self,
        status: AgentStatus,
        current_task_id: Option<String>,
        gpu_status: Vec<GPUStatus>,
        uptime_sec: u64,
    ) -> Result<()> {
        let msg = super::orban_protocol::create_heartbeat(
            self.authenticator.agent_id().to_string(),
            status,
            current_task_id,
            gpu_status,
            uptime_sec,
        );

        self.send_message(&msg).await
    }

    /// 接受任務
    pub async fn accept_task(&self, task_id: &str) -> Result<()> {
        let msg = super::orban_protocol::create_task_accept(
            task_id.to_string(),
            self.authenticator.agent_id().to_string(),
            0,
            0,
        );

        self.send_message(&msg).await
    }

    /// 拒絕任務
    pub async fn reject_task(&self, task_id: &str, reason: &str) -> Result<()> {
        let msg = super::orban_protocol::create_task_reject(
            task_id.to_string(),
            reason.to_string(),
            String::new(),
        );

        self.send_message(&msg).await
    }

    /// 完成任務
    pub async fn complete_task(&self, task_id: &str, result: TaskResult) -> Result<()> {
        // TODO: 實現完整的任務完成訊息
        info!("Task {} completed", task_id);
        Ok(())
    }

    /// 發送 PoW 響應
    pub async fn send_pow_response(&self, response: crate::gpu::PowResponse) -> Result<()> {
        use super::orban_protocol::{Message, MessageType, MessagePayload, PowResponsePayload, GpuSignature};

        info!("Sending PoW response for challenge: {}", response.challenge_id);

        let payload = PowResponsePayload {
            challenge_id: response.challenge_id,
            response: hex::encode(response.response),  // Convert Vec<u8> to hex string
            computation_time_ms: response.computation_time_ms as u32,
            gpu_signature: GpuSignature {
                device_uuid: response.gpu_signature.device_uuid,
                cuda_version: response.gpu_signature.cuda_version,
            },
        };

        let msg = Message {
            message_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            message_type: MessageType::PowResponse,
            payload: MessagePayload::PowResponse(payload),
        };

        self.send_message(&msg).await
    }

    /// 斷線
    pub async fn disconnect(&self) -> Result<()> {
        if let Some(mut ws) = self.ws.lock().await.take() {
            ws.close(None).await.ok();
            info!("Disconnected from platform");
        }
        Ok(())
    }
}
