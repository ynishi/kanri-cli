use std::env;
use std::fs;
use std::path::PathBuf;

use crate::{
    cleanable::{Cleanable, CleanableItem},
    utils, Result,
};

/// Xcode DerivedData 情報
#[derive(Debug, Clone)]
pub struct XcodeDerivedData {
    /// DerivedData ディレクトリのパス
    pub derived_data_dir: PathBuf,
    /// サイズ（バイト）
    pub size: u64,
}

/// Xcode DerivedData を検索
pub fn find_xcode_derived_data() -> Result<Option<XcodeDerivedData>> {
    let derived_data_dir = if let Ok(home) = env::var("HOME") {
        PathBuf::from(home)
            .join("Library")
            .join("Developer")
            .join("Xcode")
            .join("DerivedData")
    } else {
        return Ok(None);
    };

    if !derived_data_dir.exists() {
        return Ok(None);
    }

    let size = utils::calculate_dir_size(&derived_data_dir)?;

    Ok(Some(XcodeDerivedData {
        derived_data_dir,
        size,
    }))
}

/// Xcode DerivedData を削除
pub fn clean_derived_data(data: &XcodeDerivedData) -> Result<()> {
    if data.derived_data_dir.exists() {
        fs::remove_dir_all(&data.derived_data_dir)?;
    }
    Ok(())
}

/// Xcode クリーナー
pub struct XcodeCleaner;

impl XcodeCleaner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for XcodeCleaner {
    fn default() -> Self {
        Self::new()
    }
}

impl Cleanable for XcodeCleaner {
    fn scan(&self) -> Result<Vec<CleanableItem>> {
        if let Some(data) = find_xcode_derived_data()? {
            Ok(vec![CleanableItem::new(
                "Xcode DerivedData".to_string(),
                data.derived_data_dir,
                data.size,
            )])
        } else {
            Ok(Vec::new())
        }
    }

    fn name(&self) -> &str {
        "Xcode"
    }

    fn icon(&self) -> &str {
        "🔨"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_xcode_derived_data() {
        // 環境依存なので、エラーが出ないことだけ確認
        let result = find_xcode_derived_data();
        assert!(result.is_ok());
    }
}
