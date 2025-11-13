use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{utils, Result};

/// Rust プロジェクト情報
#[derive(Debug, Clone)]
pub struct RustProject {
    /// プロジェクトのルートディレクトリ（Cargo.toml があるディレクトリ）
    pub root: PathBuf,
    /// target ディレクトリのパス
    pub target_dir: PathBuf,
    /// target ディレクトリのサイズ（バイト）
    pub size: u64,
}

impl RustProject {
    /// target ディレクトリが存在するかチェック
    pub fn target_exists(&self) -> bool {
        self.target_dir.exists()
    }

    /// サイズを人間が読みやすい形式で取得
    pub fn formatted_size(&self) -> String {
        utils::format_size(self.size)
    }
}

/// 指定されたディレクトリ以下の Rust プロジェクトを検索
pub fn find_rust_projects(search_path: &Path) -> Result<Vec<RustProject>> {
    let mut projects = Vec::new();

    for entry in WalkDir::new(search_path)
        .into_iter()
        .filter_entry(|e| {
            // target, .git, node_modules などの大きなディレクトリはスキップ
            let file_name = e.file_name().to_string_lossy();
            !matches!(
                file_name.as_ref(),
                "target" | ".git" | "node_modules" | ".cache"
            )
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.file_name() == "Cargo.toml" {
            if let Some(project_root) = entry.path().parent() {
                let target_dir = project_root.join("target");

                // target ディレクトリが存在する場合のみ追加
                if target_dir.exists() {
                    let size = utils::calculate_dir_size(&target_dir)?;

                    projects.push(RustProject {
                        root: project_root.to_path_buf(),
                        target_dir,
                        size,
                    });
                }
            }
        }
    }

    Ok(projects)
}

/// Rust プロジェクトの target ディレクトリを削除
pub fn clean_project(project: &RustProject) -> Result<()> {
    if project.target_exists() {
        fs::remove_dir_all(&project.target_dir)?;
    }
    Ok(())
}

/// 複数の Rust プロジェクトをクリーン
pub fn clean_projects(projects: &[RustProject]) -> Result<Vec<PathBuf>> {
    let mut cleaned = Vec::new();

    for project in projects {
        clean_project(project)?;
        cleaned.push(project.root.clone());
    }

    Ok(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_rust_projects() -> Result<()> {
        let temp = TempDir::new()?;
        let project_dir = temp.path().join("test-project");
        fs::create_dir(&project_dir)?;

        // Cargo.toml を作成
        fs::write(project_dir.join("Cargo.toml"), "[package]\nname = \"test\"")?;

        // target ディレクトリを作成
        let target_dir = project_dir.join("target");
        fs::create_dir(&target_dir)?;
        fs::write(target_dir.join("test.txt"), "test data")?;

        // プロジェクトを検索
        let projects = find_rust_projects(temp.path())?;

        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].root, project_dir);
        assert!(projects[0].size > 0);

        Ok(())
    }

    #[test]
    fn test_clean_project() -> Result<()> {
        let temp = TempDir::new()?;
        let project_dir = temp.path().join("test-project");
        fs::create_dir(&project_dir)?;

        let target_dir = project_dir.join("target");
        fs::create_dir(&target_dir)?;
        fs::write(target_dir.join("test.txt"), "test data")?;

        let project = RustProject {
            root: project_dir.clone(),
            target_dir: target_dir.clone(),
            size: 100,
        };

        assert!(target_dir.exists());

        clean_project(&project)?;

        assert!(!target_dir.exists());

        Ok(())
    }
}
