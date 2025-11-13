use std::fs;
use std::path::PathBuf;

use crate::{
    cleanable::{Cleanable, CleanableItem, CleanableMetadata},
    utils, Result,
};

/// Mac アプリケーションキャッシュ情報
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// キャッシュディレクトリ名
    pub name: String,
    /// キャッシュディレクトリのパス
    pub path: PathBuf,
    /// キャッシュサイズ（バイト）
    pub size: u64,
    /// 削除が安全かどうか
    pub is_safe: bool,
}

impl CacheEntry {
    /// サイズを人間が読みやすい形式で取得
    pub fn formatted_size(&self) -> String {
        utils::format_size(self.size)
    }

    /// 安全性ラベルを取得
    pub fn safety_label(&self) -> &str {
        if self.is_safe {
            "✓ 安全"
        } else {
            "⚠ 要確認"
        }
    }
}

/// 削除しても安全なことが知られているキャッシュのリスト
const SAFE_CACHE_PATTERNS: &[&str] = &[
    "Homebrew",
    "pip",
    "yarn",
    "npm",
    "pnpm",
    "CocoaPods",
    "com.apple.bird",          // iCloud sync
    "com.apple.metal",         // Metal shader cache
    "com.spotify.client",
    "Google/Chrome",
    "Firefox",
    "com.microsoft.VSCode",
    "JetBrains",
    "Slack",
    "Discord",
    "com.docker.docker",
    "Xcode/DerivedData",
];

/// キャッシュエントリが安全かどうかチェック
fn is_safe_cache(name: &str) -> bool {
    SAFE_CACHE_PATTERNS.iter().any(|pattern| name.contains(pattern))
}

/// ユーザーの Library/Caches ディレクトリをスキャン
///
/// `min_size_gb`: 最小サイズ（GB単位）。これより小さいキャッシュは無視
pub fn scan_user_caches(min_size_gb: u64) -> Result<Vec<CacheEntry>> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
    let cache_dir = PathBuf::from(home).join("Library/Caches");

    if !cache_dir.exists() {
        return Ok(Vec::new());
    }

    let min_size_bytes = min_size_gb * 1024 * 1024 * 1024;
    let mut entries = Vec::new();

    for entry in fs::read_dir(&cache_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // サイズを計算
            let size = utils::calculate_dir_size(&path)?;

            // 最小サイズ以上の場合のみ追加
            if size >= min_size_bytes {
                let name = entry.file_name().to_string_lossy().to_string();
                let is_safe = is_safe_cache(&name);

                entries.push(CacheEntry {
                    name,
                    path,
                    size,
                    is_safe,
                });
            }
        }
    }

    // サイズの大きい順にソート
    entries.sort_by(|a, b| b.size.cmp(&a.size));

    Ok(entries)
}

/// キャッシュエントリを削除
pub fn clean_cache(entry: &CacheEntry) -> Result<()> {
    if entry.path.exists() {
        fs::remove_dir_all(&entry.path)?;
    }
    Ok(())
}

/// 複数のキャッシュエントリを削除
pub fn clean_caches(entries: &[CacheEntry]) -> Result<Vec<String>> {
    let mut cleaned = Vec::new();

    for entry in entries {
        clean_cache(entry)?;
        cleaned.push(entry.name.clone());
    }

    Ok(cleaned)
}

/// Mac キャッシュクリーナー
pub struct CacheCleaner {
    pub min_size_gb: u64,
    pub safe_only: bool,
}

impl CacheCleaner {
    pub fn new(min_size_gb: u64, safe_only: bool) -> Self {
        Self {
            min_size_gb,
            safe_only,
        }
    }
}

impl Cleanable for CacheCleaner {
    fn scan(&self) -> Result<Vec<CleanableItem>> {
        let mut caches = scan_user_caches(self.min_size_gb)?;

        if self.safe_only {
            caches.retain(|c| c.is_safe);
        }

        Ok(caches
            .into_iter()
            .map(|c| {
                let metadata = CleanableMetadata {
                    is_safe: Some(c.is_safe),
                    safety_label: Some(c.safety_label().to_string()),
                };
                CleanableItem::with_metadata(c.name, c.path, c.size, metadata)
            })
            .collect())
    }

    fn name(&self) -> &str {
        "Mac Cache"
    }

    fn icon(&self) -> &str {
        "💾"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_safe_cache() {
        assert!(is_safe_cache("Homebrew"));
        assert!(is_safe_cache("com.spotify.client"));
        assert!(is_safe_cache("Google/Chrome"));
        assert!(!is_safe_cache("com.apple.Safari"));
        assert!(!is_safe_cache("some.random.app"));
    }

    #[test]
    fn test_scan_user_caches() {
        // このテストは環境依存なので、エラーが出ないことだけ確認
        let result = scan_user_caches(1);
        assert!(result.is_ok());
    }
}
