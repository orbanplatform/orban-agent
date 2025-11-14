# Orban Agent 架構文檔

## 概述

Orban Agent 是一個 GPU 供給端代理程序，允許用戶將閒置的 GPU 算力貢獻給 Orban 平台，並獲得收益。

## 技術棧

### 核心引擎 (agent-core)

- **語言**: Rust
- **異步運行時**: Tokio
- **網路通訊**: WebSocket (tokio-tungstenite)
- **序列化**: JSON + Protocol Buffers
- **加密**: Ed25519 簽名
- **GPU 支援**:
  - NVIDIA: NVML + CUDA
  - AMD: ROCm (規劃中)
  - Apple: Metal (規劃中)

### 用戶介面 (規劃中)

- **框架**: Tauri
- **前端**: React + TypeScript
- **圖表**: Recharts
- **樣式**: TailwindCSS

## 專案結構

```
orban-agent/
├── agent-core/                 # Rust 核心引擎
│   ├── src/
│   │   ├── gpu/               # GPU 偵測與監控
│   │   │   ├── detector.rs    # GPU 偵測器
│   │   │   ├── device.rs      # GPU 設備抽象
│   │   │   ├── nvidia.rs      # NVIDIA 實現
│   │   │   ├── amd.rs         # AMD 實現 (TODO)
│   │   │   └── apple.rs       # Apple 實現 (TODO)
│   │   │
│   │   ├── network/           # 網路通訊
│   │   │   ├── client.rs      # WebSocket 客戶端
│   │   │   ├── orban_protocol.rs  # Orban Protocol 實現
│   │   │   ├── auth.rs        # 認證模組
│   │   │   └── reconnect.rs   # 斷線重連策略
│   │   │
│   │   ├── compute/           # 任務執行
│   │   │   ├── executor.rs    # 任務執行器
│   │   │   └── sandbox.rs     # 安全沙盒
│   │   │
│   │   ├── earnings/          # 收益追蹤
│   │   │   └── tracker.rs     # 收益追蹤器
│   │   │
│   │   ├── types.rs           # 類型定義
│   │   ├── error.rs           # 錯誤處理
│   │   └── lib.rs             # 主入口
│   │
│   ├── proto/
│   │   └── orban.proto        # Protocol Buffers 定義
│   │
│   └── Cargo.toml
│
├── docs/
│   ├── orban-protocol.md      # Orban Protocol 規格
│   └── ARCHITECTURE.md        # 架構文檔
│
└── README.md
```

## 核心模組詳解

### 1. GPU 模組 (`gpu/`)

#### GPUDetector

自動偵測系統中所有可用的 GPU 設備。

```rust
pub struct GPUDetector {
    devices: Vec<GPUDeviceRef>,
}

impl GPUDetector {
    pub fn detect_all() -> Result<Self>
    pub fn get_hardware_info(&self) -> HardwareInfo
    pub fn get_all_status(&self) -> Result<Vec<GPUStatus>>
    pub fn meets_requirements(&self, requirements: &TaskRequirements) -> bool
    pub fn select_best_gpu(&self, requirements: &TaskRequirements) -> Option<&GPUDeviceRef>
}
```

#### GPUDevice Trait

所有 GPU 實現必須實現的抽象接口。

```rust
pub trait GPUDevice: Send + Sync {
    fn index(&self) -> u32;
    fn vendor(&self) -> GPUVendor;
    fn name(&self) -> Result<String>;
    fn memory_info(&self) -> Result<MemoryInfo>;
    fn utilization(&self) -> Result<f32>;
    fn temperature(&self) -> Result<f32>;
    fn power_usage(&self) -> Result<f32>;
    fn fan_speed(&self) -> Result<f32>;
    fn compute_capability(&self) -> Result<String>;
    fn cuda_cores(&self) -> Option<u32>;
    fn pcie_bandwidth(&self) -> Result<u32>;
    fn uuid(&self) -> Result<String>;
    fn compute_pow(&self, challenge: &[u8], difficulty: u32) -> Result<Vec<u8>>;
}
```

#### NVIDIA GPU 實現

使用 NVML (NVIDIA Management Library) 監控 GPU 狀態。

**支援的功能**:
- ✅ GPU 資訊查詢
- ✅ 記憶體使用監控
- ✅ 使用率監控
- ✅ 溫度監控
- ✅ 功耗監控
- ✅ 風扇速度監控
- ✅ 計算能力查詢
- ⏳ CUDA PoW 計算 (規劃中)

### 2. 網路模組 (`network/`)

#### Orban Protocol

定義 Agent 與 Platform 之間的通訊協議。

**訊息類型**:
- 認證: `AuthChallenge`, `AuthResponse`, `AuthSuccess`
- 註冊: `AgentRegister`, `RegisterAck`
- 任務: `TaskAssign`, `TaskAccept`, `TaskReject`, `TaskProgress`, `TaskComplete`, `TaskFailed`
- 監控: `Heartbeat`, `MetricsBatch`
- 收益: `EarningsRecord`, `PayoutNotification`
- 安全: `PowChallenge`, `PowResponse`

**通訊流程**:
```
Agent                Platform
  |                      |
  |---- WS Connect ----->|
  |<-- AuthChallenge ----|
  |-- AuthResponse ----->|
  |<-- AuthSuccess ------|
  |-- AgentRegister ---->|
  |<-- RegisterAck ------|
  |                      |
  |<-- TaskAssign -------|
  |-- TaskAccept ------->|
  |-- TaskProgress ----->| (多次)
  |-- TaskComplete ----->|
  |<-- EarningsRecord ---|
  |                      |
  |-- Heartbeat -------->| (每30秒)
```

#### Authenticator

使用 Ed25519 簽名實現身份驗證。

```rust
pub struct Authenticator {
    keypair: Keypair,
    agent_id: String,
}

impl Authenticator {
    pub fn generate() -> Self
    pub fn from_private_key_file<P: AsRef<Path>>(path: P, agent_id: String) -> Result<Self>
    pub fn sign_challenge(&self, challenge: &[u8]) -> String
    pub fn respond_to_challenge(&self, challenge: &str) -> Result<(String, String)>
}
```

#### OrbanClient

WebSocket 客戶端，處理與平台的通訊。

```rust
pub struct OrbanClient {
    config: Arc<AgentConfig>,
    authenticator: Arc<Authenticator>,
    ws: Arc<Mutex<Option<WebSocketStream<...>>>>,
}

impl OrbanClient {
    pub async fn connect(&self) -> Result<()>
    pub async fn register(...) -> Result<()>
    pub async fn send_message(&self, msg: &Message) -> Result<()>
    pub async fn receive(&self) -> Option<Message>
    pub async fn send_heartbeat(...) -> Result<()>
    pub async fn accept_task(&self, task_id: &str) -> Result<()>
    pub async fn reject_task(&self, task_id: &str, reason: &str) -> Result<()>
}
```

#### ReconnectStrategy

指數退避重連策略。

```rust
pub struct ReconnectStrategy {
    max_retries: u32,
    base_delay_secs: u64,
    max_delay_secs: u64,
}

impl ReconnectStrategy {
    pub fn next_delay(&mut self) -> Option<Duration>
    pub async fn retry<F, Fut, T, E>(&mut self, operation: F) -> Result<T, E>
}
```

重連延遲: 1s, 2s, 4s, 8s, 16s, 32s, 64s, 128s, 256s, 300s (max)

### 3. 計算模組 (`compute/`)

#### TaskExecutor

任務執行器，負責下載模型、執行任務、上傳結果。

```rust
pub struct TaskExecutor {
    gpu_detector: GPUDetector,
    sandbox: Sandbox,
}

impl TaskExecutor {
    pub async fn execute(&self, payload: TaskPayload) -> Result<TaskResult>
}
```

**執行流程**:
1. 下載模型文件 (驗證哈希)
2. 下載輸入數據
3. 在沙盒中執行任務
4. 上傳結果 (計算哈希)
5. 返回執行指標

#### Sandbox

安全沙盒，隔離執行任務。

**隔離策略**:
- **Docker 容器**: `docker run --gpus all --network=none --memory=16g ...`
- **進程隔離**: `setrlimit` 限制 CPU/記憶體/網路

```rust
pub struct Sandbox {
    use_docker: bool,
}

impl Sandbox {
    pub fn run_task(&self, model_path: &str, input_path: &str, config: &serde_json::Value) -> Result<String>
}
```

### 4. 收益模組 (`earnings/`)

#### EarningsTracker

追蹤和持久化收益數據。

```rust
pub struct EarningsTracker {
    data: EarningsData,
    storage_path: PathBuf,
}

impl EarningsTracker {
    pub async fn record_earnings(&mut self, earnings: EarningsDetail) -> Result<()>
    pub fn confirm_earnings(&mut self, task_id: &str) -> Result<()>
    pub fn get_data(&self) -> &EarningsData
    pub fn update_today_earnings(&mut self)
}
```

**數據存儲**:
- 位置: `~/.local/share/orban-agent/earnings.json` (Linux/macOS)
- 格式: JSON
- 包含: 總收益、今日收益、待確認收益、歷史記錄

## 安全機制

### 1. 身份驗證

使用 Ed25519 公鑰加密算法：

1. Agent 生成密鑰對 (私鑰 + 公鑰)
2. Platform 發送隨機挑戰 (challenge)
3. Agent 使用私鑰簽署挑戰
4. Platform 使用公鑰驗證簽名
5. 驗證成功後發放 JWT Token

### 2. 工作證明 (Proof of Work)

防止虛假節點和作弊：

```rust
fn compute_pow(challenge: &[u8], difficulty: u32) -> Vec<u8> {
    // 使用 GPU 並行計算
    // 尋找 hash(challenge || nonce) 前 difficulty 位為 0 的 nonce
}
```

**驗證機制**:
- GPU 特徵簽名 (UUID, CUDA 版本)
- 計算時間驗證
- 結果哈希驗證

### 3. 沙盒隔離

所有任務在隔離環境中執行：

**資源限制**:
- CPU: 根據任務需求限制
- 記憶體: 根據任務需求限制
- GPU: 分配指定 GPU 設備
- 網路: 僅允許訪問指定 URL (模型下載、結果上傳)
- 文件系統: 只讀掛載模型目錄

**Docker 示例**:
```bash
docker run \
  --gpus '"device=0"' \
  --cpus=4 \
  --memory=16g \
  --network=none \
  --read-only \
  --tmpfs /tmp:size=2g \
  -v /models:/models:ro \
  -v /workspace:/workspace \
  orban/runner:latest \
  python inference.py --model=/models/llama-7b --input=/workspace/input.json
```

## 效能優化

### 1. GPU 記憶體管理

```rust
// 選擇可用記憶體最多的 GPU
pub fn select_best_gpu(&self, requirements: &TaskRequirements) -> Option<&GPUDeviceRef> {
    suitable_devices.sort_by(|a, b| {
        let a_mem = a.memory_info().map(|m| m.free).unwrap_or(0);
        let b_mem = b.memory_info().map(|m| m.free).unwrap_or(0);
        b_mem.cmp(&a_mem)
    });
}
```

### 2. 並發任務執行

使用 Tokio 異步運行時，支援多任務並發：

```rust
// 同時執行多個任務 (如果有多個 GPU)
let tasks = FuturesUnordered::new();
for task in task_queue {
    tasks.push(executor.execute(task));
}
```

### 3. 網路優化

- **Protocol Buffers**: 比 JSON 節省 ~60% 頻寬
- **批次上報**: 合併指標減少訊息數量
- **壓縮**: 大型訊息使用 gzip 壓縮

## 錯誤處理

### 錯誤類型

```rust
pub enum Error {
    // GPU 相關
    GPUNotFound,
    InsufficientVRAM { required: u32, available: u32 },
    GPUError(String),

    // 網路相關
    ConnectionFailed(String),
    AuthenticationFailed(String),
    WebSocketError(TungsteniteError),

    // 任務相關
    TaskExecutionFailed(String),
    DownloadFailed(String),
    TaskTimeout,
    OutOfMemory,

    // 其他
    Unknown(String),
}
```

### 錯誤恢復策略

| 錯誤類型 | 可恢復 | 處理方式 |
|---------|--------|----------|
| `ConnectionFailed` | ✅ | 指數退避重連 |
| `DownloadFailed` | ✅ | 重試 3 次 |
| `TaskTimeout` | ✅ | 報告失敗，接受新任務 |
| `GPUNotFound` | ❌ | 停止 Agent |
| `InsufficientVRAM` | ❌ | 拒絕任務 |
| `AuthenticationFailed` | ❌ | 檢查密鑰 |

## 監控與日誌

### 日誌級別

```rust
// 使用 tracing 結構化日誌
use tracing::{info, warn, error, debug, trace};

info!("Task {} completed in {:.2}s", task_id, duration);
warn!("GPU temperature high: {:.1}°C", temp);
error!("Failed to connect: {}", error);
```

### 指標收集

**即時指標** (每 10 秒):
- GPU 使用率
- VRAM 使用量
- 溫度
- 功耗

**聚合指標** (每 5 分鐘):
- 完成任務數
- 失敗任務數
- 總 GPU 時數
- 平均使用率
- 總能耗
- 收益

## 配置

### Agent 配置文件

```toml
# config.toml
[agent]
agent_id = "agent-tw-a1b2c3d4"
platform_url = "wss://platform.orban.ai"
private_key_path = "/etc/orban/agent.key"

[availability]
hours_per_day = 24
reliability_score = 0.98

[resources]
max_concurrent_tasks = 2
reserved_vram_gb = 2  # 預留給系統的 VRAM

[logging]
level = "info"
file = "/var/log/orban-agent.log"
```

## 未來規劃

### 短期 (1-2 個月)

- [ ] 完成 Docker 沙盒實現
- [ ] 實現文件下載/上傳 (S3)
- [ ] CUDA PoW 實現
- [ ] Tauri UI 開發
- [ ] 安裝程式打包

### 中期 (3-6 個月)

- [ ] AMD ROCm 支援
- [ ] Apple Metal 支援
- [ ] Protocol Buffers 完整實現
- [ ] 本地模型緩存
- [ ] P2P 模型分發

### 長期 (6-12 個月)

- [ ] 去中心化驗證節點
- [ ] 鏈上結算 (區塊鏈整合)
- [ ] 聯邦學習支援
- [ ] 自動調優與負載均衡

## 參考資料

- [Orban Protocol 規格](./orban-protocol.md)
- [NVML 文檔](https://docs.nvidia.com/deploy/nvml-api/)
- [Tokio 文檔](https://tokio.rs/)
- [Tauri 文檔](https://tauri.app/)
