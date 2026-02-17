# AGENTS.md — CoffeeTap MVP (Rust)

## 目的 / ゴール
CoffeeTap の「最小プロト」を Rust で実装する。
既存ブロックチェーン（当面 Solana 想定）を使い、オンチェーンは送金のみ。
アプリ側（Rust）は「支援先管理」「支援リンク生成」「Tx検証」「履歴保存」までを担当する。

### MVPで成立の条件
- クリエイター（受取先）の pubkey を登録できる
- 支援リンク（or URI）を生成できる
- 送金Txの署名(signature)から RPC でトランザクションを検証できる
- 検証OKなら履歴として保存できる（重複登録防止）
- すべてローカルで動く（CLI優先）

## スコープ（MVPでやらない）
- オンチェーンプログラム（スマコン）実装はしない
- 返礼NFT/トークン配布はしない
- KYC/決済規制対応はしない
- 不正対策の完全性は追わない（最低限：重複防止・受取先/金額チェック）

## 主要要件
### チェーン/通貨
- 優先：Solana（SOL送金）
- 可能なら次点：USDC(SPL)送金検証を拡張として追加

### 検証ロジック（必須）
`verify(signature)` は以下を満たすこと：
- 受取先が creator_pubkey である（または想定のトークンアカウント）
- 送金額が要求amount以上（MVPは「以上」でOK）
- すでに同じ signature が DB に存在しない（重複防止）

## 形（アーキテクチャ）
### 第一形態：CLI（最優先）
コマンド例：
- `coffeetap add-creator --name alice --pubkey <PUBKEY>`
- `coffeetap create-link --creator alice --amount 1 --currency sol`
- `coffeetap verify --signature <SIG>`
- `coffeetap history --creator alice`

### DB
- MVPは SQLite（ローカルファイル）でよい
- テーブル（例）
  - creators(id, name, pubkey, created_at)
  - taps(id, creator_id, currency, amount, signature UNIQUE, donor_pubkey, slot, created_at)

## 依存関係の方針
- CLI：clap
- DB：rusqlite（軽量で十分）
- Solana RPC：solana_client / solana_sdk
- ログ：tracing（必要最小限）
- エラー：thiserror / anyhow（どちらかに統一）

## コーディング規約
- まず動く最小実装 → その後に整える（過剰抽象化しない）
- public API は最小に、内部はモジュールで分ける
- 失敗時は「何が不足/不正か」をメッセージで明確に出す
- 可能な限り `Result<T, E>` を返し、panicしない

## ファイル構成（推奨）
- `src/main.rs`（CLIエントリ）
- `src/commands/*.rs`（add-creator, create-link, verify, history）
- `src/db.rs`（SQLite操作）
- `src/chain/solana.rs`（RPC/Tx解析）
- `src/models.rs`（struct定義）

## 動作確認コマンド（Codexはまずこれを実行）
- `cargo fmt`
- `cargo clippy -- -D warnings`
- `cargo test`

## 実装の進め方（優先順位）
1. CLI骨組み（clap）と `add-creator`（SQLite保存）
2. `verify`（signatureからTx取得 → 受取先チェック → 保存）
3. `history`（一覧表示）
4. `create-link`（支援URI/URL生成）
5. 余裕があれば USDC(SPL) 検証追加

## 注意
- RPCエンドポイントは環境変数で切替可能にする（例：SOLANA_RPC_URL）
- 「mainnet/devnet」はMVPでは devnet をデフォルト推奨
- シークレットキーをリポジトリに絶対入れない

## 出力のスタイル（日本語）
- コミットメッセージやCLIヘルプは簡潔に
- 端的に「次に何をすればいいか」も表示する
