#!/bin/bash
# Eyes on Claude Code インストールスクリプト
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="${HOME}/Applications"
APP_NAME="Eyes on Claude Code.app"

echo "Eyes on Claude Code インストーラー"
echo "==================================="

# 1. Rust確認
echo ""
echo "[1/5] Rust環境を確認中..."
if ! command -v cargo &> /dev/null; then
    echo "Rustがインストールされていません。"
    echo "以下のコマンドでインストールしてください:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi
echo "Rust: $(rustc --version)"

# 2. npm install
echo "[2/5] npm依存関係をインストール中..."
cd "${SCRIPT_DIR}/app"
npm install

# 3. ビルド
echo "[3/5] Tauriアプリをビルド中..."
npm run build

# 4. ~/Applications にインストール
echo "[4/5] ${APP_DIR} にインストール中..."
mkdir -p "${APP_DIR}"
rm -rf "${APP_DIR}/${APP_NAME}"
cp -r "src-tauri/target/release/bundle/macos/${APP_NAME}" "${APP_DIR}/"

# 5. CLI ツールをインストール
echo "[5/5] CLI ツールをインストール中..."
mkdir -p ~/.local/bin
cp "${SCRIPT_DIR}/eocc-hook" ~/.local/bin/
cp "${SCRIPT_DIR}/eocc-watch" ~/.local/bin/
chmod +x ~/.local/bin/eocc-hook
chmod +x ~/.local/bin/eocc-watch

echo ""
echo "インストール完了!"
echo ""
echo "次のステップ:"
echo "  1. ~/.claude/settings.json に hooks を設定"
echo "     (eocc-settings.json を参照)"
echo ""
echo "  2. アプリを起動:"
echo "     open ~/Applications/Eyes\ on\ Claude\ Code.app"
echo ""
echo "  3. ログイン時に自動起動するには:"
echo "     システム設定 > 一般 > ログイン項目 に追加"
