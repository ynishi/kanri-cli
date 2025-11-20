use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{
    cleanable::{Cleanable, CleanableItem},
    utils, Result,
};

/// Flutter プロジェクト情報
#[derive(Debug, Clone)]
pub struct FlutterProject {
    /// プロジェクトのルートディレクトリ（pubspec.yaml があるディレクトリ）
    pub root: PathBuf,
    /// build ディレクトリのパス
    pub build_dir: PathBuf,
    /// .dart_tool ディレクトリのパス
    pub dart_tool_dir: PathBuf,
    /// 合計サイズ（バイト）
    pub size: u64,
}

impl FlutterProject {
    /// build ディレクトリが存在するかチェック
    pub fn build_exists(&self) -> bool {
        self.build_dir.exists()
    }

    /// .dart_tool ディレクトリが存在するかチェック
    pub fn dart_tool_exists(&self) -> bool {
        self.dart_tool_dir.exists()
    }

    /// サイズを人間が読みやすい形式で取得
    pub fn formatted_size(&self) -> String {
        utils::format_size(self.size)
    }
}

/// 指定されたディレクトリ以下の Flutter プロジェクトを検索
pub fn find_flutter_projects(search_path: &Path) -> Result<Vec<FlutterProject>> {
    let mut projects = Vec::new();

    for entry in WalkDir::new(search_path)
        .into_iter()
        .filter_entry(|e| {
            // target, .git, node_modules, build などの大きなディレクトリはスキップ
            let file_name = e.file_name().to_string_lossy();
            !matches!(
                file_name.as_ref(),
                "target" | ".git" | "node_modules" | ".cache" | "build" | ".dart_tool"
            )
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() && entry.file_name() == "pubspec.yaml" {
            if let Some(project_root) = entry.path().parent() {
                let build_dir = project_root.join("build");
                let dart_tool_dir = project_root.join(".dart_tool");

                // build または .dart_tool が存在する場合のみ追加
                if build_dir.exists() || dart_tool_dir.exists() {
                    let build_size = if build_dir.exists() {
                        utils::calculate_dir_size(&build_dir)?
                    } else {
                        0
                    };

                    let dart_tool_size = if dart_tool_dir.exists() {
                        utils::calculate_dir_size(&dart_tool_dir)?
                    } else {
                        0
                    };

                    let total_size = build_size + dart_tool_size;

                    projects.push(FlutterProject {
                        root: project_root.to_path_buf(),
                        build_dir,
                        dart_tool_dir,
                        size: total_size,
                    });
                }
            }
        }
    }

    Ok(projects)
}

/// Flutter プロジェクトをクリーン
pub fn clean_project(project: &FlutterProject) -> Result<()> {
    if project.build_exists() {
        fs::remove_dir_all(&project.build_dir)?;
    }
    if project.dart_tool_exists() {
        fs::remove_dir_all(&project.dart_tool_dir)?;
    }
    Ok(())
}

/// 複数の Flutter プロジェクトをクリーン
pub fn clean_projects(projects: &[FlutterProject]) -> Result<Vec<PathBuf>> {
    let mut cleaned = Vec::new();

    for project in projects {
        clean_project(project)?;
        cleaned.push(project.root.clone());
    }

    Ok(cleaned)
}

/// Flutter プロジェクトクリーナー
pub struct FlutterCleaner {
    pub search_path: PathBuf,
}

impl FlutterCleaner {
    pub fn new(search_path: PathBuf) -> Self {
        Self { search_path }
    }
}

impl Cleanable for FlutterCleaner {
    fn scan(&self) -> Result<Vec<CleanableItem>> {
        let projects = find_flutter_projects(&self.search_path)?;

        Ok(projects
            .into_iter()
            .map(|p| CleanableItem::new(p.root.display().to_string(), p.root.clone(), p.size))
            .collect())
    }

    fn name(&self) -> &str {
        "Flutter"
    }

    fn icon(&self) -> &str {
        "🦋"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_flutter_projects() -> Result<()> {
        let temp = TempDir::new()?;
        let project_dir = temp.path().join("test-flutter-project");
        fs::create_dir(&project_dir)?;

        // pubspec.yaml を作成
        fs::write(
            project_dir.join("pubspec.yaml"),
            r#"name: test_flutter
description: A test Flutter project
version: 1.0.0"#,
        )?;

        // build ディレクトリを作成
        let build_dir = project_dir.join("build");
        fs::create_dir(&build_dir)?;
        fs::write(build_dir.join("test.txt"), "test data")?;

        // .dart_tool ディレクトリを作成
        let dart_tool_dir = project_dir.join(".dart_tool");
        fs::create_dir(&dart_tool_dir)?;
        fs::write(dart_tool_dir.join("cache.txt"), "cache data")?;

        // プロジェクトを検索
        let projects = find_flutter_projects(temp.path())?;

        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].root, project_dir);
        assert!(projects[0].size > 0);

        Ok(())
    }

    #[test]
    fn test_clean_project() -> Result<()> {
        let temp = TempDir::new()?;
        let project_dir = temp.path().join("test-flutter-project");
        fs::create_dir(&project_dir)?;

        let build_dir = project_dir.join("build");
        fs::create_dir(&build_dir)?;
        fs::write(build_dir.join("test.txt"), "test data")?;

        let dart_tool_dir = project_dir.join(".dart_tool");
        fs::create_dir(&dart_tool_dir)?;
        fs::write(dart_tool_dir.join("cache.txt"), "cache data")?;

        let project = FlutterProject {
            root: project_dir.clone(),
            build_dir: build_dir.clone(),
            dart_tool_dir: dart_tool_dir.clone(),
            size: 100,
        };

        assert!(build_dir.exists());
        assert!(dart_tool_dir.exists());

        clean_project(&project)?;

        assert!(!build_dir.exists());
        assert!(!dart_tool_dir.exists());

        Ok(())
    }
}
