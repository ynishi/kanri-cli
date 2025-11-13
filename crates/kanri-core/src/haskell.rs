use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{
    cleanable::{Cleanable, CleanableItem},
    utils, Result,
};

/// Haskell ビルド成果物情報
#[derive(Debug, Clone)]
pub struct HaskellBuild {
    /// プロジェクトのルートディレクトリ
    pub root: PathBuf,
    /// ビルドディレクトリのパス
    pub build_dir: PathBuf,
    /// サイズ（バイト）
    pub size: u64,
    /// ビルドタイプ（stack-work, dist, dist-newstyle）
    pub build_type: String,
}

/// 指定されたディレクトリ以下の Haskell ビルド成果物を検索
pub fn find_haskell_builds(search_path: &Path) -> Result<Vec<HaskellBuild>> {
    let mut builds = Vec::new();

    for entry in WalkDir::new(search_path)
        .into_iter()
        .filter_entry(|e| {
            let file_name = e.file_name().to_string_lossy();
            // .stack-work, dist, dist-newstyle は検索対象なので除外しない
            !matches!(
                file_name.as_ref(),
                "target" | ".git" | "node_modules" | ".cache"
            )
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy();

        // .stack-work, dist, dist-newstyle ディレクトリを検出
        if entry.file_type().is_dir()
            && matches!(
                file_name.as_ref(),
                ".stack-work" | "dist" | "dist-newstyle"
            )
        {
            if let Some(project_root) = path.parent() {
                // Haskell プロジェクトか確認（*.cabal または stack.yaml の存在）
                let has_cabal = project_root
                    .read_dir()
                    .ok()
                    .and_then(|mut entries| {
                        entries.any(|e| {
                            e.ok()
                                .and_then(|e| {
                                    e.path()
                                        .extension()
                                        .and_then(|ext| ext.to_str())
                                        .map(|ext| ext == "cabal")
                                })
                                .unwrap_or(false)
                        })
                        .then_some(())
                    })
                    .is_some();

                let has_stack_yaml = project_root.join("stack.yaml").exists();

                if has_cabal || has_stack_yaml {
                    let size = utils::calculate_dir_size(path)?;

                    builds.push(HaskellBuild {
                        root: project_root.to_path_buf(),
                        build_dir: path.to_path_buf(),
                        size,
                        build_type: file_name.to_string(),
                    });
                }
            }
        }
    }

    Ok(builds)
}

/// Haskell ビルド成果物を削除
pub fn clean_build(build: &HaskellBuild) -> Result<()> {
    if build.build_dir.exists() {
        fs::remove_dir_all(&build.build_dir)?;
    }
    Ok(())
}

/// Haskell クリーナー
pub struct HaskellCleaner {
    pub search_path: PathBuf,
}

impl HaskellCleaner {
    pub fn new(search_path: PathBuf) -> Self {
        Self { search_path }
    }
}

impl Cleanable for HaskellCleaner {
    fn scan(&self) -> Result<Vec<CleanableItem>> {
        let builds = find_haskell_builds(&self.search_path)?;

        Ok(builds
            .into_iter()
            .map(|b| {
                CleanableItem::new(
                    format!("{} ({})", b.root.display(), b.build_type),
                    b.build_dir,
                    b.size,
                )
            })
            .collect())
    }

    fn name(&self) -> &str {
        "Haskell"
    }

    fn icon(&self) -> &str {
        "λ"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_find_haskell_builds() -> Result<()> {
        let temp = TempDir::new()?;
        let project_dir = temp.path().join("test-project");
        fs::create_dir(&project_dir)?;

        // Cabal ファイルを作成
        fs::write(project_dir.join("test.cabal"), "name: test")?;

        // .stack-work ディレクトリを作成
        let stack_work_dir = project_dir.join(".stack-work");
        fs::create_dir(&stack_work_dir)?;
        fs::write(stack_work_dir.join("test.txt"), "test")?;

        let builds = find_haskell_builds(temp.path())?;

        assert_eq!(builds.len(), 1);
        assert_eq!(builds[0].root, project_dir);
        assert_eq!(builds[0].build_type, ".stack-work");

        Ok(())
    }
}
