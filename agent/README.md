# Orban Agent - GPU 供給端節點

> 從第一性原理設計的分散式 GPU 算力貢獻系統

## 核心設計原則

### 1. 第一性原理思考

**問題本質**：閒置 GPU + 計算需求 = 資源錯配

**解決方案**：
- 發現：偵測本地 GPU 能力
- 連接：安全連接到 Orban 調度平台
- 執行：在隔離環境中運行任務
- 證明：提供可驗證的計算證明
- 獲益：透明的收益追蹤

### 2. 技術棧選擇依據

```
需求               → 技術選擇           → 理由
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
GPU 直接操作       → Rust              → 零成本抽象 + FFI
記憶體安全         → Rust              → 所有權系統
跨平台編譯         → Rust + Cargo      → 一次編譯，處處運行
桌面 UI            → Tauri + React     → 輕量 (5MB vs 150MB)
GPU 驅動           → CUDA/ROCm/Metal   → 硬體原生支援
容器隔離           → Docker/Podman     → 任務沙盒
通訊協議           → WebSocket/gRPC    → 即時雙向通訊
```

## 架構設計

### 分層架構

```
┌─────────────────────────────────────────┐
│  Layer 4: Application Layer             │
│  ├─ Tauri Desktop UI                    │
│  ├─ System Tray                         │
│  └─ CLI Interface                       │
├─────────────────────────────────────────┤
│  Layer 3: Business Logic                │
│  ├─ Earnings Tracker                    │
│  ├─ Task Scheduler                      │
│  └─ Config Manager                      │
├─────────────────────────────────────────┤
│  Layer 2: Core Services                 │
│  ├─ GPU Monitor                         │
│  ├─ Task Executor                       │
│  ├─ Network Client                      │
│  └─ Security Sandbox                    │
├─────────────────────────────────────────┤
│  Layer 1: Hardware Abstraction          │
│  ├─ NVIDIA CUDA                         │
│  ├─ AMD ROCm                            │
│  ├─ Intel oneAPI                        │
│  └─ Apple Metal                         │
├─────────────────────────────────────────┤
│  Layer 0: Operating System              │
│  └─ Linux / Windows / macOS             │
└─────────────────────────────────────────┘
```

### 資料流

```
用戶啟動 Agent
    ↓
GPU 偵測與註冊
    ↓
連接到 Orban Platform ←─────┐
    ↓                        │
領取任務                     │
    ↓                        │
下載模型/資料                │
    ↓                        │
執行計算 (GPU)               │
    ↓                        │
生成工作證明                 │
    ↓                        │
上傳結果與證明               │
    ↓                        │
記錄收益 ────────────────────┘
    ↓
更新 UI 顯示
```

## 專案結構

```
agent/
├── core/                      # Rust 核心庫
│   ├── src/
│   │   ├── gpu/              # GPU 抽象層
│   │   │   ├── mod.rs
│   │   │   ├── detector.rs   # GPU 偵測
│   │   │   ├── monitor.rs    # 即時監控
│   │   │   ├── nvidia.rs     # NVIDIA 支援
│   │   │   ├── amd.rs        # AMD 支援
│   │   │   └── apple.rs      # Apple Metal
│   │   ├── compute/          # 計算引擎
│   │   │   ├── mod.rs
│   │   │   ├── executor.rs   # 任務執行器
│   │   │   ├── sandbox.rs    # 安全沙盒
│   │   │   └── verifier.rs   # 工作證明
│   │   ├── network/          # 網路層
│   │   │   ├── mod.rs
│   │   │   ├── client.rs     # Platform 客戶端
│   │   │   ├── protocol.rs   # 通訊協議
│   │   │   └── p2p.rs        # P2P 傳輸
│   │   ├── earnings/         # 收益系統
│   │   │   ├── mod.rs
│   │   │   ├── tracker.rs    # 收益追蹤
│   │   │   └── calculator.rs # 費率計算
│   │   ├── config/           # 配置管理
│   │   │   └── mod.rs
│   │   └── lib.rs
│   ├── Cargo.toml
│   └── build.rs
│
├── desktop/                   # Tauri 桌面應用
│   ├── src-tauri/
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── commands.rs   # Tauri 命令
│   │   │   └── tray.rs       # 系統列
│   │   ├── Cargo.toml
│   │   ├── tauri.conf.json
│   │   └── icons/
│   ├── src/
│   │   ├── components/
│   │   │   ├── Dashboard.tsx
│   │   │   ├── GPUMonitor.tsx
│   │   │   ├── EarningsChart.tsx
│   │   │   ├── TaskHistory.tsx
│   │   │   └── Settings.tsx
│   │   ├── hooks/
│   │   │   └── useAgentData.ts
│   │   ├── App.tsx
│   │   └── main.tsx
│   ├── package.json
│   └── vite.config.ts
│
├── installer/                 # 安裝程式
│   ├── linux/
│   │   ├── install.sh
│   │   └── orban-agent.service
│   ├── windows/
│   │   └── installer.nsi
│   └── macos/
│       └── create-pkg.sh
│
├── proto/                     # Protocol Buffers
│   ├── agent.proto
│   └── task.proto
│
└── docs/
    ├── ARCHITECTURE.md
    ├── GPU_SUPPORT.md
    └── SECURITY.md
```

## 核心功能

### 1. GPU 偵測與監控

```rust
// 自動偵測所有支援的 GPU
let detector = GPUDetector::new();
let devices = detector.detect_all()?;

for device in devices {
    println!("Found: {} ({}GB VRAM)",
        device.name(),
        device.memory_total_gb()
    );
}
```

### 2. 任務執行

```rust
let executor = TaskExecutor::new(gpu_device);
let result = executor.execute(task).await?;
let proof = executor.generate_proof(&result)?;
```

### 3. 收益追蹤

```rust
let tracker = EarningsTracker::load()?;
tracker.record_completed_task(task, earnings);
println!("Total earned: ${}", tracker.total());
```

## 安全性設計

### 多層防護

1. **網路層**：TLS 1.3 加密，mTLS 雙向認證
2. **執行層**：容器沙盒，資源限制（cgroups）
3. **驗證層**：Proof of GPU Work，防止偽造
4. **資料層**：模型加密傳輸，本地加密儲存

### 工作證明機制

```rust
// GPU 特有的計算密集型驗證
fn generate_proof_of_work(result: &TaskResult) -> Proof {
    // 只能用 GPU 高效計算的哈希
    let challenge = platform.get_challenge();
    let proof = gpu_intensive_hash(challenge, result);

    Proof {
        challenge,
        response: proof,
        gpu_signature: hardware_id(),
        timestamp: now(),
    }
}
```

## 快速開始

### 安裝

```bash
# Linux / macOS
curl -fsSL https://get.orban.ai/agent | sh

# Windows (PowerShell)
irm https://get.orban.ai/agent.ps1 | iex
```

### 啟動

```bash
# 啟動 Agent
orban-agent start

# 開啟控制台
orban-agent ui

# 查看狀態
orban-agent status
```

## 開發

### 建置核心

```bash
cd core
cargo build --release
```

### 建置桌面應用

```bash
cd desktop
npm install
npm run tauri build
```

### 測試

```bash
# 單元測試
cargo test

# 整合測試
cargo test --test integration

# GPU 測試（需要實體 GPU）
cargo test --features gpu-tests
```

## License

MIT License (Agent Core - Open Source)
Proprietary (Advanced Scheduling Algorithms)
