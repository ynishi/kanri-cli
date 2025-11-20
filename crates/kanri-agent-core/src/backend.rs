use crate::{Expertise, Result, Scope};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Backend trait: Expertise の保存・取得を抽象化
pub trait Backend: Send + Sync {
    /// 指定スコープの全 Expertise を取得
    fn pull(&self, scope: Scope) -> Result<Vec<Expertise>>;

    /// Expertise を保存（現在のバージョンを versions/ に保存してから更新）
    fn push(&self, expertise: &Expertise) -> Result<()>;

    /// 指定スコープの Expertise ID 一覧を取得
    fn list(&self, scope: Scope) -> Result<Vec<String>>;

    /// Expertise を削除
    fn delete(&self, scope: Scope, id: &str) -> Result<()>;

    /// 特定の Expertise を取得
    fn get(&self, scope: Scope, id: &str) -> Result<Option<Expertise>>;

    /// 特定バージョンの Expertise を取得
    fn get_version(&self, scope: Scope, id: &str, version: &str) -> Result<Option<Expertise>>;

    /// バージョン一覧を取得（新しい順）
    fn list_versions(&self, scope: Scope, id: &str) -> Result<Vec<String>>;
}

/// LocalBackend: ローカルファイルシステムバックエンド
pub struct LocalBackend {
    base_path: PathBuf,
}

impl LocalBackend {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// デフォルトパス（~/.kanri-agent/cache/）
    pub fn default() -> Result<Self> {
        let home =
            std::env::var("HOME").map_err(|_| crate::Error::Config("HOME not set".to_string()))?;
        let base_path = PathBuf::from(home).join(".kanri-agent").join("cache");
        Ok(Self { base_path })
    }

    /// ベースディレクトリを取得
    pub fn target_dir(&self) -> &Path {
        &self.base_path
    }

    /// スコープのディレクトリパスを取得
    fn scope_dir(&self, scope: Scope) -> PathBuf {
        self.base_path.join(scope.as_str())
    }

    /// Expertise のファイルパスを取得
    fn expertise_path(&self, scope: Scope, id: &str) -> PathBuf {
        self.scope_dir(scope).join(id).join("expertise.yaml")
    }

    /// バージョンディレクトリのパスを取得
    fn versions_dir(&self, scope: Scope, id: &str) -> PathBuf {
        self.scope_dir(scope).join(id).join("versions")
    }

    /// 特定バージョンのファイルパスを取得
    fn version_path(&self, scope: Scope, id: &str, version: &str) -> PathBuf {
        self.versions_dir(scope, id)
            .join(format!("{}.yaml", version))
    }
}

impl Backend for LocalBackend {
    fn pull(&self, scope: Scope) -> Result<Vec<Expertise>> {
        let scope_dir = self.scope_dir(scope);

        if !scope_dir.exists() {
            return Ok(Vec::new());
        }

        let mut expertises = Vec::new();

        for entry in WalkDir::new(&scope_dir)
            .min_depth(1)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.file_name() == Some(std::ffi::OsStr::new("expertise.yaml")) {
                let content = fs::read_to_string(path)?;
                let expertise = Expertise::from_yaml(&content)?;
                expertises.push(expertise);
            }
        }

        Ok(expertises)
    }

    fn push(&self, expertise: &Expertise) -> Result<()> {
        let exp_path = self.expertise_path(expertise.metadata.scope, &expertise.id);

        // 既存のバージョンがあれば versions/ に保存
        if exp_path.exists() {
            let existing_content = fs::read_to_string(&exp_path)?;
            let existing = Expertise::from_yaml(&existing_content)?;

            // versions/ ディレクトリを作成
            let versions_dir = self.versions_dir(expertise.metadata.scope, &expertise.id);
            fs::create_dir_all(&versions_dir)?;

            // 既存バージョンを保存
            let version_path =
                self.version_path(expertise.metadata.scope, &expertise.id, &existing.version);
            fs::write(&version_path, existing_content)?;
        }

        // ディレクトリを作成
        if let Some(parent) = exp_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // YAML として保存
        let yaml = expertise.to_yaml()?;
        fs::write(&exp_path, yaml)?;

        Ok(())
    }

    fn list(&self, scope: Scope) -> Result<Vec<String>> {
        let scope_dir = self.scope_dir(scope);

        if !scope_dir.exists() {
            return Ok(Vec::new());
        }

        let mut ids = Vec::new();

        for entry in fs::read_dir(&scope_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    ids.push(name.to_string());
                }
            }
        }

        Ok(ids)
    }

    fn delete(&self, scope: Scope, id: &str) -> Result<()> {
        let exp_dir = self.scope_dir(scope).join(id);

        if exp_dir.exists() {
            fs::remove_dir_all(&exp_dir)?;
        }

        Ok(())
    }

    fn get(&self, scope: Scope, id: &str) -> Result<Option<Expertise>> {
        let exp_path = self.expertise_path(scope, id);

        if !exp_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&exp_path)?;
        let expertise = Expertise::from_yaml(&content)?;

        Ok(Some(expertise))
    }

    fn get_version(&self, scope: Scope, id: &str, version: &str) -> Result<Option<Expertise>> {
        let version_path = self.version_path(scope, id, version);

        if !version_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&version_path)?;
        let expertise = Expertise::from_yaml(&content)?;

        Ok(Some(expertise))
    }

    fn list_versions(&self, scope: Scope, id: &str) -> Result<Vec<String>> {
        let versions_dir = self.versions_dir(scope, id);

        if !versions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();

        for entry in fs::read_dir(&versions_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".yaml") {
                        let version = name.trim_end_matches(".yaml");
                        versions.push(version.to_string());
                    }
                }
            }
        }

        // バージョンでソート（降順）
        versions.sort_by(|a, b| {
            let va = crate::Version::parse(a).unwrap_or_default();
            let vb = crate::Version::parse(b).unwrap_or_default();
            vb.cmp(&va)
        });

        Ok(versions)
    }
}

/// GitBackend: ローカル Git リポジトリバックエンド
/// 変更は自動コミット、push は行わない（事故防止）
pub struct GitBackend {
    repo_path: PathBuf,
}

impl GitBackend {
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        // .git ディレクトリがあるか確認
        let git_dir = repo_path.join(".git");
        if !git_dir.exists() {
            // なければ git init
            fs::create_dir_all(&repo_path)?;
            Command::new("git")
                .arg("init")
                .current_dir(&repo_path)
                .output()
                .map_err(|e| crate::Error::Backend(format!("Failed to init git: {}", e)))?;
        }

        Ok(Self { repo_path })
    }

    fn scope_dir(&self, scope: Scope) -> PathBuf {
        self.repo_path.join(scope.as_str())
    }

    fn expertise_path(&self, scope: Scope, id: &str) -> PathBuf {
        self.scope_dir(scope).join(id).join("expertise.yaml")
    }

    fn versions_dir(&self, scope: Scope, id: &str) -> PathBuf {
        self.scope_dir(scope).join(id).join("versions")
    }

    fn version_path(&self, scope: Scope, id: &str, version: &str) -> PathBuf {
        self.versions_dir(scope, id)
            .join(format!("{}.yaml", version))
    }

    /// Git commit を実行（自動コミット、push はしない）
    fn commit(&self, message: &str) -> Result<()> {
        // git add -A
        Command::new("git")
            .arg("add")
            .arg("-A")
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| crate::Error::Backend(format!("Failed to git add: {}", e)))?;

        // git commit
        let output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| crate::Error::Backend(format!("Failed to git commit: {}", e)))?;

        // 変更がない場合（nothing to commit）はエラーにしない
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("nothing to commit") {
                return Err(crate::Error::Backend(format!(
                    "Git commit failed: {}",
                    stderr
                )));
            }
        }

        Ok(())
    }
}

impl Backend for GitBackend {
    fn pull(&self, scope: Scope) -> Result<Vec<Expertise>> {
        let scope_dir = self.scope_dir(scope);

        if !scope_dir.exists() {
            return Ok(Vec::new());
        }

        let mut expertises = Vec::new();

        for entry in WalkDir::new(&scope_dir)
            .min_depth(1)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.file_name() == Some(std::ffi::OsStr::new("expertise.yaml")) {
                let content = fs::read_to_string(path)?;
                let expertise = Expertise::from_yaml(&content)?;
                expertises.push(expertise);
            }
        }

        Ok(expertises)
    }

    fn push(&self, expertise: &Expertise) -> Result<()> {
        let exp_path = self.expertise_path(expertise.metadata.scope, &expertise.id);

        // 既存のバージョンがあれば versions/ に保存
        if exp_path.exists() {
            let existing_content = fs::read_to_string(&exp_path)?;
            let existing = Expertise::from_yaml(&existing_content)?;

            // versions/ ディレクトリを作成
            let versions_dir = self.versions_dir(expertise.metadata.scope, &expertise.id);
            fs::create_dir_all(&versions_dir)?;

            // 既存バージョンを保存
            let version_path =
                self.version_path(expertise.metadata.scope, &expertise.id, &existing.version);
            fs::write(&version_path, existing_content)?;
        }

        // ディレクトリを作成
        if let Some(parent) = exp_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // YAML として保存
        let yaml = expertise.to_yaml()?;
        fs::write(&exp_path, yaml)?;

        // Git にコミット
        let commit_msg = format!(
            "Add/Update expertise: {}:{} v{}",
            expertise.metadata.scope.as_str(),
            expertise.id,
            expertise.version
        );
        self.commit(&commit_msg)?;

        Ok(())
    }

    fn list(&self, scope: Scope) -> Result<Vec<String>> {
        let scope_dir = self.scope_dir(scope);

        if !scope_dir.exists() {
            return Ok(Vec::new());
        }

        let mut ids = Vec::new();

        for entry in fs::read_dir(&scope_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    ids.push(name.to_string());
                }
            }
        }

        Ok(ids)
    }

    fn delete(&self, scope: Scope, id: &str) -> Result<()> {
        let exp_dir = self.scope_dir(scope).join(id);

        if exp_dir.exists() {
            fs::remove_dir_all(&exp_dir)?;

            // Git にコミット
            let commit_msg = format!("Delete expertise: {}:{}", scope.as_str(), id);
            self.commit(&commit_msg)?;
        }

        Ok(())
    }

    fn get(&self, scope: Scope, id: &str) -> Result<Option<Expertise>> {
        let exp_path = self.expertise_path(scope, id);

        if !exp_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&exp_path)?;
        let expertise = Expertise::from_yaml(&content)?;

        Ok(Some(expertise))
    }

    fn get_version(&self, scope: Scope, id: &str, version: &str) -> Result<Option<Expertise>> {
        let version_path = self.version_path(scope, id, version);

        if !version_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&version_path)?;
        let expertise = Expertise::from_yaml(&content)?;

        Ok(Some(expertise))
    }

    fn list_versions(&self, scope: Scope, id: &str) -> Result<Vec<String>> {
        let versions_dir = self.versions_dir(scope, id);

        if !versions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut versions = Vec::new();

        for entry in fs::read_dir(&versions_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".yaml") {
                        let version = name.trim_end_matches(".yaml");
                        versions.push(version.to_string());
                    }
                }
            }
        }

        // バージョンでソート（降順）
        versions.sort_by(|a, b| {
            let va = crate::Version::parse(a).unwrap_or_default();
            let vb = crate::Version::parse(b).unwrap_or_default();
            vb.cmp(&va)
        });

        Ok(versions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_local_backend_push_and_pull() {
        let temp = TempDir::new().unwrap();
        let backend = LocalBackend::new(temp.path().to_path_buf());

        let mut exp = Expertise::new(
            "test-skill".to_string(),
            "Test Skill".to_string(),
            Scope::Personal,
        );
        exp.body.synthesis_logic = "Test content".to_string();

        // Push
        backend.push(&exp).unwrap();

        // Pull
        let pulled = backend.pull(Scope::Personal).unwrap();
        assert_eq!(pulled.len(), 1);
        assert_eq!(pulled[0].id, "test-skill");
        assert_eq!(pulled[0].body.synthesis_logic, "Test content");
    }

    #[test]
    fn test_local_backend_list() {
        let temp = TempDir::new().unwrap();
        let backend = LocalBackend::new(temp.path().to_path_buf());

        let exp1 = Expertise::new("skill1".to_string(), "Skill 1".to_string(), Scope::Personal);
        let exp2 = Expertise::new("skill2".to_string(), "Skill 2".to_string(), Scope::Personal);

        backend.push(&exp1).unwrap();
        backend.push(&exp2).unwrap();

        let ids = backend.list(Scope::Personal).unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"skill1".to_string()));
        assert!(ids.contains(&"skill2".to_string()));
    }

    #[test]
    fn test_local_backend_get() {
        let temp = TempDir::new().unwrap();
        let backend = LocalBackend::new(temp.path().to_path_buf());

        let exp = Expertise::new(
            "my-skill".to_string(),
            "My Skill".to_string(),
            Scope::Company,
        );
        backend.push(&exp).unwrap();

        let retrieved = backend.get(Scope::Company, "my-skill").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "my-skill");

        let not_found = backend.get(Scope::Company, "non-existent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_local_backend_delete() {
        let temp = TempDir::new().unwrap();
        let backend = LocalBackend::new(temp.path().to_path_buf());

        let exp = Expertise::new(
            "to-delete".to_string(),
            "Delete Me".to_string(),
            Scope::Project,
        );
        backend.push(&exp).unwrap();

        let before = backend.list(Scope::Project).unwrap();
        assert_eq!(before.len(), 1);

        backend.delete(Scope::Project, "to-delete").unwrap();

        let after = backend.list(Scope::Project).unwrap();
        assert_eq!(after.len(), 0);
    }
}
