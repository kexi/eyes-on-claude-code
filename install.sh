#!/bin/bash
# Claude Monitor インストールスクリプト
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="${HOME}/Applications"
APP_NAME="ClaudeMonitor.app"

echo "Claude Monitor インストーラー"
echo "=============================="

# 1. ビルド
echo ""
echo "[1/4] Swift アプリをビルド中..."
cd "${SCRIPT_DIR}/ClaudeMonitor"
swift build -c release

# 2. .app バンドル作成
echo "[2/4] アプリバンドルを作成中..."
rm -rf "${APP_NAME}"
mkdir -p "${APP_NAME}/Contents/MacOS"
cp .build/release/ClaudeMonitor "${APP_NAME}/Contents/MacOS/"
cat > "${APP_NAME}/Contents/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>ClaudeMonitor</string>
    <key>CFBundleIdentifier</key>
    <string>com.local.ClaudeMonitor</string>
    <key>CFBundleName</key>
    <string>Claude Monitor</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

# 3. ~/Applications にインストール
echo "[3/4] ${APP_DIR} にインストール中..."
mkdir -p "${APP_DIR}"
rm -rf "${APP_DIR}/${APP_NAME}"
cp -r "${APP_NAME}" "${APP_DIR}/"

# 4. CLI ツールをインストール
echo "[4/4] CLI ツールをインストール中..."
mkdir -p ~/.local/bin
cp "${SCRIPT_DIR}/claude-monitor-hook" ~/.local/bin/
cp "${SCRIPT_DIR}/claude-monitor-watch" ~/.local/bin/
chmod +x ~/.local/bin/claude-monitor-hook
chmod +x ~/.local/bin/claude-monitor-watch

echo ""
echo "インストール完了!"
echo ""
echo "次のステップ:"
echo "  1. ~/.claude/settings.json に hooks を設定"
echo "     (claude-monitor-settings.json を参照)"
echo ""
echo "  2. アプリを起動:"
echo "     open ~/Applications/ClaudeMonitor.app"
echo ""
echo "  3. ログイン時に自動起動するには:"
echo "     システム設定 > 一般 > ログイン項目 に追加"
