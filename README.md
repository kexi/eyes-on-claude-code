# Eyes on Claude Code

グローバルHooksを使用して、全てのClaude Codeセッションからのイベントを監視するメニューバー常駐（Tauri）アプリです。

## ファイル構成

```
~/.local/bin/
  └── eocc-hook            # Hookスクリプト（アプリが自動配置するシンボリックリンク）

~/.claude/
  └── settings.json          # グローバルHooks設定

~/.eocc/
  └── logs/
      ├── events.jsonl       # イベントログ（JSONL形式）
      ├── console.log        # 人間可読ログ
      └── latest.json        # 最新イベント（デバッグ用）
      └── stdin-debug.log    # Hookのstdin入力デバッグ（必要時のみ）
```

## セットアップ手順

### 事前要件

- `jq`（必須。Hookが `events.jsonl` を生成するために使用します）
  - macOS: `brew install jq`

### 1. Hookをインストール（推奨：アプリ経由）

アプリ起動時に Hook を自動インストールし、`~/.local/bin/eocc-hook` を作成（シンボリックリンク）します。

1. `app/` から開発起動 or ビルド済みアプリを起動
2. 初回は Setup モーダルが表示されるので、生成された設定を `~/.claude/settings.json` に反映

### 2. グローバルHooks設定を適用（手動で行う場合）

```bash
# スクリプトをコピー（このリポジトリ直下にある場合）
mkdir -p ~/.local/bin
cp ./eocc-hook ~/.local/bin/eocc-hook

# 実行権限を付与
chmod +x ~/.local/bin/eocc-hook

# PATHに追加（必要に応じて）
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

`~/.claude/settings.json` に `hooks` セクションを追加/更新してください。
（既存設定がある場合は、他の設定を壊さないようにマージしてください。アプリの Setup 画面は「既存設定 + hooks」をマージしたJSONを生成できます。）

```bash
# 既存の設定を確認
cat ~/.claude/settings.json

# エディタで編集
code ~/.claude/settings.json  # または vim, nano など
```

### 3. Claude Code を再起動

Hooks設定は起動時に読み込まれるため、既存のClaude Codeセッションを再起動してください。

```bash
# 新しいセッションを開始
claude
```

### 4. 動作確認

別のターミナルでログを監視：

```bash
# 人間可読ログ
tail -f ~/.eocc/logs/console.log
```

## 使い方

### ログファイル直接参照

```bash
# 人間可読ログ
cat ~/.eocc/logs/console.log

# JSONログ（jqで整形）
cat ~/.eocc/logs/events.jsonl | jq '.'

# 最新イベント
cat ~/.eocc/logs/latest.json | jq '.'
```

### ログのクリア

```bash
# 全ログをクリア
rm -f ~/.eocc/logs/*.jsonl ~/.eocc/logs/*.log ~/.eocc/logs/*.json
```

## イベントタイプ

| イベント | 絵文字 | 説明 |
|----------|--------|------|
| `notification` (permission_prompt) | 🔐 | 許可待ち |
| `notification` (idle_prompt) | ⏳ | 入力待ち（60秒以上アイドル） |
| `stop` | ✅ | 応答完了 |
| `post_tool_use` | 🔧 | ツール実行後（状態更新用） |
| `user_prompt_submit` | 💬 | ユーザーがプロンプト送信 |
| `session_start` | 🚀 | セッション開始 |
| `session_end` | 🏁 | セッション終了 |

## ログ出力例

### console.log
```
[14:32:15] 🚀 my-project: session_start (startup)
[14:32:45] 🔐 my-project: notification (permission_prompt) - Claude needs your permission to use Bash
[14:32:58] 🔧 my-project: post_tool_use
[14:33:00] 💬 my-project: user_prompt_submit
[14:33:02] ✅ my-project: stop
[14:35:10] 🏁 my-project: session_end
```

### events.jsonl
```json
{
  "timestamp": "2025-01-02T05:32:15Z",
  "event": "session_start",
  "matcher": "startup",
  "project_name": "my-project",
  "project_dir": "/Users/you/projects/my-project",
  "session_id": "abc123",
  "message": "",
  "notification_type": "",
  "tool_name": "",
  "raw_input": { ... }
}
```

## トラブルシューティング

### Hooksが動作しない場合

1. Claude Codeで `/hooks` コマンドを実行して設定を確認
2. スクリプトの実行権限を確認: `ls -la ~/.local/bin/eocc-hook`
3. `jq` がインストールされているか確認: `command -v jq`

### ログが記録されない場合

1. ログディレクトリの存在確認: `ls -la ~/.eocc/logs/`
2. 手動でスクリプトをテスト:
   ```bash
   echo '{"session_id":"test","message":"test message"}' | \
     CLAUDE_PROJECT_DIR=/tmp/test ~/.local/bin/eocc-hook notification permission_prompt
   cat ~/.eocc/logs/latest.json
   ```

## デスクトップアプリ (Tauri)

`app/` にメニューバー常駐アプリがあります。

### 機能

- メニューバーにアイコン表示
- セッション状態のリアルタイム監視
- waiting数をバッジ/ツールチップ/メニュー表示で通知
- Recent Events サブメニュー
- ログフォルダを開く機能

### 必要環境

- Rust (rustup)
- Node.js
- pnpm（`tauri.conf.json` が `pnpm dev/build` を使用します）

### 開発環境セットアップ

```bash
# Rustのインストール（未インストールの場合）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# プロジェクトディレクトリに移動
cd app

# 依存関係インストール
pnpm install
```

### 開発モードで起動

```bash
# Cargo を PATH に追加（シェル起動時に自動で追加されない場合）
source "$HOME/.cargo/env"

cd app
pnpm dev
```

### リリースビルド

```bash
source "$HOME/.cargo/env"
cd app
pnpm build

# 成果物
# - src-tauri/target/release/bundle/macos/Eyes on Claude Code.app
# - src-tauri/target/release/bundle/dmg/Eyes on Claude Code_1.0.0_aarch64.dmg
```

### 動作確認手順

1. **アプリを起動**
   ```bash
   open src-tauri/target/release/bundle/macos/Eyes\ on\ Claude\ Code.app
   ```

2. **テストイベントを送信**
   ```bash
   # セッション開始
   echo '{"session_id": "test-001"}' | \
     CLAUDE_PROJECT_DIR="/path/to/project" \
     ~/.local/bin/eocc-hook session_start startup

   # permission待ち（アイコンがオレンジに変化）
   echo '{"session_id": "test-001", "notification_type": "permission_prompt"}' | \
     CLAUDE_PROJECT_DIR="/path/to/project" \
     ~/.local/bin/eocc-hook notification permission_prompt

   # 完了（アイコンがグレーに戻る）
   echo '{"session_id": "test-001"}' | \
     CLAUDE_PROJECT_DIR="/path/to/project" \
     ~/.local/bin/eocc-hook stop

   # セッション終了
   echo '{"session_id": "test-001"}' | \
     CLAUDE_PROJECT_DIR="/path/to/project" \
     ~/.local/bin/eocc-hook session_end
   ```

3. **メニューバーで確認**
   - アイコンをクリックしてメニューを表示
   - セッション一覧、Recent Events を確認

### ログファイルのリセット

```bash
rm ~/.eocc/logs/events.jsonl
touch ~/.eocc/logs/events.jsonl
```

### インストール

```bash
# または手動で ~/Applications にコピー
cp -r app/src-tauri/target/release/bundle/macos/Eyes\ on\ Claude\ Code.app ~/Applications/
```

### ログイン時に自動起動

システム設定 > 一般 > ログイン項目 に `Eyes on Claude Code.app` を追加
