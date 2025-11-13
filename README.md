# kanri（管理）

Mac のローカル環境を管理・クリーンアップする CLI ツール

## 機能

- **Rust プロジェクト**: `target/` ディレクトリの検索・削除
- **Node.js プロジェクト**: `node_modules/` の検索・削除
- **Docker**: 未使用イメージ・コンテナ・ボリュームの削除
- **Mac キャッシュ** (⚠️ Experimental): `~/Library/Caches/` の大容量キャッシュの検索・削除（1GB以上）

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

### Mac アプリケーションキャッシュのクリーンアップ (⚠️ Experimental)

```bash
# 検索・表示のみ（デフォルト、1GB以上）
kanri clean cache

# 安全なキャッシュのみ表示
kanri clean cache --safe-only

# 最小サイズを指定（2GB以上）
kanri clean cache --min-size 2

# 確認しながら削除（推奨）
kanri clean cache -i

# 削除を実行（注意！）
kanri clean cache -d
```

**注意事項**:
- この機能は実験的です。削除前に必ず内容を確認してください
- `✓ 安全` マークは一般的に削除しても問題ないキャッシュです
- `⚠ 要確認` マークは慎重に判断してください
- アプリケーションによっては再ダウンロードが必要になる場合があります

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
        ├── cleanable.rs    # Cleanable trait（拡張可能な設計）
        ├── rust.rs         # Rust クリーナー
        ├── node.rs         # Node.js クリーナー
        ├── docker.rs       # Docker クリーナー
        └── cache.rs        # Mac キャッシュクリーナー
```

## 拡張方法：新しいクリーナーの追加

`Cleanable` trait を実装すれば、新しいクリーナーを簡単に追加できます。

### 例：Python 仮想環境クリーナー

```rust
use kanri_core::{Cleanable, CleanableItem, Result};
use std::path::PathBuf;

pub struct PythonCleaner {
    pub search_path: PathBuf,
}

impl Cleanable for PythonCleaner {
    fn scan(&self) -> Result<Vec<CleanableItem>> {
        // venv, .venv ディレクトリを検索
        // ...
        Ok(items)
    }

    fn name(&self) -> &str {
        "Python"
    }

    fn icon(&self) -> &str {
        "🐍"
    }
}
```

### Cleanable trait の利点

- **統一されたインターフェース**: すべてのクリーナーが同じパターンで動作
- **再利用可能**: 共通機能（サイズ計算、削除処理）を共有
- **拡張容易**: 新しいクリーナーの追加が簡単
- **メタデータ対応**: 安全性情報などの追加情報をサポート

## ライセンス

MIT OR Apache-2.0
