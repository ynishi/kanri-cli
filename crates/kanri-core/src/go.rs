use std::env;
use std::fs;
use std::path::PathBuf;

use crate::{
    cleanable::{Cleanable, CleanableItem},
    utils, Result,
};

/// Go モジュールキャッシュ情報
#[derive(Debug, Clone)]
pub struct GoModCache {
    /// キャッシュディレクトリのパス
    pub cache_dir: PathBuf,
    /// サイズ（バイト）
    pub size: u64,
}

/// Go モジュールキャッシュを検索
pub fn find_go_mod_cache() -> Result<Option<GoModCache>> {
    // GOMODCACHE 環境変数を確認
    let cache_dir = if let Ok(gomodcache) = env::var("GOMODCACHE") {
        PathBuf::from(gomodcache)
    } else if let Ok(gopath) = env::var("GOPATH") {
        PathBuf::from(gopath).join("pkg").join("mod")
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join("go").join("pkg").join("mod")
    } else {
        return Ok(None);
    };

    if !cache_dir.exists() {
        return Ok(None);
    }

    let size = utils::calculate_dir_size(&cache_dir)?;

    Ok(Some(GoModCache { cache_dir, size }))
}

/// Go モジュールキャッシュを削除
pub fn clean_mod_cache(cache: &GoModCache) -> Result<()> {
    if cache.cache_dir.exists() {
        fs::remove_dir_all(&cache.cache_dir)?;
    }
    Ok(())
}

/// Go クリーナー
pub struct GoCleaner;

impl GoCleaner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GoCleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl Cleanable for GoCleaner {
    fn scan(&self) -> Result<Vec<CleanableItem>> {
        if let Some(cache) = find_go_mod_cache()? {
            Ok(vec![CleanableItem::new(
                "Go module cache".to_string(),
                cache.cache_dir,
                cache.size,
            )])
        } else {
            Ok(Vec::new())
        }
    }

    fn name(&self) -> &str {
        "Go"
    }

    fn icon(&self) -> &str {
        "🐹"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_go_mod_cache() {
        // 環境依存なので、エラーが出ないことだけ確認
        let result = find_go_mod_cache();
        assert!(result.is_ok());
    }
}
