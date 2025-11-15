#
# Orban Agent 一鍵安裝腳本 (Windows PowerShell)
#
# 使用方式：
#   iwr -useb https://raw.githubusercontent.com/orbanplatform/orban-agent/main/install.ps1 | iex
#   或
#   Invoke-WebRequest -Uri https://raw.githubusercontent.com/orbanplatform/orban-agent/main/install.ps1 -UseBasicParsing | Invoke-Expression
#

$ErrorActionPreference = "Stop"

# 顏色函數
function Write-Info($message) {
    Write-Host "ℹ $message" -ForegroundColor Blue
}

function Write-Success($message) {
    Write-Host "✓ $message" -ForegroundColor Green
}

function Write-Error-Custom($message) {
    Write-Host "✗ $message" -ForegroundColor Red
}

function Write-Warn($message) {
    Write-Host "⚠ $message" -ForegroundColor Yellow
}

# 打印橫幅
function Print-Banner {
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "  🚀 Orban Agent Installer v1.0.0" -ForegroundColor Cyan
    Write-Host "  Contribute your GPU, earn rewards" -ForegroundColor Cyan
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host ""
}

# 檢測平台
function Detect-Platform {
    $arch = $env:PROCESSOR_ARCHITECTURE

    switch ($arch) {
        "AMD64" { $script:archType = "x86_64" }
        "ARM64" { $script:archType = "aarch64" }
        default {
            Write-Error-Custom "Unsupported architecture: $arch"
            exit 1
        }
    }

    $script:platform = "windows-$script:archType"
    Write-Success "Detected platform: $script:platform"
}

# 下載二進制文件
function Download-Binary {
    Write-Info "Downloading Orban Agent for $script:platform..."

    $githubRepo = "orbanplatform/orban-agent"
    $releaseUrl = "https://github.com/$githubRepo/releases/latest/download/orban-agent-$script:platform.exe"

    $tempFile = "$env:TEMP\orban-agent-$PID.exe"

    try {
        Invoke-WebRequest -Uri $releaseUrl -OutFile $tempFile -UseBasicParsing
        Write-Success "Downloaded Orban Agent"
        $script:binarySource = $tempFile
    }
    catch {
        Write-Error-Custom "Failed to download from: $releaseUrl"
        Write-Host ""
        Write-Warn "GitHub Release 可能還在構建中，或者版本不存在"
        Write-Warn "將從源碼編譯安裝（首次安裝可能需要 5-10 分鐘）..."
        Write-Host ""
        Build-FromSource
    }
}

# 從源碼構建
function Build-FromSource {
    Write-Info "Building Orban Agent from source..."

    # 檢查 Git
    if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
        Write-Error-Custom "Git is not installed"
        Write-Host ""
        Write-Host "Please install Git first:"
        Write-Host "  Download from: https://git-scm.com/download/win"
        Write-Host "  Or use winget: winget install Git.Git"
        exit 1
    }

    # 檢查 Rust
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Warn "Rust is not installed. Installing Rust automatically..."
        Write-Host ""

        # 下載並安裝 Rust
        try {
            $rustupUrl = "https://win.rustup.rs/x86_64"
            $rustupInstaller = "$env:TEMP\rustup-init.exe"

            Write-Info "Downloading Rust installer..."
            Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupInstaller -UseBasicParsing

            Write-Info "Installing Rust (this may take a few minutes)..."
            Start-Process -FilePath $rustupInstaller -ArgumentList "-y" -Wait -NoNewWindow

            # 更新環境變量
            $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

            Write-Success "Rust installed successfully"
        }
        catch {
            Write-Error-Custom "Failed to install Rust"
            Write-Host ""
            Write-Host "Please install Rust manually from: https://rustup.rs/"
            exit 1
        }
    }

    # 克隆倉庫
    $tempDir = "$env:TEMP\orban-agent-build-$PID"
    Write-Info "Cloning repository to $tempDir..."

    try {
        git clone --depth 1 https://github.com/orbanplatform/orban-agent.git $tempDir
    }
    catch {
        Write-Error-Custom "Failed to clone repository"
        exit 1
    }

    # 構建
    Push-Location "$tempDir\agent-core"

    Write-Info "Building release binary (this may take 5-10 minutes on first build)..."

    try {
        cargo build --release

        # 複製二進制文件
        $builtBinary = "$tempDir\agent-core\target\release\orban-agent.exe"
        $finalTemp = "$env:TEMP\orban-agent-final-$PID.exe"
        Copy-Item -Path $builtBinary -Destination $finalTemp

        Pop-Location

        # 清理構建目錄
        Remove-Item -Path $tempDir -Recurse -Force

        $script:binarySource = $finalTemp
        Write-Success "Built Orban Agent from source"
    }
    catch {
        Pop-Location
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
        Write-Error-Custom "Build failed"
        exit 1
    }
}

# 安裝二進制文件
function Install-Binary {
    Write-Info "Installing Orban Agent..."

    # 安裝目錄 - 使用用戶目錄以避免需要管理員權限
    $installDir = "$env:USERPROFILE\.orban\bin"

    # 創建目錄
    if (-not (Test-Path $installDir)) {
        New-Item -ItemType Directory -Path $installDir -Force | Out-Null
    }

    # 複製文件
    $destPath = "$installDir\orban-agent.exe"
    Copy-Item -Path $script:binarySource -Destination $destPath -Force

    Write-Success "Installed to: $destPath"

    # 添加到 PATH
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$installDir*") {
        Write-Info "Adding $installDir to PATH..."
        [Environment]::SetEnvironmentVariable(
            "Path",
            "$userPath;$installDir",
            "User"
        )
        # 更新當前會話的 PATH
        $env:Path = "$env:Path;$installDir"
        Write-Success "Added to PATH (restart terminal to take effect)"
    }
}

# 創建配置目錄
function Setup-Config {
    $configDir = "$env:USERPROFILE\.orban-agent"

    if (-not (Test-Path $configDir)) {
        New-Item -ItemType Directory -Path $configDir -Force | Out-Null
    }

    if (-not (Test-Path "$configDir\logs")) {
        New-Item -ItemType Directory -Path "$configDir\logs" -Force | Out-Null
    }

    Write-Success "Created config directory: $configDir"
}

# 驗證安裝
function Verify-Installation {
    Write-Info "Verifying installation..."

    # 刷新 PATH
    $env:Path = [Environment]::GetEnvironmentVariable("Path", "User") + ";" + [Environment]::GetEnvironmentVariable("Path", "Machine")

    $agentPath = "$env:USERPROFILE\.orban\bin\orban-agent.exe"

    if (Test-Path $agentPath) {
        try {
            $versionOutput = & $agentPath version 2>&1
            if ($versionOutput -match "Version:\s*(.+)") {
                $version = $matches[1].Trim()
                Write-Success "orban-agent $version installed successfully!"
                return $true
            }
        }
        catch {
            Write-Error-Custom "Installation verification failed: $_"
            return $false
        }
    }

    Write-Error-Custom "Installation verification failed"
    return $false
}

# 顯示後續步驟
function Show-NextSteps {
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "✓ Installation completed successfully!" -ForegroundColor Green
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Next steps:"
    Write-Host ""
    Write-Host "  1. Restart your terminal or run:" -ForegroundColor Yellow
    Write-Host "     `$env:Path = [Environment]::GetEnvironmentVariable('Path', 'User') + ';' + [Environment]::GetEnvironmentVariable('Path', 'Machine')"
    Write-Host ""
    Write-Host "  2. Start the agent:"
    Write-Host "     orban-agent start" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  3. Check status:"
    Write-Host "     orban-agent status" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  4. View earnings:"
    Write-Host "     orban-agent earnings" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  5. View logs:"
    Write-Host "     orban-agent logs" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "For more information, visit: https://docs.orban.ai"
    Write-Host ""
}

# 主函數
function Main {
    Print-Banner
    Detect-Platform
    Download-Binary
    Install-Binary
    Setup-Config

    if (Verify-Installation) {
        Show-NextSteps
    }
    else {
        Write-Error-Custom "Installation failed. Please check the errors above."
        exit 1
    }
}

# 運行
Main
