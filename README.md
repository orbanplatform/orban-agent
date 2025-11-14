# 🚀 Orban Agent - GPU Supply-Side Agent

> Monetize your idle GPU compute power and contribute to AI workloads

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

[中文文檔](./README-zh.md)

## 📖 Overview

Orban Agent is a high-performance, secure GPU supply-side agent that allows users to contribute their idle GPU compute power to the Orban platform, execute AI inference and training tasks, and earn rewards.

### Core Features

- ✅ **Multi-GPU Support**: NVIDIA (CUDA), AMD (ROCm), Apple (Metal)
- ✅ **Secure Isolation**: Docker/process sandbox for task execution
- ✅ **Automation**: Auto-start, reconnection, fault recovery
- ✅ **Earnings Tracking**: Real-time earnings monitoring, history, auto settlement
- ✅ **Low Latency**: WebSocket persistent connection, efficient binary protocol
- ✅ **Privacy Protection**: End-to-end encryption, local data stays local

## 🏗️ Architecture

```
┌─────────────────────────────────────┐
│      Tauri Desktop UI (React)      │  ← Earnings Dashboard
└──────────────┬──────────────────────┘
               │ IPC
┌──────────────▼──────────────────────┐
│         Orban Agent Core            │
│           (Rust Engine)             │
├─────────────────────────────────────┤
│  GPU Monitor  │  Task Executor      │
│  Network      │  Earnings Tracker   │
└────┬──────────┴──────────────┬──────┘
     │                         │
┌────▼────┐             ┌──────▼──────┐
│  NVIDIA │             │   Orban     │
│   GPU   │             │  Platform   │
└─────────┘             └─────────────┘
```

## 🚀 Quick Start

### System Requirements

- **OS**: Linux (Ubuntu 20.04+) / macOS 12+ / Windows 10+
- **GPU**: NVIDIA (CUDA 11.0+) / AMD (ROCm 5.0+) / Apple Silicon
- **Memory**: 8GB+ RAM
- **Storage**: 10GB+ available space

### One-Click Installation

**Linux / macOS**
```bash
curl -fsSL https://get.orban.ai/agent | sh
```

**Windows (PowerShell)**
```powershell
irm https://get.orban.ai/agent.ps1 | iex
```

### Alternative Installation (If DNS Resolution Fails)

If you encounter DNS resolution errors like `Could not resolve host: get.orban.ai`, use these alternative methods:

**Method 1: Direct GitHub Download (Linux)**
```bash
curl -fsSL https://raw.githubusercontent.com/orbanplatform/orban-agent/main/agent/installer/linux/install.sh | sh
```

**Method 2: Manual Script Download (Linux)**
```bash
# Download the installation script
wget https://raw.githubusercontent.com/orbanplatform/orban-agent/main/agent/installer/linux/install.sh

# Make it executable
chmod +x install.sh

# Run the installer
./install.sh
```

**Method 3: Direct Binary Download (Linux x86_64)**
```bash
# Download the latest release
wget https://github.com/orbanplatform/orban-agent/releases/latest/download/orban-agent-linux-x86_64 -O /tmp/orban-agent

# Make it executable
chmod +x /tmp/orban-agent

# Move to system path
sudo mv /tmp/orban-agent /usr/local/bin/orban-agent

# Verify installation
orban-agent --version
```

**Method 4: Build from Source (All Platforms)**

See the [Manual Installation](#manual-installation) section below for detailed build instructions.

### Manual Installation

#### 1. Install Dependencies

**Ubuntu/Debian**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install NVIDIA drivers (if you have NVIDIA GPU)
sudo apt install nvidia-driver-535 nvidia-cuda-toolkit

# Install Docker (optional, for sandbox isolation)
curl -fsSL https://get.docker.com | sh
```

#### 2. Build Agent

```bash
git clone https://github.com/orban-ai/orban-agent.git
cd orban-agent/agent-core
cargo build --release
```

#### 3. Generate Keys

```bash
./target/release/orban-agent keygen --output ~/.orban/agent.key
```

#### 4. Configure

Create config file `~/.orban/config.toml`:

```toml
[agent]
platform_url = "wss://platform.orban.ai"
private_key_path = "/home/user/.orban/agent.key"

[availability]
hours_per_day = 24
reliability_score = 0.98

[resources]
max_concurrent_tasks = 1
reserved_vram_gb = 2
```

#### 5. Start

```bash
./target/release/orban-agent start
```

## 📊 Earnings

### Pricing Model

Base Rate × GPU Multiplier × Runtime = Earnings

**Base Rate**: $0.01 USD / GPU Hour

**GPU Multipliers**:

| GPU Model | Multiplier | Hourly Rate |
|-----------|------------|-------------|
| RTX 4090  | 2.5×       | $0.025     |
| RTX 3090  | 1.8×       | $0.018     |
| A100      | 5.0×       | $0.050     |
| RTX 3080  | 1.5×       | $0.015     |
| V100      | 3.5×       | $0.035     |

### Earnings Example

**RTX 4090** running 24/7:
- Daily: $0.025 × 24 = **$0.60**
- Monthly: $0.60 × 30 = **$18.00**
- Yearly: $18.00 × 12 = **$216.00**

## 📝 Development

### Project Structure

```
orban-agent/
├── agent-core/          # Rust core engine
│   ├── src/
│   │   ├── gpu/        # GPU detection & monitoring
│   │   ├── network/    # Orban Protocol communication
│   │   ├── compute/    # Task execution engine
│   │   └── earnings/   # Earnings tracking
│   └── Cargo.toml
│
├── docs/
│   ├── orban-protocol.md     # Protocol specification
│   └── ARCHITECTURE.md       # Architecture documentation
│
└── README.md
```

### Local Development

```bash
# Clone repository
git clone https://github.com/orban-ai/orban-agent.git
cd orban-agent

# Build core
cd agent-core
cargo build

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check code
cargo clippy
cargo fmt
```

## 📚 Documentation

- [Orban Protocol Specification](./docs/orban-protocol.md)
- [Architecture Documentation](./docs/ARCHITECTURE.md)
- [中文文檔](./README-zh.md)

## 🔒 Security

### Authentication

Uses Ed25519 public-key cryptography:
- Agent generates unique keypair
- Challenge-response authentication
- JWT token session management

### Sandbox Isolation

All tasks execute in isolated environment:
- Docker container isolation
- No network access (except specified URLs)
- Resource limits (CPU/Memory/GPU)
- Read-only filesystem

### Proof of Work

Prevents fake nodes:
- GPU signature verification
- Real-time PoW challenges
- Result cross-validation

## 🤝 Community & Support

- 💬 Discord: https://discord.gg/orban
- 📧 Email: support@orban.ai
- 🐛 Bug Reports: https://github.com/orban-ai/orban-agent/issues
- 📖 Docs: https://docs.orban.ai

## 🔧 Troubleshooting

### Installation Issues

**Problem: `Could not resolve host: get.orban.ai`**

This DNS resolution error occurs when the domain cannot be reached. Solutions:

1. **Use Direct GitHub Installation:**
   ```bash
   curl -fsSL https://raw.githubusercontent.com/orbanplatform/orban-agent/main/agent/installer/linux/install.sh | sh
   ```

2. **Check DNS Settings:**
   ```bash
   # Test DNS resolution
   nslookup get.orban.ai

   # Try alternative DNS servers (Google DNS)
   echo "nameserver 8.8.8.8" | sudo tee /etc/resolv.conf
   ```

3. **Use Alternative Installation Methods:**
   - See the [Alternative Installation](#alternative-installation-if-dns-resolution-fails) section above

**Problem: Download Fails from GitHub**

If GitHub is blocked or slow:

1. **Check GitHub Accessibility:**
   ```bash
   curl -I https://github.com
   ```

2. **Use Mirror or VPN:**
   - Use a VPN if GitHub is blocked in your region
   - Contact support@orban.ai for alternative download mirrors

3. **Build from Source:**
   - Clone the repository and build manually (see Manual Installation)

**Problem: Permission Denied**

If you see permission errors during installation:

```bash
# Give installer sudo access when prompted
# Or manually move binary with sudo:
sudo mv /tmp/orban-agent /usr/local/bin/orban-agent
```

### Runtime Issues

**Problem: No GPU Detected**

```bash
# For NVIDIA GPUs, verify drivers:
nvidia-smi

# For AMD GPUs:
rocm-smi

# Install drivers if missing:
sudo apt install nvidia-driver-535  # NVIDIA
# or
sudo apt install rocm  # AMD
```

**Problem: Agent Won't Start**

```bash
# Check agent status
orban-agent status

# View logs
journalctl --user -u orban-agent -f

# Restart agent
systemctl --user restart orban-agent
```

**Problem: Connection to Platform Failed**

```bash
# Test platform connectivity
curl -I https://platform.orban.ai

# Check firewall settings (allow WebSocket connections)
sudo ufw allow out 443/tcp
```

### Getting Help

If issues persist:
1. Check logs: `journalctl --user -u orban-agent -n 50`
2. Report issue: https://github.com/orban-ai/orban-agent/issues
3. Include: OS version, GPU model, error messages, and logs

## 📜 License

MIT License - See [LICENSE](./LICENSE) file

## 🙏 Acknowledgements

Special thanks to:

- [Tokio](https://tokio.rs/) - Async runtime
- [NVML](https://developer.nvidia.com/nvml) - NVIDIA management library
- [Tauri](https://tauri.app/) - Desktop app framework
- [Rust](https://www.rust-lang.org/) - Systems programming language

---

**Disclaimer**: Before using Orban Agent, ensure compliance with local laws and GPU vendor terms of use. Extended high-load operation may affect hardware lifespan.
