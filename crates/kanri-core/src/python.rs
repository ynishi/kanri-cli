use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{
    cleanable::{Cleanable, CleanableItem},
    utils, Result,
};

/// Python 仮想環境情報
#[derive(Debug, Clone)]
pub struct PythonVenv {
    /// プロジェクトのルートディレクトリ
    pub root: PathBuf,
    /// 仮想環境ディレクトリのパス
    pub venv_dir: PathBuf,
    /// サイズ（バイト）
    pub size: u64,
}

/// 指定されたディレクトリ以下の Python 仮想環境を検索
pub fn find_python_venvs(search_path: &Path) -> Result<Vec<PythonVenv>> {
    let mut venvs = Vec::new();

    for entry in WalkDir::new(search_path)
        .into_iter()
        .filter_entry(|e| {
            let file_name = e.file_name().to_string_lossy();
            !matches!(
                file_name.as_ref(),
                "target" | ".git" | "node_modules" | ".cache"
            )
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy();

        // venv, .venv, env, .env ディレクトリを検出
        if entry.file_type().is_dir()
            && matches!(file_name.as_ref(), "venv" | ".venv" | "env" | ".env")
        {
            // Python 仮想環境か確認（pyvenv.cfg または bin/activate の存在）
            let pyvenv_cfg = path.join("pyvenv.cfg");
            let bin_activate = path.join("bin").join("activate");

            if pyvenv_cfg.exists() || bin_activate.exists() {
                if let Some(project_root) = path.parent() {
                    let size = utils::calculate_dir_size(path)?;

                    venvs.push(PythonVenv {
                        root: project_root.to_path_buf(),
                        venv_dir: path.to_path_buf(),
                        size,
                    });
                }
            }
        }
    }

    Ok(venvs)
}

/// Python 仮想環境を削除
pub fn clean_venv(venv: &PythonVenv) -> Result<()> {
    if venv.venv_dir.exists() {
        fs::remove_dir_all(&venv.venv_dir)?;
    }
    Ok(())
}

/// Python クリーナー
pub struct PythonCleaner {
    pub search_path: PathBuf,
}

impl PythonCleaner {
    pub fn new(search_path: PathBuf) -> Self {
        Self { search_path }
    }
}

impl Cleanable for PythonCleaner {
    fn scan(&self) -> Result<Vec<CleanableItem>> {
        let venvs = find_python_venvs(&self.search_path)?;

        Ok(venvs
            .into_iter()
            .map(|v| {
                CleanableItem::new(
                    v.root.display().to_string(),
                    v.venv_dir,
                    v.size,
                )
            })
            .collect())
    }

    fn name(&self) -> &str {
        "Python"
    }

    fn icon(&self) -> &str {
        "🐍"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_find_python_venvs() -> Result<()> {
        let temp = TempDir::new()?;
        let project_dir = temp.path().join("test-project");
        fs::create_dir(&project_dir)?;

        let venv_dir = project_dir.join("venv");
        fs::create_dir_all(&venv_dir)?;
        fs::write(venv_dir.join("pyvenv.cfg"), "test")?;

        let venvs = find_python_venvs(temp.path())?;

        assert_eq!(venvs.len(), 1);
        assert_eq!(venvs[0].root, project_dir);

        Ok(())
    }
}
