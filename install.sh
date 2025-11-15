#!/bin/bash
#
# Orban Agent 一鍵安裝腳本
#
# 使用方式：
#   curl -fsSL https://raw.githubusercontent.com/orbanplatform/orban-agent/main/install.sh | bash
#

set -e

# 顏色輸出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
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

# 打印橫幅
print_banner() {
    echo ""
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}  🚀 Orban Agent Installer v1.0.0${NC}"
    echo -e "${CYAN}  Contribute your GPU, earn rewards${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
}

# 檢測作業系統和架構
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)     OS_TYPE="linux" ;;
        Darwin*)    OS_TYPE="macos" ;;
        MINGW*|MSYS*|CYGWIN*) OS_TYPE="windows" ;;
        *)
            error "Unsupported operating system: $OS"
            exit 1
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64)   ARCH_TYPE="x86_64" ;;
        aarch64|arm64)  ARCH_TYPE="aarch64" ;;
        *)
            error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac

    PLATFORM="${OS_TYPE}-${ARCH_TYPE}"
    success "Detected platform: ${PLATFORM}"
}

# 下載二進制文件
download_binary() {
    info "Downloading Orban Agent for ${PLATFORM}..."

    # GitHub Release URL
    GITHUB_REPO="orbanplatform/orban-agent"
    RELEASE_URL="https://github.com/${GITHUB_REPO}/releases/latest/download/orban-agent-${PLATFORM}"

    # 臨時文件
    TEMP_FILE="/tmp/orban-agent-$$"

    # 下載
    if command -v curl &> /dev/null; then
        if ! curl -fsSL "${RELEASE_URL}" -o "${TEMP_FILE}"; then
            error "Failed to download from: ${RELEASE_URL}"
            echo ""
            warn "GitHub Release 可能還在構建中，或者版本不存在"
            warn "將從源碼編譯安裝（首次安裝可能需要 5-10 分鐘）..."
            echo ""
            build_from_source
            return
        fi
    elif command -v wget &> /dev/null; then
        if ! wget -q "${RELEASE_URL}" -O "${TEMP_FILE}"; then
            error "Failed to download from: ${RELEASE_URL}"
            echo ""
            warn "GitHub Release 可能還在構建中，或者版本不存在"
            warn "將從源碼編譯安裝（首次安裝可能需要 5-10 分鐘）..."
            echo ""
            build_from_source
            return
        fi
    else
        error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi

    chmod +x "${TEMP_FILE}"
    success "Downloaded Orban Agent"

    BINARY_SOURCE="${TEMP_FILE}"
}

# 從源碼構建
build_from_source() {
    info "Building Orban Agent from source..."

    # 檢查 Git
    if ! command -v git &> /dev/null; then
        error "Git is not installed"
        echo ""
        echo "Please install Git first:"
        case "$OS_TYPE" in
            linux)
                echo "  Ubuntu/Debian: sudo apt install git"
                echo "  CentOS/RHEL:   sudo yum install git"
                ;;
            macos)
                echo "  Run: xcode-select --install"
                echo "  Or install via Homebrew: brew install git"
                ;;
        esac
        exit 1
    fi

    # 檢查 Rust
    if ! command -v cargo &> /dev/null; then
        warn "Rust is not installed. Installing Rust automatically..."
        echo ""

        # 自動安裝 Rust (非互動模式)
        if ! curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
            error "Failed to install Rust"
            echo ""
            echo "Please install Rust manually from: https://rustup.rs/"
            exit 1
        fi

        # 載入 Rust 環境
        source "$HOME/.cargo/env"
        success "Rust installed successfully"
    fi

    # 克隆倉庫
    TEMP_DIR="/tmp/orban-agent-build-$$"
    info "Cloning repository to ${TEMP_DIR}..."

    if ! git clone --depth 1 https://github.com/orbanplatform/orban-agent.git "${TEMP_DIR}"; then
        error "Failed to clone repository"
        exit 1
    fi

    cd "${TEMP_DIR}/agent-core"

    # 構建
    info "Building release binary (this may take 5-10 minutes on first build)..."
    if ! cargo build --release; then
        error "Build failed"
        cd - > /dev/null
        rm -rf "${TEMP_DIR}"
        exit 1
    fi

    # 複製二進制文件到臨時位置
    BUILT_BINARY="${TEMP_DIR}/agent-core/target/release/orban-agent"
    FINAL_TEMP="/tmp/orban-agent-final-$$"
    cp "${BUILT_BINARY}" "${FINAL_TEMP}"

    # 清理構建目錄（節省空間）
    cd - > /dev/null
    rm -rf "${TEMP_DIR}"

    BINARY_SOURCE="${FINAL_TEMP}"
    success "Built Orban Agent from source"
}

# 安裝二進制文件
install_binary() {
    info "Installing Orban Agent..."

    # 安裝目錄
    if [ "$OS_TYPE" = "windows" ]; then
        INSTALL_DIR="$HOME/.orban/bin"
    else
        INSTALL_DIR="/usr/local/bin"
    fi

    # 創建目錄（如果不存在）
    if [ ! -d "$INSTALL_DIR" ]; then
        info "Creating directory: $INSTALL_DIR"
        if [ -w "$(dirname "$INSTALL_DIR")" ]; then
            mkdir -p "$INSTALL_DIR"
        else
            sudo mkdir -p "$INSTALL_DIR"
        fi
    fi

    # 複製文件
    if [ -w "$INSTALL_DIR" ]; then
        cp "${BINARY_SOURCE}" "${INSTALL_DIR}/orban-agent"
    else
        sudo cp "${BINARY_SOURCE}" "${INSTALL_DIR}/orban-agent"
    fi

    # 添加到 PATH (如果需要)
    if [ "$INSTALL_DIR" = "$HOME/.orban/bin" ]; then
        case "$SHELL" in
            */bash)
                RC_FILE="$HOME/.bashrc"
                ;;
            */zsh)
                RC_FILE="$HOME/.zshrc"
                ;;
            *)
                RC_FILE="$HOME/.profile"
                ;;
        esac

        if ! grep -q ".orban/bin" "$RC_FILE" 2>/dev/null; then
            echo 'export PATH="$HOME/.orban/bin:$PATH"' >> "$RC_FILE"
            export PATH="$HOME/.orban/bin:$PATH"
            warn "Added $INSTALL_DIR to PATH in $RC_FILE"
            warn "Run: source $RC_FILE"
        fi
    fi

    success "Installed to: ${INSTALL_DIR}/orban-agent"
}

# 創建配置目錄
setup_config() {
    CONFIG_DIR="$HOME/.orban-agent"
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$CONFIG_DIR/logs"

    success "Created config directory: $CONFIG_DIR"
}

# 驗證安裝
verify_installation() {
    info "Verifying installation..."

    if command -v orban-agent &> /dev/null; then
        VERSION=$(orban-agent version 2>&1 | grep "Version:" | awk '{print $2}')
        success "orban-agent ${VERSION} installed successfully!"
        return 0
    elif [ -f "/usr/local/bin/orban-agent" ]; then
        VERSION=$(/usr/local/bin/orban-agent version 2>&1 | grep "Version:" | awk '{print $2}')
        success "orban-agent ${VERSION} installed successfully!"
        return 0
    else
        error "Installation verification failed"
        return 1
    fi
}

# 顯示後續步驟
show_next_steps() {
    echo ""
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✓ Installation completed successfully!${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "Next steps:"
    echo ""
    echo "  1. Start the agent:"
    echo -e "     ${CYAN}orban-agent start${NC}"
    echo ""
    echo "  2. Check status:"
    echo -e "     ${CYAN}orban-agent status${NC}"
    echo ""
    echo "  3. View earnings:"
    echo -e "     ${CYAN}orban-agent earnings${NC}"
    echo ""
    echo "  4. View logs:"
    echo -e "     ${CYAN}orban-agent logs${NC}"
    echo ""
    echo "For more information, visit: https://docs.orban.ai"
    echo ""
}

# 主函數
main() {
    print_banner
    detect_platform
    download_binary
    install_binary
    setup_config

    if verify_installation; then
        show_next_steps
    else
        error "Installation failed. Please check the errors above."
        exit 1
    fi
}

# 運行
main
