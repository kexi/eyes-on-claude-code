# Eyes on Claude Code

Claude Code の **グローバルHooks** からイベントを収集し、複数プロジェクトのセッション状態を **メニューバー（トレイ）** と **ダッシュボード** で一覧できる Tauri アプリです。

[screenshot_ダッシュボードの全体（Sessions一覧と右上ステータス）]

---

## 1. setup

### 前提

- Claude Code
- Node.js v23+

### 初回起動

初回起動時にはアプリケーション上にhooksの設定のインストラクションが表示されます。

![hooks](https://github.com/user-attachments/assets/7b657f43-584a-4f57-83cd-d9587558d519)

Setupモーダルの指示に従って、生成された設定を `~/.claude/settings.json` に反映してください。


> [!Note]
> Setupモーダル上で表示されているJSONは既存設定を読み込み、**hooksだけを差し替える** JSON を生成します（古い eocc hooks は置換され、他のhooks/設定は保持されます）。

### 生成されるファイル/保存先

**Hook/ログ**

```
~/.local/bin/
  └── eocc-hook              # Hookスクリプト（アプリが作るシンボリックリンク）

~/.claude/
  └── settings.json          # グローバルHooks設定

~/.eocc/
  └── logs/
      └── events.jsonl       # イベントキュー（アプリ処理後に削除される）
```

**アプリ設定**

- アプリ設定ファイルは Tauri の `app_data_dir/settings.json` に保存されます（例: Always on Top / opacity / sound）。

---

## 2. 使い方

### ダッシュボード

- ダッシュボードには Sessions 一覧が表示されます。
- 右上のステータスは `Monitoring` または `N waiting` を表示します。

[screenshot_Sessions一覧（Waitingのセッションがハイライトされている状態）]

#### セッションカード

- カードをクリックすると展開し、Git状態と Diff 操作が表示されます。
- `waiting_for` がある場合は、カード内に補足（message/tool_name）が表示されます。

[screenshot_セッションカード展開（unstaged/staged/commit/branchのDiffボタン）]

#### Refresh

- `Refresh` で手動更新できます。
- バックアップとして **5秒ごとに自動更新**も行います。

### トレイ（メニューバー）メニュー

- waiting数、セッション一覧、Recent Events、Dashboard表示、ログフォルダを開く、セッションクリア等ができます。

[screenshot_トレイメニュー（waiting header / sessions / open dashboard / open log folder）]

### Diff（difit連携）

セッションカードの展開時に、次のDiffを開けます。

- `unstaged`: 作業ツリーの差分
- `staged`: indexの差分
- `commit`: 最新コミットの差分（`HEAD~1..HEAD`）
- `branch`: デフォルトブランチ（自動検出）との比較

注意:

- Diff表示は内部で `npx difit` を対象リポジトリ内で起動します。
  - Node.js（`npx`）が必要です。
  - 初回は `npx` が `difit` を取得する場合があり、環境によってはネットワークが必要です。
- 対象がGitリポジトリでない場合、または差分が無い場合は開けません。

### ウィンドウ操作/設定

- Always on Top の切替に対応します。
- フォーカス状態に応じてOpacityを変えられます。
  - difitウィンドウがフォーカスされるとダッシュボードは「非アクティブ」として扱われます。
- Sound をONにすると、状態変化に応じて効果音が鳴ります。
  - Waiting（Permission/Input）: 注意音
  - Completed: 完了音

---

## 3. サポート機能一覧

- 複数セッション監視: `~/.eocc/logs/events.jsonl` を監視して状態を集約
- 状態表示: Active / WaitingPermission / WaitingInput / Completed
- waiting数の可視化: トレイのツールチップ、ダッシュボードの表示、（対応環境では）バッジ
- Recent Events: 最新イベントをメニューから確認（最大50件保持、メニュー表示は直近10件）
- ログフォルダを開く: メニューから `~/.eocc/logs` を開く
- セッションクリア/削除:
  - トレイ: Clear Sessions
  - ダッシュボード: Remove session
- Git情報の表示: ブランチ名、unstaged/staged有無、最新コミット
- Diff表示: difit（`npx difit`）で差分を別ウィンドウに表示
- 通知音: waiting / completed を音で通知（SoundのON/OFFあり）

### 監視イベント（Hook）

Hookスクリプトはイベントを `events.jsonl` に追記します。

| event | 用途 | セッション状態への反映 |
|---|---|---|
| `session_start` (startup/resume) | セッション開始 | Activeとして登録 |
| `session_end` | セッション終了 | セッション削除 |
| `notification` (permission_prompt) | 承認待ち | WaitingPermission |
| `notification` (idle_prompt) | 入力待ち | WaitingInput |
| `stop` | 応答完了 | Completed |
| `post_tool_use` | ツール実行後 | Active |
| `user_prompt_submit` | プロンプト送信 | Active |

---

## 4. dev向けのinstruction

### 必要環境

- Rust（`rustup`）
- Node.js（`npx`含む）
- pnpm

### セットアップ

```bash
cd app
pnpm install
```

### 開発起動

```bash
cd app
pnpm tauri dev
```

補足:

- Vite dev server は `localhost:1420` を使用します（`app/vite.config.ts`）。

### ビルド

```bash
cd app
pnpm tauri build
```

成果物（macOS例）:

- `app/src-tauri/target/release/bundle/macos/Eyes on Claude Code.app`
- `app/src-tauri/target/release/bundle/dmg/Eyes on Claude Code_1.0.0_aarch64.dmg`

---

## 5. troubleshooting

### Setupモーダルが消えない（hooksがNGのまま）

- `~/.claude/settings.json` を開き、Setupモーダルで生成した内容に反映できているか確認
- Claude Codeで `/hooks` を実行して、hooksが読み込まれているか確認
- `~/.local/bin/eocc-hook` の存在/実行権限を確認
  - `ls -la ~/.local/bin/eocc-hook`
- Node.js が利用できるか確認
  - `command -v node`

### ログが増えない / セッションが表示されない

- `~/.eocc/logs/console.log` を確認
  - `tail -f ~/.eocc/logs/console.log`
- `~/.eocc/logs/stdin-debug.log` でHookが呼ばれているか確認
- `events.jsonl` が作成/更新されているか確認
  - `ls -la ~/.eocc/logs/`

### セッションが unknown になる

- 手動テスト等で `CLAUDE_PROJECT_DIR` が未設定だと `project_dir=unknown` になります。
- Claude Code実行経由のhooksで `project_dir` が入っているか `events.jsonl` を確認してください。

### Diffが開けない

- 対象がGitリポジトリか確認（`.git` が無いとエラーになります）
- 差分が無いと「No diff content to display」になり開けません
- Node.js（`npx`）が利用できるか確認
  - `command -v node && command -v npx`
- 環境によっては `npx` が `difit` を取得するためネットワークが必要です

### 音が鳴らない

- メニューの Sound がONになっているか確認
- ブラウザ/OS側の制限でAudioがブロックされる場合があります（設定変更後に再度状態変化を発生させて確認してください）

