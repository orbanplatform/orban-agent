# Building Orban Agent

本文檔說明如何從源碼建置 Orban Agent。

## 前置需求

### 1. Rust 工具鏈

```bash
# 安裝 Rust (使用 rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 確認安裝
rustc --version
cargo --version
```

### 2. GPU 驅動和工具

#### NVIDIA GPU (Linux)

```bash
# Ubuntu/Debian
sudo apt-get install nvidia-driver-535 nvidia-utils-535

# Fedora/RHEL
sudo dnf install nvidia-driver nvidia-settings

# 驗證
nvidia-smi
```

#### AMD GPU (Linux)

```bash
# 安裝 ROCm
wget https://repo.radeon.com/amdgpu-install/latest/ubuntu/jammy/amdgpu-install_*.deb
sudo dpkg -i amdgpu-install_*.deb
sudo amdgpu-install --usecase=rocm

# 驗證
rocm-smi
```

### 3. 系統依賴

#### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git
```

#### macOS

```bash
# 使用 Homebrew
brew install pkg-config openssl
```

#### Windows

需要安裝：
- Visual Studio 2019+ (包含 C++ 工具)
- Git for Windows

## 建置步驟

### 1. Clone 代碼庫

```bash
git clone https://github.com/orbanplatform/orban-agent.git
cd orban-agent/agent/core
```

### 2. 選擇 Features

根據您的 GPU 類型選擇對應的 features：

```bash
# NVIDIA GPU (預設)
cargo build --release

# AMD GPU
cargo build --release --features amd --no-default-features

# Apple Silicon
cargo build --release --features apple --no-default-features

# 無 Docker 支援
cargo build --release --no-default-features --features nvidia

# 完整功能
cargo build --release --features "nvidia,amd,apple,docker"
```

### 3. 執行測試

```bash
# 單元測試
cargo test

# GPU 測試 (需要實體 GPU)
cargo test --features gpu-tests

# 整合測試
cargo test --test integration
```

### 4. 安裝

```bash
# 安裝到系統
cargo install --path .

# 或手動複製
sudo cp target/release/orban-agent /usr/local/bin/
```

## 開發建置

### 開發模式

```bash
# Debug 建置（更快，但效能較差）
cargo build

# 執行
cargo run -- start

# 監聽文件變更自動重建
cargo install cargo-watch
cargo watch -x run
```

### Linting 和格式化

```bash
# 格式化代碼
cargo fmt

# Linting
cargo clippy

# 修復 clippy 建議
cargo clippy --fix
```

## 常見問題

### Q: 找不到 NVML 庫

**A**: 確保安裝了 NVIDIA 驅動和 CUDA Toolkit

```bash
# Ubuntu
sudo apt-get install nvidia-cuda-toolkit

# 設定環境變數
export LD_LIBRARY_PATH=/usr/local/cuda/lib64:$LD_LIBRARY_PATH
```

### Q: 編譯失敗：找不到 OpenSSL

**A**: 安裝 OpenSSL 開發庫

```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev

# macOS
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)
```

### Q: Docker 權限錯誤

**A**: 將使用者加入 docker 群組

```bash
sudo usermod -aG docker $USER
# 登出並重新登入生效
```

### Q: 建置很慢

**A**: 使用更多 CPU 核心編譯

```bash
# 使用 4 個核心
cargo build --release -j 4

# 或設定環境變數
export CARGO_BUILD_JOBS=4
```

## 交叉編譯

### Linux → Windows

```bash
# 安裝目標
rustup target add x86_64-pc-windows-gnu

# 建置
cargo build --release --target x86_64-pc-windows-gnu
```

### Linux → macOS

需要使用 osxcross 工具鏈（較複雜，建議在 macOS 上直接編譯）

### 使用 Docker 建置

```bash
# 建置 Docker 映像
docker build -t orban-agent-builder .

# 在容器中編譯
docker run --rm -v $(pwd):/workspace orban-agent-builder cargo build --release
```

## 效能優化建置

### 最大化效能

```bash
# 在 Cargo.toml 中設定
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = 'abort'

# 建置
cargo build --release
```

### 減小二進制大小

```bash
# 安裝 upx (可執行文件壓縮器)
sudo apt-get install upx

# 壓縮
upx --best --lzma target/release/orban-agent
```

## CI/CD

GitHub Actions 工作流程範例：

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    steps:
    - uses: actions/checkout@v2

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Build
      run: cargo build --release

    - name: Test
      run: cargo test

    - name: Upload artifact
      uses: actions/upload-artifact@v2
      with:
        name: orban-agent-${{ matrix.os }}
        path: target/release/orban-agent*
```

## 下一步

建置完成後：

1. 閱讀 [README.md](README.md) 了解使用方式
2. 查看 [DESIGN.md](DESIGN.md) 了解架構設計
3. 運行 `orban-agent --help` 查看所有命令

## 獲取幫助

- GitHub Issues: https://github.com/orbanplatform/orban-agent/issues
- Discord: https://discord.gg/orban
- Email: support@orban.ai
