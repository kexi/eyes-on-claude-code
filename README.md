# Claude Monitor - プロトタイプ

グローバルHooksを使用して、全てのClaude Codeセッションからのイベントを監視するプロトタイプです。

## ファイル構成

```
~/.local/bin/
  └── claude-monitor-hook    # Hookスクリプト（イベント受信・ログ記録）
  └── claude-monitor-watch   # ログ監視スクリプト

~/.claude/
  └── settings.json          # グローバルHooks設定

~/.claude-monitor/
  └── logs/
      ├── events.jsonl       # イベントログ（JSONL形式）
      ├── console.log        # 人間可読ログ
      └── latest.json        # 最新イベント（デバッグ用）
```

## セットアップ手順

### 1. スクリプトをインストール

```bash
# ディレクトリ作成
mkdir -p ~/.local/bin ~/.claude-monitor/logs

# スクリプトをコピー（このファイルと同じディレクトリにある場合）
cp claude-monitor-hook ~/.local/bin/
cp claude-monitor-watch ~/.local/bin/

# 実行権限を付与
chmod +x ~/.local/bin/claude-monitor-hook
chmod +x ~/.local/bin/claude-monitor-watch

# PATHに追加（必要に応じて）
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

### 2. グローバルHooks設定を適用

**既存の設定がない場合:**
```bash
cp claude-monitor-settings.json ~/.claude/settings.json
```

**既存の設定がある場合:**
手動で `~/.claude/settings.json` に `hooks` セクションをマージしてください。

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
# リアルタイム監視
claude-monitor-watch

# または直接tail
tail -f ~/.claude-monitor/logs/console.log
```

## 使い方

### ログ監視

```bash
# リアルタイム監視（デフォルト）
claude-monitor-watch

# 最新20行を表示
claude-monitor-watch -n 20

# JSON形式で表示
claude-monitor-watch -n 10 -j

# サマリー表示
claude-monitor-watch -s
```

### ログファイル直接参照

```bash
# 人間可読ログ
cat ~/.claude-monitor/logs/console.log

# JSONログ（jqで整形）
cat ~/.claude-monitor/logs/events.jsonl | jq '.'

# 最新イベント
cat ~/.claude-monitor/logs/latest.json | jq '.'
```

### ログのクリア

```bash
# 全ログをクリア
rm -f ~/.claude-monitor/logs/*.jsonl ~/.claude-monitor/logs/*.log ~/.claude-monitor/logs/*.json
```

## イベントタイプ

| イベント | 絵文字 | 説明 |
|----------|--------|------|
| `notification` (permission_prompt) | 🔐 | 許可待ち |
| `notification` (idle_prompt) | ⏳ | 入力待ち（60秒以上アイドル） |
| `stop` | ✅ | 応答完了 |
| `session_start` | 🚀 | セッション開始 |
| `session_end` | 🏁 | セッション終了 |

## ログ出力例

### console.log
```
[14:32:15] 🚀 my-project: session_start (startup)
[14:32:45] 🔐 my-project: notification (permission_prompt) - Claude needs your permission to use Bash
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
  "raw_input": { ... }
}
```

## トラブルシューティング

### Hooksが動作しない場合

1. Claude Codeで `/hooks` コマンドを実行して設定を確認
2. 設定ファイルのJSONが有効か確認: `jq '.' ~/.claude/settings.json`
3. スクリプトの実行権限を確認: `ls -la ~/.local/bin/claude-monitor-*`

### ログが記録されない場合

1. ログディレクトリの存在確認: `ls -la ~/.claude-monitor/logs/`
2. 手動でスクリプトをテスト:
   ```bash
   echo '{"session_id":"test","message":"test message"}' | \
     CLAUDE_PROJECT_DIR=/tmp/test ~/.local/bin/claude-monitor-hook notification permission_prompt
   cat ~/.claude-monitor/logs/latest.json
   ```

## 次のステップ

このプロトタイプで動作確認ができたら：

1. **デーモン化** - ログファイルではなくHTTP/WebSocketでイベントを受信
2. **デスクトップアプリ** - Tauriでリアルタイム表示・通知
3. **状態管理** - セッション状態のトラッキングと永続化
