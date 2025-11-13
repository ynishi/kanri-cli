use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{cleanable::{Cleanable, CleanableItem}, utils, Result};

/// Node.js プロジェクト情報
#[derive(Debug, Clone)]
pub struct NodeProject {
    /// プロジェクトのルートディレクトリ（package.json があるディレクトリ）
    pub root: PathBuf,
    /// node_modules ディレクトリのパス
    pub node_modules_dir: PathBuf,
    /// node_modules ディレクトリのサイズ（バイト）
    pub size: u64,
}

impl NodeProject {
    /// node_modules ディレクトリが存在するかチェック
    pub fn node_modules_exists(&self) -> bool {
        self.node_modules_dir.exists()
    }

    /// サイズを人間が読みやすい形式で取得
    pub fn formatted_size(&self) -> String {
        utils::format_size(self.size)
    }
}

/// 指定されたディレクトリ以下の Node.js プロジェクトを検索
pub fn find_node_projects(search_path: &Path) -> Result<Vec<NodeProject>> {
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
        if entry.file_type().is_file() && entry.file_name() == "package.json" {
            if let Some(project_root) = entry.path().parent() {
                let node_modules_dir = project_root.join("node_modules");

                // node_modules ディレクトリが存在する場合のみ追加
                if node_modules_dir.exists() {
                    let size = utils::calculate_dir_size(&node_modules_dir)?;

                    projects.push(NodeProject {
                        root: project_root.to_path_buf(),
                        node_modules_dir,
                        size,
                    });
                }
            }
        }
    }

    Ok(projects)
}

/// Node.js プロジェクトの node_modules ディレクトリを削除
pub fn clean_project(project: &NodeProject) -> Result<()> {
    if project.node_modules_exists() {
        fs::remove_dir_all(&project.node_modules_dir)?;
    }
    Ok(())
}

/// 複数の Node.js プロジェクトをクリーン
pub fn clean_projects(projects: &[NodeProject]) -> Result<Vec<PathBuf>> {
    let mut cleaned = Vec::new();

    for project in projects {
        clean_project(project)?;
        cleaned.push(project.root.clone());
    }

    Ok(cleaned)
}

/// Node.js プロジェクトクリーナー
pub struct NodeCleaner {
    pub search_path: PathBuf,
}

impl NodeCleaner {
    pub fn new(search_path: PathBuf) -> Self {
        Self { search_path }
    }
}

impl Cleanable for NodeCleaner {
    fn scan(&self) -> Result<Vec<CleanableItem>> {
        let projects = find_node_projects(&self.search_path)?;

        Ok(projects
            .into_iter()
            .map(|p| CleanableItem::new(p.root.display().to_string(), p.node_modules_dir, p.size))
            .collect())
    }

    fn name(&self) -> &str {
        "Node.js"
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
    fn test_find_node_projects() -> Result<()> {
        let temp = TempDir::new()?;
        let project_dir = temp.path().join("test-project");
        fs::create_dir(&project_dir)?;

        // package.json を作成
        fs::write(
            project_dir.join("package.json"),
            r#"{"name": "test", "version": "1.0.0"}"#,
        )?;

        // node_modules ディレクトリを作成
        let node_modules_dir = project_dir.join("node_modules");
        fs::create_dir(&node_modules_dir)?;
        fs::write(node_modules_dir.join("test.txt"), "test data")?;

        // プロジェクトを検索
        let projects = find_node_projects(temp.path())?;

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

        let node_modules_dir = project_dir.join("node_modules");
        fs::create_dir(&node_modules_dir)?;
        fs::write(node_modules_dir.join("test.txt"), "test data")?;

        let project = NodeProject {
            root: project_dir.clone(),
            node_modules_dir: node_modules_dir.clone(),
            size: 100,
        };

        assert!(node_modules_dir.exists());

        clean_project(&project)?;

        assert!(!node_modules_dir.exists());

        Ok(())
    }
}
