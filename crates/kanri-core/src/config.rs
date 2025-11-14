use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Result;

/// Kanri 設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub b2: Option<B2Config>,
    pub storage: Option<StorageConfig>,
}

/// B2 設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct B2Config {
    /// デフォルトのバケット名
    pub bucket: String,
    /// Application Key ID（オプション、環境変数優先）
    pub application_key_id: Option<String>,
    /// Application Key（オプション、環境変数優先）
    pub application_key: Option<String>,
}

/// Storage 設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// ストレージバックエンド ("b2" or "rclone")
    #[serde(default = "default_backend")]
    pub backend: String,
    /// Rclone リモート名（backend="rclone"の場合に使用）
    pub rclone_remote: Option<String>,
}

fn default_backend() -> String {
    "b2".to_string()
}

impl Config {
    /// 設定ファイルのパスを取得
    pub fn config_path() -> Result<PathBuf> {
        let home = env::var("HOME")
            .map_err(|_| crate::Error::Config("HOME environment variable not set".into()))?;
        Ok(PathBuf::from(home).join(".kanri").join("config.toml"))
    }

    /// 設定を読み込み
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            crate::Error::Config(format!("Failed to read config file: {}", e))
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            crate::Error::Config(format!("Failed to parse config file: {}", e))
        })?;

        Ok(config)
    }

    /// 設定を保存
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // ディレクトリを作成
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                crate::Error::Config(format!("Failed to create config directory: {}", e))
            })?;
        }

        let content = toml::to_string_pretty(self).map_err(|e| {
            crate::Error::Config(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(&path, content).map_err(|e| {
            crate::Error::Config(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    /// B2 認証情報を取得（環境変数優先）
    pub fn get_b2_credentials(&self) -> Result<(String, String)> {
        // 環境変数を優先
        let key_id = env::var("B2_APPLICATION_KEY_ID")
            .or_else(|_| {
                self.b2
                    .as_ref()
                    .and_then(|b2| b2.application_key_id.clone())
                    .ok_or_else(|| env::VarError::NotPresent)
            })
            .map_err(|_| {
                crate::Error::Config(
                    "B2_APPLICATION_KEY_ID not found in environment or config".into(),
                )
            })?;

        let key = env::var("B2_APPLICATION_KEY")
            .or_else(|_| {
                self.b2
                    .as_ref()
                    .and_then(|b2| b2.application_key.clone())
                    .ok_or_else(|| env::VarError::NotPresent)
            })
            .map_err(|_| {
                crate::Error::Config("B2_APPLICATION_KEY not found in environment or config".into())
            })?;

        Ok((key_id, key))
    }

    /// B2 バケット名を取得
    pub fn get_b2_bucket(&self) -> Result<String> {
        self.b2
            .as_ref()
            .map(|b2| b2.bucket.clone())
            .ok_or_else(|| crate::Error::Config("B2 bucket not configured".into()))
    }

    /// StorageClient を作成
    pub fn create_storage_client(&self) -> Result<Box<dyn crate::StorageClient>> {
        let backend = self
            .storage
            .as_ref()
            .map(|s| s.backend.as_str())
            .unwrap_or("b2");

        match backend {
            "b2" => {
                let (key_id, key) = self.get_b2_credentials()?;
                let client = crate::b2::B2Client::new(key_id, key)?;
                Ok(Box::new(client))
            }
            "rclone" => {
                let remote = self
                    .storage
                    .as_ref()
                    .and_then(|s| s.rclone_remote.clone())
                    .ok_or_else(|| {
                        crate::Error::Config("Rclone remote not configured".into())
                    })?;
                let client = crate::rclone::RcloneClient::new(remote)?;
                Ok(Box::new(client))
            }
            _ => Err(crate::Error::Config(format!(
                "Unknown storage backend: {}",
                backend
            ))),
        }
    }

    /// ストレージバックエンド名を取得
    pub fn get_storage_backend(&self) -> String {
        self.storage
            .as_ref()
            .map(|s| s.backend.clone())
            .unwrap_or_else(|| "b2".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = Config {
            b2: Some(B2Config {
                bucket: "my-bucket".to_string(),
                application_key_id: Some("key-id".to_string()),
                application_key: Some("key".to_string()),
            }),
            storage: None,
        };

        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("my-bucket"));
        assert!(toml.contains("key-id"));

        let parsed: Config = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.b2.unwrap().bucket, "my-bucket");
    }

    #[test]
    fn test_storage_config_serialization() {
        let config = Config {
            b2: Some(B2Config {
                bucket: "my-bucket".to_string(),
                application_key_id: None,
                application_key: None,
            }),
            storage: Some(StorageConfig {
                backend: "rclone".to_string(),
                rclone_remote: Some("b2:my-bucket".to_string()),
            }),
        };

        let toml = toml::to_string(&config).unwrap();
        assert!(toml.contains("rclone"));
        assert!(toml.contains("b2:my-bucket"));

        let parsed: Config = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.storage.unwrap().backend, "rclone");
    }

    #[test]
    fn test_get_storage_backend_default() {
        let config = Config {
            b2: None,
            storage: None,
        };

        assert_eq!(config.get_storage_backend(), "b2");
    }

    #[test]
    fn test_get_storage_backend_rclone() {
        let config = Config {
            b2: None,
            storage: Some(StorageConfig {
                backend: "rclone".to_string(),
                rclone_remote: Some("b2:bucket".to_string()),
            }),
        };

        assert_eq!(config.get_storage_backend(), "rclone");
    }
}
