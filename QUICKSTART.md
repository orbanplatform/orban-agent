# 🚀 Orban Agent 快速啟動指南

> 5 分鐘內開始賺取 GPU 收益

## 一鍵安裝

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/orbanplatform/orban-agent/main/install.sh | bash
```

### Windows (PowerShell)

```powershell
iwr -useb https://raw.githubusercontent.com/orbanplatform/orban-agent/main/install.ps1 | iex
```

## 立即開始

安裝完成後，只需 3 個命令：

```bash
# 1. 啟動 agent
orban-agent start

# 2. 檢查狀態
orban-agent status

# 3. 查看收益
orban-agent earnings
```

就這麼簡單！🎉

## 詳細使用

### 查看所有命令

```bash
orban-agent --help
```

### 常用命令

| 命令 | 說明 |
|------|------|
| `orban-agent start` | 啟動 agent（後台運行） |
| `orban-agent start -f` | 前台運行（調試用） |
| `orban-agent stop` | 停止 agent |
| `orban-agent status` | 查看運行狀態 |
| `orban-agent status -v` | 查看詳細狀態（包含 GPU 信息） |
| `orban-agent earnings` | 查看收益摘要 |
| `orban-agent earnings -h` | 查看收益歷史記錄 |
| `orban-agent logs` | 查看日誌（最後 50 行） |
| `orban-agent logs -f` | 實時追蹤日誌 |
| `orban-agent version` | 查看版本信息 |

## 系統要求

### 最低要求
- **作業系統**: Linux (Ubuntu 20.04+) / macOS 12+ / Windows 10+
- **記憶體**: 4GB+ RAM
- **儲存空間**: 10GB+ 可用空間

### GPU 支援
- **NVIDIA**: CUDA 11.0+ (推薦 RTX 系列)
- **AMD**: ROCm 5.0+
- **Apple**: M1/M2/M3 (Metal)

> 💡 **沒有 GPU？** Agent 仍可安裝和運行，只是無法執行任務。

## 配置

預設配置文件位於：`~/.orban-agent/config.toml`

```toml
[agent]
platform_url = "https://platform.orban.ai"

[gpu]
max_concurrent_tasks = 1
reserved_vram_gb = 2.0

[network]
heartbeat_interval_secs = 30
connection_timeout_secs = 10
```

## 故障排除

### Agent 無法啟動

```bash
# 檢查日誌
orban-agent logs

# 檢查 GPU
nvidia-smi  # NVIDIA
rocm-smi    # AMD
```

### GPU 未檢測到

1. **NVIDIA**: 安裝最新驅動
   ```bash
   # Ubuntu/Debian
   sudo apt install nvidia-driver-535
   ```

2. **AMD**: 安裝 ROCm
   ```bash
   # Ubuntu
   sudo apt install rocm
   ```

3. **Apple**: M 系列芯片自帶 Metal，無需額外安裝

### 無法連接平台

```bash
# 檢查網絡連接
ping platform.orban.ai

# 檢查防火牆
sudo ufw allow out 443/tcp
```

## 收益計算

```
收益 = 基礎費率 × GPU 倍數 × 運行時間
```

**基礎費率**: $0.01 USD / GPU 小時

**GPU 倍數**:
- RTX 4090: 2.5×  → $0.025/小時
- RTX 3090: 1.8×  → $0.018/小時
- A100: 5.0×      → $0.050/小時

**例如**: RTX 4090 運行 24 小時 = $0.025 × 24 = **$0.60/天**

## 進階功能

### 自動啟動（Linux/macOS）

```bash
# 創建 systemd 服務
sudo tee /etc/systemd/system/orban-agent.service > /dev/null <<EOF
[Unit]
Description=Orban GPU Agent
After=network.target

[Service]
Type=simple
User=$USER
ExecStart=$(which orban-agent) start --foreground
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF

# 啟用自動啟動
sudo systemctl enable orban-agent
sudo systemctl start orban-agent
```

### Docker 運行（隔離）

```bash
docker run -d \
  --name orban-agent \
  --gpus all \
  -v ~/.orban-agent:/root/.orban-agent \
  orban/agent:latest
```

## 獲取幫助

- 📖 文檔: https://docs.orban.ai
- 💬 Discord: https://discord.gg/orban
- 📧 Email: support@orban.ai
- 🐛 問題回報: https://github.com/orbanplatform/orban-agent/issues

## 更新 Agent

```bash
# 重新運行安裝腳本即可
curl -fsSL https://raw.githubusercontent.com/orbanplatform/orban-agent/main/install.sh | bash
```

## 卸載

```bash
# 停止 agent
orban-agent stop

# 刪除二進制文件
sudo rm /usr/local/bin/orban-agent

# 刪除配置（可選）
rm -rf ~/.orban-agent
```

---

**開始賺錢吧！** 💰

有問題？加入我們的 [Discord 社群](https://discord.gg/orban) 獲取幫助。
