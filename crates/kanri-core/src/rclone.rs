use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::{Result, StorageClient};

/// Rclone CLI のラッパー
pub struct RcloneClient {
    remote: String,
}

impl RcloneClient {
    pub fn new(remote: String) -> Result<Self> {
        if remote.is_empty() {
            return Err(crate::Error::Config("Rclone remote is empty".into()));
        }
        Ok(Self { remote })
    }

    /// Rclone CLI がインストールされているか確認
    pub fn is_installed() -> bool {
        Command::new("rclone")
            .arg("version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// rclone のリモートパスを構築
    fn build_remote_path(&self, path: &str) -> String {
        format!("{}:{}", self.remote, path)
    }
}

impl StorageClient for RcloneClient {
    fn authorize(&self) -> Result<()> {
        // rclone は設定ファイルベースなので、ここでは接続テストを行う
        let output = Command::new("rclone")
            .arg("lsd")
            .arg(&self.remote)
            .arg("--max-depth")
            .arg("1")
            .output()
            .map_err(|e| crate::Error::B2(format!("Failed to run rclone lsd: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::Error::B2(format!(
                "Failed to access rclone remote: {}",
                stderr
            )));
        }

        Ok(())
    }

    fn upload_file(&self, _bucket: &str, local_path: &Path, remote_path: &str) -> Result<String> {
        let remote_full = self.build_remote_path(remote_path);

        let output = Command::new("rclone")
            .arg("copyto")
            .arg(local_path)
            .arg(&remote_full)
            .arg("--progress")
            .output()
            .map_err(|e| crate::Error::B2(format!("Failed to upload file: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::Error::B2(format!("Upload failed: {}", stderr)));
        }

        Ok(remote_full)
    }

    fn upload_directory(
        &self,
        _bucket: &str,
        local_dir: &Path,
        remote_prefix: &str,
    ) -> Result<Vec<String>> {
        let remote_full = self.build_remote_path(remote_prefix);

        let output = Command::new("rclone")
            .arg("copy")
            .arg(local_dir)
            .arg(&remote_full)
            .arg("--progress")
            .output()
            .map_err(|e| crate::Error::B2(format!("Failed to upload directory: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::Error::B2(format!("Upload failed: {}", stderr)));
        }

        // rclone copy は個別のファイルIDを返さないので、空のベクタを返す
        Ok(vec![])
    }

    fn download_file_by_name(
        &self,
        _bucket: &str,
        remote_path: &str,
        local_path: &Path,
    ) -> Result<()> {
        let remote_full = self.build_remote_path(remote_path);

        // 親ディレクトリを作成
        if let Some(parent) = local_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::Error::B2(format!("Failed to create parent directory: {}", e)))?;
        }

        let output = Command::new("rclone")
            .arg("copyto")
            .arg(&remote_full)
            .arg(local_path)
            .arg("--progress")
            .output()
            .map_err(|e| crate::Error::B2(format!("Failed to download file: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::Error::B2(format!("Download failed: {}", stderr)));
        }

        Ok(())
    }

    fn list_files(&self, _bucket: &str, prefix: &str) -> Result<Vec<String>> {
        let remote_full = self.build_remote_path(prefix);

        let output = Command::new("rclone")
            .arg("lsf")
            .arg(&remote_full)
            .arg("--recursive")
            .arg("--files-only")
            .output()
            .map_err(|e| crate::Error::B2(format!("Failed to list files: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::Error::B2(format!("List files failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<String> = stdout
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| {
                // prefix を付けて完全なパスを返す
                if prefix.is_empty() {
                    line.to_string()
                } else {
                    PathBuf::from(prefix).join(line).to_string_lossy().to_string()
                }
            })
            .collect();

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rclone_cli_check() {
        let installed = RcloneClient::is_installed();
        println!("Rclone CLI installed: {}", installed);
    }

    #[test]
    fn test_build_remote_path() -> Result<()> {
        let client = RcloneClient::new("b2:my-bucket".to_string())?;
        assert_eq!(
            client.build_remote_path("files/archive"),
            "b2:my-bucket:files/archive"
        );
        Ok(())
    }
}
