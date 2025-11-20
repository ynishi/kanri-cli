use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::Result;

/// kanri-agent 設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub version: String,

    #[serde(default)]
    pub scopes: Vec<ScopeConfig>,

    #[serde(default)]
    pub deployment: DeploymentConfig,
}

/// Scope 設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeConfig {
    pub name: String,
    pub backend: BackendConfig,
    pub description: Option<String>,
}

/// Backend 設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BackendConfig {
    Local {
        path: String,
        #[serde(default)]
        gitignore: bool,
    },
    Git {
        repo: String,
        branch: String,
        #[serde(default)]
        auto_pull: bool,
        #[serde(default)]
        auto_push: bool,
    },
    Rclone {
        remote: String,
        prefix: String,
    },
}

/// Deployment 設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeploymentConfig {
    #[serde(default)]
    pub targets: Vec<TargetConfig>,

    #[serde(default = "default_true")]
    pub auto_clean: bool,

    #[serde(default = "default_true")]
    pub backup_before_clean: bool,
}

fn default_true() -> bool {
    true
}

/// Target 設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetConfig {
    pub name: String,
    pub enabled: bool,
    pub path: String,
    pub format: String,
    pub prefix_style: PrefixStyle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrefixStyle {
    Filename,    // company:skill.md
    Frontmatter, // YAMLフロントマターに記載
    None,        // Prefix なし
}

impl Config {
    /// 設定ファイルのパスを取得
    pub fn config_path() -> Result<PathBuf> {
        let home =
            std::env::var("HOME").map_err(|_| crate::Error::Config("HOME not set".to_string()))?;
        Ok(PathBuf::from(home).join(".kanri-agent").join("config.yaml"))
    }

    /// 設定を読み込み
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&path)?;
        let config: Config = serde_yaml::from_str(&content)
            .map_err(|e| crate::Error::Config(format!("Failed to parse config: {}", e)))?;

        Ok(config)
    }

    /// 設定を保存
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // ディレクトリを作成
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(self)
            .map_err(|e| crate::Error::Config(format!("Failed to serialize config: {}", e)))?;

        fs::write(&path, content)?;

        Ok(())
    }

    /// デフォルト設定を生成
    pub fn create_default() -> Self {
        Config {
            version: "1.0".to_string(),
            scopes: vec![ScopeConfig {
                name: "personal".to_string(),
                backend: BackendConfig::Local {
                    path: "~/.kanri-agent/cache/personal".to_string(),
                    gitignore: false,
                },
                description: Some("個人用プロンプト・ワークフロー".to_string()),
            }],
            deployment: DeploymentConfig {
                targets: vec![TargetConfig {
                    name: "claude-code".to_string(),
                    enabled: true,
                    path: ".claude/commands/".to_string(),
                    format: "markdown".to_string(),
                    prefix_style: PrefixStyle::Filename,
                }],
                auto_clean: true,
                backup_before_clean: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = Config::create_default();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.scopes.len(), 1);
        assert_eq!(config.deployment.targets.len(), 1);
    }

    #[test]
    fn test_config_yaml_roundtrip() {
        let config = Config::create_default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: Config = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(parsed.version, config.version);
        assert_eq!(parsed.scopes.len(), config.scopes.len());
    }
}
