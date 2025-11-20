use crate::Result;
use std::path::PathBuf;

/// クリーンアップ可能な項目のメタデータ
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct CleanableMetadata {
    /// 安全性フラグ（キャッシュクリーナーなどで使用）
    pub is_safe: Option<bool>,
    /// 安全性ラベル
    pub safety_label: Option<String>,
}


/// クリーンアップ可能な項目を表すtrait
pub trait Cleanable: Sized {
    /// 削除対象の項目を検索
    fn scan(&self) -> Result<Vec<CleanableItem>>;

    /// 名前（例: "kanri", "JetBrains"）
    fn name(&self) -> &str;

    /// アイコン（例: "🦀", "📦", "💾"）
    fn icon(&self) -> &str;
}

/// クリーンアップ可能な個別項目
#[derive(Debug, Clone)]
pub struct CleanableItem {
    /// 項目の名前
    pub name: String,
    /// 項目のパス
    pub path: PathBuf,
    /// サイズ（バイト）
    pub size: u64,
    /// メタデータ
    pub metadata: CleanableMetadata,
}

impl CleanableItem {
    /// 新しい CleanableItem を作成
    pub fn new(name: String, path: PathBuf, size: u64) -> Self {
        Self {
            name,
            path,
            size,
            metadata: CleanableMetadata::default(),
        }
    }

    /// メタデータ付きで新しい CleanableItem を作成
    pub fn with_metadata(
        name: String,
        path: PathBuf,
        size: u64,
        metadata: CleanableMetadata,
    ) -> Self {
        Self {
            name,
            path,
            size,
            metadata,
        }
    }

    /// サイズを人間が読みやすい形式で取得
    pub fn formatted_size(&self) -> String {
        crate::utils::format_size(self.size)
    }

    /// 安全性ラベルを取得
    pub fn safety_label(&self) -> Option<&str> {
        self.metadata.safety_label.as_deref()
    }

    /// 安全かどうか
    pub fn is_safe(&self) -> bool {
        self.metadata.is_safe.unwrap_or(true)
    }
}

/// 複数のアイテムをまとめて削除
pub fn clean_items(items: &[CleanableItem]) -> Result<Vec<String>> {
    let mut cleaned = Vec::new();

    for item in items {
        if item.path.exists() {
            std::fs::remove_dir_all(&item.path)?;
            cleaned.push(item.name.clone());
        }
    }

    Ok(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cleanable_item_new() {
        let item = CleanableItem::new("test".to_string(), PathBuf::from("/tmp/test"), 1024);

        assert_eq!(item.name, "test");
        assert_eq!(item.size, 1024);
        assert!(item.is_safe()); // デフォルトは安全
    }

    #[test]
    fn test_cleanable_item_with_metadata() {
        let metadata = CleanableMetadata {
            is_safe: Some(false),
            safety_label: Some("⚠ 要確認".to_string()),
        };

        let item = CleanableItem::with_metadata(
            "test".to_string(),
            PathBuf::from("/tmp/test"),
            1024,
            metadata,
        );

        assert!(!item.is_safe());
        assert_eq!(item.safety_label(), Some("⚠ 要確認"));
    }
}
