#!/bin/bash
#
# Orban Agent 安裝腳本 (Linux)
#
# 使用方式：
#   方式 1 (推薦): curl -fsSL https://get.orban.ai/agent | sh
#   方式 2 (備用): curl -fsSL https://raw.githubusercontent.com/orbanplatform/orban-agent/main/agent/installer/linux/install.sh | sh
#   方式 3 (離線): wget https://raw.githubusercontent.com/orbanplatform/orban-agent/main/agent/installer/linux/install.sh && bash install.sh
#

set -e

# 設置重試次數
MAX_RETRIES=3
RETRY_DELAY=2

# 顏色輸出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# 橫幅
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🚀 Orban Agent Installer v1.0.0"
echo "  Contribute your GPU, earn rewards"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 檢查作業系統
info "Detecting operating system..."
OS="$(uname -s)"
ARCH="$(uname -m)"

if [ "$OS" != "Linux" ]; then
    error "This installer is for Linux only"
    error "For macOS, use: curl -fsSL https://get.orban.ai/agent-macos | sh"
    error "For Windows, use: irm https://get.orban.ai/agent.ps1 | iex"
    exit 1
fi

success "Operating System: Linux ($ARCH)"

# 檢查必要工具
info "Checking required tools..."

if ! command -v curl &> /dev/null; then
    error "curl is required but not installed"
    exit 1
fi

success "Required tools available"

# 偵測 GPU
info "Detecting GPU..."

GPU_FOUND=false
GPU_TYPE=""

# 檢查 NVIDIA GPU
if command -v nvidia-smi &> /dev/null; then
    GPU_INFO=$(nvidia-smi --query-gpu=name,memory.total --format=csv,noheader 2>/dev/null || true)
    if [ -n "$GPU_INFO" ]; then
        GPU_FOUND=true
        GPU_TYPE="NVIDIA"
        success "Found NVIDIA GPU: $GPU_INFO"
    fi
fi

# 檢查 AMD GPU
if [ "$GPU_FOUND" = false ] && command -v rocm-smi &> /dev/null; then
    if rocm-smi &> /dev/null; then
        GPU_FOUND=true
        GPU_TYPE="AMD"
        success "Found AMD GPU (ROCm detected)"
    fi
fi

if [ "$GPU_FOUND" = false ]; then
    warn "No compatible GPU detected"
    warn "Agent will still install but won't be able to execute tasks"
    echo ""
    read -p "Continue installation? (y/n) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# 檢查 Docker（可選）
info "Checking for Docker..."

if command -v docker &> /dev/null; then
    if docker ps &> /dev/null; then
        success "Docker is available"
        DOCKER_AVAILABLE=true
    else
        warn "Docker is installed but not accessible (may need sudo)"
        DOCKER_AVAILABLE=false
    fi
else
    warn "Docker not found (tasks will run with reduced isolation)"
    DOCKER_AVAILABLE=false
fi

# 下載 Agent 二進制文件
info "Downloading Orban Agent..."

INSTALL_DIR="/usr/local/bin"
TEMP_FILE="/tmp/orban-agent"

# 根據架構選擇正確的二進制文件
if [ "$ARCH" = "x86_64" ]; then
    DOWNLOAD_URL="https://github.com/orbanplatform/orban-agent/releases/latest/download/orban-agent-linux-x86_64"
elif [ "$ARCH" = "aarch64" ]; then
    DOWNLOAD_URL="https://github.com/orbanplatform/orban-agent/releases/latest/download/orban-agent-linux-aarch64"
else
    error "Unsupported architecture: $ARCH"
    exit 1
fi

# 帶重試的下載函數
download_with_retry() {
    local url=$1
    local output=$2
    local attempt=1

    while [ $attempt -le $MAX_RETRIES ]; do
        info "Download attempt $attempt/$MAX_RETRIES..."

        if curl -fsSL "$url" -o "$output" 2>/dev/null; then
            return 0
        fi

        if [ $attempt -lt $MAX_RETRIES ]; then
            warn "Download failed, retrying in ${RETRY_DELAY}s..."
            sleep $RETRY_DELAY
            RETRY_DELAY=$((RETRY_DELAY * 2))  # 指數退避
        fi

        attempt=$((attempt + 1))
    done

    return 1
}

# 嘗試下載
if ! download_with_retry "$DOWNLOAD_URL" "$TEMP_FILE"; then
    error "Failed to download Orban Agent after $MAX_RETRIES attempts"
    echo ""
    echo "Troubleshooting steps:"
    echo "1. Check your internet connection"
    echo "2. Verify GitHub is accessible: curl -I https://github.com"
    echo "3. Try manual download:"
    echo "   wget $DOWNLOAD_URL -O /tmp/orban-agent"
    echo "   chmod +x /tmp/orban-agent"
    echo "   sudo mv /tmp/orban-agent $INSTALL_DIR/orban-agent"
    echo ""
    exit 1
fi

if [ ! -f "$TEMP_FILE" ]; then
    error "Downloaded file not found"
    exit 1
fi

chmod +x "$TEMP_FILE"
success "Downloaded Orban Agent"

# 安裝二進制文件
info "Installing to $INSTALL_DIR..."

if [ -w "$INSTALL_DIR" ]; then
    mv "$TEMP_FILE" "$INSTALL_DIR/orban-agent"
else
    sudo mv "$TEMP_FILE" "$INSTALL_DIR/orban-agent"
fi

success "Installed Orban Agent"

# 創建配置目錄
CONFIG_DIR="$HOME/.orban-agent"
mkdir -p "$CONFIG_DIR"

# 創建 systemd 服務（可選）
info "Setting up systemd service..."

if command -v systemctl &> /dev/null; then
    SERVICE_FILE="$HOME/.config/systemd/user/orban-agent.service"
    mkdir -p "$(dirname "$SERVICE_FILE")"

    cat > "$SERVICE_FILE" << EOF
[Unit]
Description=Orban GPU Agent
After=network.target

[Service]
Type=simple
ExecStart=$INSTALL_DIR/orban-agent start
Restart=always
RestartSec=10

[Install]
WantedBy=default.target
EOF

    systemctl --user daemon-reload
    success "Systemd service created"

    echo ""
    read -p "Enable auto-start on boot? (y/n) " -n 1 -r
    echo ""
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        systemctl --user enable orban-agent
        success "Auto-start enabled"
    fi
fi

# 完成
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
success "Installation completed successfully!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Next steps:"
echo ""
echo "  1. Start the agent:"
echo "     $ orban-agent start"
echo ""
echo "  2. Check status:"
echo "     $ orban-agent status"
echo ""
echo "  3. View earnings:"
echo "     $ orban-agent earnings"
echo ""
echo "  4. Open dashboard (if using Tauri UI):"
echo "     $ orban-agent ui"
echo ""
echo "For more information, visit: https://docs.orban.ai"
echo ""

# 提供快速啟動選項
echo ""
read -p "Start the agent now? (y/n) " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    info "Starting Orban Agent..."
    orban-agent start &
    success "Agent started in background"
    echo ""
    info "Check status with: orban-agent status"
fi

echo ""
