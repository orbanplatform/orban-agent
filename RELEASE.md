# 🚀 Orban Agent 發布指南

這個文件說明如何發布 Orban Agent，讓任何人都可以下載和使用。

## 📋 發布前檢查清單

### 1. 代碼準備

- [x] ✅ CLI 已完全實現
- [x] ✅ 所有命令已測試
- [x] ✅ 編譯無錯誤
- [ ] 更新版本號（在 `agent-core/Cargo.toml`）
- [ ] 更新 CHANGELOG.md

### 2. 文檔準備

- [x] ✅ README.md 已更新
- [x] ✅ QUICKSTART.md 已創建
- [x] ✅ 安裝腳本已創建

### 3. 自動化準備

- [x] ✅ GitHub Actions workflow 已配置
- [x] ✅ 支持多平台構建
  - Linux x86_64
  - Linux aarch64
  - macOS x86_64 (Intel)
  - macOS aarch64 (Apple Silicon)
  - Windows x86_64

## 🔄 發布流程

### 步驟 1: 提交所有更改

```bash
git add .
git commit -m "chore: Prepare for v1.0.0 release"
git push origin main
```

### 步驟 2: 創建 Git Tag

```bash
# 創建標籤
git tag -a v1.0.0 -m "Release v1.0.0: Initial CLI release"

# 推送標籤
git push origin v1.0.0
```

### 步驟 3: 自動構建

GitHub Actions 會自動：
1. ✅ 在多個平台上構建二進制文件
2. ✅ 創建 GitHub Release
3. ✅ 上傳所有平台的二進制文件
4. ✅ 生成 Release Notes

查看進度：https://github.com/orbanplatform/orban-agent/actions

### 步驟 4: 驗證發布

```bash
# 檢查 Release 是否創建成功
# https://github.com/orbanplatform/orban-agent/releases

# 測試下載和安裝
curl -fsSL https://raw.githubusercontent.com/orbanplatform/orban-agent/main/install.sh | bash
```

## 📦 發布後的資產

發布後，以下文件將可供下載：

```
orban-agent-linux-x86_64        # Linux Intel/AMD
orban-agent-linux-aarch64       # Linux ARM
orban-agent-macos-x86_64        # macOS Intel
orban-agent-macos-aarch64       # macOS Apple Silicon (M1/M2/M3)
orban-agent-windows-x86_64.exe  # Windows
```

## 🧪 測試安裝

### 本地測試

```bash
# 運行本地測試腳本
./test-local-install.sh
```

### 測試實際安裝（發布後）

```bash
# Linux/macOS
curl -fsSL https://raw.githubusercontent.com/orbanplatform/orban-agent/main/install.sh | bash

# 驗證
orban-agent version
orban-agent status
```

## 📝 用戶安裝指南

發布後，用戶可以通過以下方式安裝：

### 方法 1: 一鍵安裝（推薦）

```bash
curl -fsSL https://raw.githubusercontent.com/orbanplatform/orban-agent/main/install.sh | bash
```

### 方法 2: 直接下載二進制文件

```bash
# 選擇適合您平台的版本
wget https://github.com/orbanplatform/orban-agent/releases/latest/download/orban-agent-linux-x86_64

# 安裝
chmod +x orban-agent-linux-x86_64
sudo mv orban-agent-linux-x86_64 /usr/local/bin/orban-agent

# 驗證
orban-agent version
```

### 方法 3: 從源碼構建

```bash
# 克隆倉庫
git clone https://github.com/orbanplatform/orban-agent.git
cd orban-agent/agent-core

# 構建
cargo build --release

# 安裝
sudo cp target/release/orban-agent /usr/local/bin/

# 驗證
orban-agent version
```

## 🔧 故障排除

### GitHub Actions 構建失敗

1. 檢查 Actions 日誌：https://github.com/orbanplatform/orban-agent/actions
2. 常見問題：
   - 依賴缺失
   - 交叉編譯問題
   - 權限問題

### Release 未創建

1. 確認 tag 已推送：`git tag -l`
2. 確認 workflow 文件正確：`.github/workflows/release.yml`
3. 檢查 GitHub token 權限

### 二進制文件無法運行

1. 檢查平台是否匹配
2. 檢查文件權限：`chmod +x orban-agent`
3. 檢查依賴：
   - Linux: `ldd orban-agent`
   - macOS: `otool -L orban-agent`

## 📊 發布後檢查

- [ ] 所有平台的二進制文件都已上傳
- [ ] Release Notes 正確生成
- [ ] 安裝腳本可以下載最新版本
- [ ] 文檔連結都正確
- [ ] 在至少一個平台上測試安裝

## 🎯 下一步

發布成功後：

1. **宣傳**
   - 在 Discord 發布公告
   - 在 Twitter 分享
   - 更新官網

2. **監控**
   - 關注 GitHub Issues
   - 收集用戶反饋
   - 監控下載量

3. **維護**
   - 修復 bugs
   - 添加新功能
   - 定期發布更新

## 🔄 持續發布

每次發布新版本時：

```bash
# 1. 更新版本號
vim agent-core/Cargo.toml

# 2. 更新 CHANGELOG
vim CHANGELOG.md

# 3. 提交
git add .
git commit -m "chore: Bump version to v1.0.1"

# 4. 創建標籤
git tag v1.0.1

# 5. 推送
git push origin main
git push origin v1.0.1
```

GitHub Actions 會自動處理剩下的工作！🎉

## 📞 獲取幫助

如果遇到問題：
- 查看 GitHub Actions 日誌
- 提交 Issue
- 聯繫團隊：dev@orban.ai

---

**祝發布順利！** 🚀
