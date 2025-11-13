# kanri（管理）

Mac のローカル環境を管理・クリーンアップする CLI ツール

## 機能

- **Rust プロジェクト**: `target/` ディレクトリの検索・削除
- **Docker**: 未使用イメージ・コンテナ・ボリュームの削除
- **Node.js**: `node_modules/` の検索・削除
- その他、開発に溜まりがちな不要ファイルの一括クリーンアップ

## インストール

```bash
cargo install --path crates/kanri-cli
```

## 使い方

### Rust プロジェクトのクリーンアップ

```bash
# 検索・表示のみ（デフォルト）
kanri clean rust
kanri clean rust --search  # 明示的に指定
kanri clean rust -s        # 短縮形

# 削除を実行
kanri clean rust --delete
kanri clean rust -d

# 確認しながら削除
kanri clean rust --interactive
kanri clean rust -i

# 特定のディレクトリを検索
kanri clean rust -p ~/projects
```

### Node.js プロジェクトのクリーンアップ

```bash
# 検索・表示のみ（デフォルト）
kanri clean node
kanri clean node -s

# 削除を実行
kanri clean node -d

# 確認しながら削除
kanri clean node -i

# 特定のディレクトリを検索
kanri clean node -p ~/projects
```

### Docker のクリーンアップ

```bash
# 検索・表示のみ（デフォルト）
kanri clean docker

# 削除を実行
kanri clean docker -d

# 確認しながら削除
kanri clean docker -i

# 未使用イメージもすべて削除
kanri clean docker -d --all

# ボリュームも削除
kanri clean docker -d --volumes

# すべてのオプションを指定
kanri clean docker -d --all --volumes
```

## 開発

```bash
# ビルド
cargo build

# テスト
cargo test

# 実行
cargo run -p kanri-cli -- --help
```

## プロジェクト構造

```
kanri/
├── Cargo.toml              # Workspace 設定
└── crates/
    ├── kanri-cli/          # CLI エントリーポイント
    └── kanri-core/         # コア機能
```

## ライセンス

MIT OR Apache-2.0
