use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{
    cleanable::{Cleanable, CleanableItem},
    utils, Result,
};

/// 大きなファイル・ディレクトリ情報
#[derive(Debug, Clone)]
pub struct LargeItem {
    /// アイテムのパス
    pub path: PathBuf,
    /// サイズ（バイト）
    pub size: u64,
    /// ディレクトリかどうか
    pub is_dir: bool,
}

/// 大きなファイル・ディレクトリを検索
pub fn find_large_items(
    search_path: &Path,
    min_size: u64,
    extensions: Option<&[String]>,
    include_dirs: bool,
    include_files: bool,
) -> Result<Vec<LargeItem>> {
    let mut items = Vec::new();

    // 他のクリーナーで管理されるディレクトリを除外
    let excluded_dirs = [
        "node_modules",
        "target",
        ".git",
        ".stack-work",
        "dist",
        "dist-newstyle",
        "__pycache__",
    ];

    for entry in WalkDir::new(search_path)
        .into_iter()
        .filter_entry(|e| {
            let file_name = e.file_name().to_string_lossy();
            !excluded_dirs.contains(&file_name.as_ref())
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_dir = metadata.is_dir();
        let is_file = metadata.is_file();

        // ディレクトリかファイルかでフィルタ
        if (is_dir && !include_dirs) || (is_file && !include_files) {
            continue;
        }

        // 拡張子フィルタ（ファイルのみ）
        if is_file {
            if let Some(exts) = extensions {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_with_dot = format!(".{}", ext);
                    if !exts.iter().any(|e| e == &ext_with_dot || e == ext) {
                        continue;
                    }
                } else {
                    // 拡張子フィルタが指定されているのに拡張子がない場合はスキップ
                    continue;
                }
            }
        }

        // サイズ計算
        let size = if is_dir {
            match utils::calculate_dir_size(path) {
                Ok(s) => s,
                Err(_) => continue,
            }
        } else {
            metadata.len()
        };

        // 検索パス自身は除外（サブディレクトリのみを対象とする）
        if path == search_path {
            continue;
        }

        // サイズ閾値でフィルタ
        if size >= min_size {
            items.push(LargeItem {
                path: path.to_path_buf(),
                size,
                is_dir,
            });
        }
    }

    // サイズ順にソート（大きい順）
    items.sort_by(|a, b| b.size.cmp(&a.size));

    Ok(items)
}

/// 大きなファイル・ディレクトリクリーナー
pub struct LargeFilesCleaner {
    pub search_path: PathBuf,
    pub min_size: u64,
    pub extensions: Option<Vec<String>>,
    pub include_dirs: bool,
    pub include_files: bool,
}

impl LargeFilesCleaner {
    pub fn new(search_path: PathBuf, min_size: u64) -> Self {
        Self {
            search_path,
            min_size,
            extensions: None,
            include_dirs: true,
            include_files: true,
        }
    }

    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = Some(extensions);
        self
    }

    pub fn with_include_dirs(mut self, include_dirs: bool) -> Self {
        self.include_dirs = include_dirs;
        self
    }

    pub fn with_include_files(mut self, include_files: bool) -> Self {
        self.include_files = include_files;
        self
    }
}

impl Cleanable for LargeFilesCleaner {
    fn scan(&self) -> Result<Vec<CleanableItem>> {
        let items = find_large_items(
            &self.search_path,
            self.min_size,
            self.extensions.as_deref(),
            self.include_dirs,
            self.include_files,
        )?;

        Ok(items
            .into_iter()
            .map(|item| {
                let type_label = if item.is_dir { "dir" } else { "file" };
                let name = format!(
                    "{} ({})",
                    item.path.display(),
                    type_label
                );
                CleanableItem::new(name, item.path, item.size)
            })
            .collect())
    }

    fn name(&self) -> &str {
        "Large Files"
    }

    fn icon(&self) -> &str {
        "📦"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_large_files() -> Result<()> {
        let temp = TempDir::new()?;
        let test_dir = temp.path();

        // 3GB のファイルを作成（実際には書き込まずサイズだけ設定）
        let large_file = test_dir.join("model.ckpt");
        let file = fs::File::create(&large_file)?;
        file.set_len(3 * 1024 * 1024 * 1024)?; // 3GB

        // 1GB のファイル（閾値以下）
        let small_file = test_dir.join("small.txt");
        let file = fs::File::create(&small_file)?;
        file.set_len(1024 * 1024 * 1024)?; // 1GB

        // 2GB 閾値で検索
        let items = find_large_items(
            test_dir,
            2 * 1024 * 1024 * 1024,
            None,
            false,
            true,
        )?;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, large_file);
        assert_eq!(items[0].size, 3 * 1024 * 1024 * 1024);
        assert!(!items[0].is_dir);

        Ok(())
    }

    #[test]
    fn test_find_large_files_with_extension_filter() -> Result<()> {
        let temp = TempDir::new()?;
        let test_dir = temp.path();

        // 3GB の .ckpt ファイル
        let ckpt_file = test_dir.join("model.ckpt");
        let file = fs::File::create(&ckpt_file)?;
        file.set_len(3 * 1024 * 1024 * 1024)?;

        // 3GB の .txt ファイル
        let txt_file = test_dir.join("data.txt");
        let file = fs::File::create(&txt_file)?;
        file.set_len(3 * 1024 * 1024 * 1024)?;

        // .ckpt のみをフィルタ
        let extensions = vec![".ckpt".to_string()];
        let items = find_large_items(
            test_dir,
            2 * 1024 * 1024 * 1024,
            Some(&extensions),
            false,
            true,
        )?;

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].path, ckpt_file);

        Ok(())
    }

    #[test]
    fn test_find_large_dirs() -> Result<()> {
        let temp = TempDir::new()?;
        let test_dir = temp.path();

        // 検索用のプロジェクトディレクトリを作成
        let projects_dir = test_dir.join("projects");
        fs::create_dir(&projects_dir)?;

        // 大きなディレクトリを作成
        let large_dir = projects_dir.join("large_data");
        fs::create_dir(&large_dir)?;

        // ディレクトリ内に複数のファイルを作成
        for i in 0..3 {
            let file = fs::File::create(large_dir.join(format!("file{}.bin", i)))?;
            file.set_len(1024 * 1024 * 1024)?; // 1GB each
        }

        // 小さなディレクトリも作成（検出されないはず）
        let small_dir = projects_dir.join("small_data");
        fs::create_dir(&small_dir)?;
        let file = fs::File::create(small_dir.join("file.txt"))?;
        file.set_len(100 * 1024 * 1024)?; // 100MB

        // ディレクトリのみを検索（4GB閾値でprojects_dirを除外）
        let items = find_large_items(
            &projects_dir,
            4 * 1024 * 1024 * 1024,
            None,
            true,
            false,
        )?;

        // large_dir は検出されないはず（3GBで4GB未満）
        assert_eq!(items.len(), 0);

        // 2GB閾値で再度検索
        let items = find_large_items(
            &projects_dir,
            2 * 1024 * 1024 * 1024,
            None,
            true,
            false,
        )?;

        // large_dir と projects_dir の両方が検出される可能性がある
        // large_dir のみが含まれることを確認
        let large_dir_found = items.iter().any(|item| item.path == large_dir);
        assert!(large_dir_found, "large_dir should be found");
        assert!(items.iter().all(|item| item.is_dir), "all items should be directories");

        // large_dirのサイズを確認
        let large_item = items.iter().find(|item| item.path == large_dir).unwrap();
        assert!(large_item.size >= 3 * 1024 * 1024 * 1024);

        Ok(())
    }
}
