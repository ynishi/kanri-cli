use std::env;
use std::fs;
use std::path::PathBuf;

use crate::{
    cleanable::{Cleanable, CleanableItem},
    utils, Result,
};

/// Gradle キャッシュ情報
#[derive(Debug, Clone)]
pub struct GradleCache {
    /// キャッシュディレクトリのパス
    pub cache_dir: PathBuf,
    /// サイズ（バイト）
    pub size: u64,
}

/// Gradle キャッシュを検索
pub fn find_gradle_cache() -> Result<Option<GradleCache>> {
    // GRADLE_USER_HOME 環境変数を確認
    let cache_dir = if let Ok(gradle_home) = env::var("GRADLE_USER_HOME") {
        PathBuf::from(gradle_home)
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".gradle")
    } else {
        return Ok(None);
    };

    if !cache_dir.exists() {
        return Ok(None);
    }

    let size = utils::calculate_dir_size(&cache_dir)?;

    Ok(Some(GradleCache { cache_dir, size }))
}

/// Gradle キャッシュを削除
pub fn clean_gradle_cache(cache: &GradleCache) -> Result<()> {
    if cache.cache_dir.exists() {
        fs::remove_dir_all(&cache.cache_dir)?;
    }
    Ok(())
}

/// Gradle クリーナー
pub struct GradleCleaner;

impl GradleCleaner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GradleCleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl Cleanable for GradleCleaner {
    fn scan(&self) -> Result<Vec<CleanableItem>> {
        if let Some(cache) = find_gradle_cache()? {
            Ok(vec![CleanableItem::new(
                "Gradle cache".to_string(),
                cache.cache_dir,
                cache.size,
            )])
        } else {
            Ok(Vec::new())
        }
    }

    fn name(&self) -> &str {
        "Gradle"
    }

    fn icon(&self) -> &str {
        "🐘"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_gradle_cache() {
        // 環境依存なので、エラーが出ないことだけ確認
        let result = find_gradle_cache();
        assert!(result.is_ok());
    }
}
