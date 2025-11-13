use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Result;

/// アーカイブメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveIndex {
    pub archives: Vec<Archive>,
}

/// アーカイブ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Archive {
    /// アーカイブ ID
    pub id: String,
    /// 作成日時
    pub created_at: DateTime<Utc>,
    /// クリーナー名
    pub cleaner: String,
    /// アーカイブ先（B2 パス）
    pub destination: String,
    /// アーカイブアイテム
    pub items: Vec<ArchiveItem>,
    /// 合計サイズ
    pub total_size: u64,
}

/// アーカイブアイテム
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveItem {
    /// ローカルパス
    pub local_path: PathBuf,
    /// B2 パス
    pub b2_path: String,
    /// SHA256 ハッシュ
    pub sha256: String,
    /// サイズ
    pub size: u64,
    /// ディレクトリかどうか
    pub is_dir: bool,
}

impl ArchiveIndex {
    /// アーカイブインデックスのパスを取得
    pub fn index_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| crate::Error::Config("HOME environment variable not set".into()))?;
        Ok(PathBuf::from(home)
            .join(".kanri")
            .join("archive_index.json"))
    }

    /// アーカイブインデックスを読み込み
    pub fn load() -> Result<Self> {
        let path = Self::index_path()?;

        if !path.exists() {
            return Ok(ArchiveIndex {
                archives: Vec::new(),
            });
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            crate::Error::Config(format!("Failed to read archive index: {}", e))
        })?;

        let index: ArchiveIndex = serde_json::from_str(&content).map_err(|e| {
            crate::Error::Config(format!("Failed to parse archive index: {}", e))
        })?;

        Ok(index)
    }

    /// アーカイブインデックスを保存
    pub fn save(&self) -> Result<()> {
        let path = Self::index_path()?;

        // ディレクトリを作成
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                crate::Error::Config(format!("Failed to create archive directory: {}", e))
            })?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            crate::Error::Config(format!("Failed to serialize archive index: {}", e))
        })?;

        fs::write(&path, content).map_err(|e| {
            crate::Error::Config(format!("Failed to write archive index: {}", e))
        })?;

        Ok(())
    }

    /// 新しいアーカイブを追加
    pub fn add_archive(&mut self, archive: Archive) {
        self.archives.push(archive);
    }

    /// ID でアーカイブを検索
    pub fn find_by_id(&self, id: &str) -> Option<&Archive> {
        self.archives.iter().find(|a| a.id == id)
    }

    /// アーカイブを削除
    pub fn remove_archive(&mut self, id: &str) -> bool {
        if let Some(pos) = self.archives.iter().position(|a| a.id == id) {
            self.archives.remove(pos);
            true
        } else {
            false
        }
    }
}

impl Archive {
    /// 新しいアーカイブを作成
    pub fn new(cleaner: String, destination: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            cleaner,
            destination,
            items: Vec::new(),
            total_size: 0,
        }
    }

    /// アイテムを追加
    pub fn add_item(&mut self, item: ArchiveItem) {
        self.total_size += item.size;
        self.items.push(item);
    }
}

impl ArchiveItem {
    /// 新しいアーカイブアイテムを作成
    pub fn new(
        local_path: PathBuf,
        b2_path: String,
        sha256: String,
        size: u64,
        is_dir: bool,
    ) -> Self {
        Self {
            local_path,
            b2_path,
            sha256,
            size,
            is_dir,
        }
    }

    /// ファイルから ArchiveItem を作成
    pub fn from_file(local_path: &Path, b2_path: String) -> Result<Self> {
        let metadata = fs::metadata(local_path).map_err(|e| {
            crate::Error::Archive(format!("Failed to get file metadata: {}", e))
        })?;

        let size = metadata.len();
        let is_dir = metadata.is_dir();

        // ディレクトリの場合は SHA256 は空
        let sha256 = if is_dir {
            String::new()
        } else {
            crate::b2::B2Client::calculate_sha256(local_path)?
        };

        Ok(Self::new(
            local_path.to_path_buf(),
            b2_path,
            sha256,
            size,
            is_dir,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archive_creation() {
        let mut archive = Archive::new("test-cleaner".to_string(), "b2://bucket/path".to_string());
        assert!(!archive.id.is_empty());
        assert_eq!(archive.cleaner, "test-cleaner");
        assert_eq!(archive.items.len(), 0);

        let item = ArchiveItem::new(
            PathBuf::from("/tmp/test"),
            "path/to/file".to_string(),
            "abc123".to_string(),
            1024,
            false,
        );

        archive.add_item(item);
        assert_eq!(archive.items.len(), 1);
        assert_eq!(archive.total_size, 1024);
    }

    #[test]
    fn test_archive_index() {
        let mut index = ArchiveIndex {
            archives: Vec::new(),
        };

        let archive = Archive::new("test".to_string(), "b2://test".to_string());
        let archive_id = archive.id.clone();

        index.add_archive(archive);
        assert_eq!(index.archives.len(), 1);

        let found = index.find_by_id(&archive_id);
        assert!(found.is_some());

        let removed = index.remove_archive(&archive_id);
        assert!(removed);
        assert_eq!(index.archives.len(), 0);
    }
}
