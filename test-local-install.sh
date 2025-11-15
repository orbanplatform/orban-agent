#!/bin/bash
#
# 測試本地安裝腳本
# 用於在發布前測試安裝流程
#

set -e

echo "=== Testing Local Installation ==="
echo ""

# 構建 release 版本
echo "1. Building release binary..."
cd agent-core
cargo build --release
cd ..

# 複製到臨時目錄模擬下載
echo ""
echo "2. Simulating download..."
TEMP_DIR="/tmp/orban-agent-test-$$"
mkdir -p "$TEMP_DIR"
cp agent-core/target/release/orban-agent "$TEMP_DIR/orban-agent-test"
chmod +x "$TEMP_DIR/orban-agent-test"

# 測試二進制文件
echo ""
echo "3. Testing binary..."
"$TEMP_DIR/orban-agent-test" version
echo ""

# 測試所有命令
echo "4. Testing all commands..."
echo ""

echo "  - Testing status..."
"$TEMP_DIR/orban-agent-test" status
echo ""

echo "  - Testing earnings..."
"$TEMP_DIR/orban-agent-test" earnings
echo ""

echo "  - Testing help..."
"$TEMP_DIR/orban-agent-test" --help
echo ""

# 清理
echo "5. Cleaning up..."
rm -rf "$TEMP_DIR"

echo ""
echo "=== ✓ All tests passed! ==="
echo ""
echo "Next steps:"
echo "  1. Create a git tag: git tag v1.0.0"
echo "  2. Push the tag: git push origin v1.0.0"
echo "  3. GitHub Actions will automatically build and release"
echo ""
