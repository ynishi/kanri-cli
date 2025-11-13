use std::path::Path;
use std::process::{Command, Stdio};

use crate::Result;

/// B2 CLI のラッパー
pub struct B2Client {
    key_id: String,
    key: String,
}

impl B2Client {
    pub fn new(key_id: String, key: String) -> Self {
        Self { key_id, key }
    }

    /// B2 CLI がインストールされているか確認
    pub fn is_installed() -> bool {
        Command::new("b2")
            .arg("version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// B2 にログイン（認証）
    pub fn authorize(&self) -> Result<()> {
        let output = Command::new("b2")
            .arg("authorize-account")
            .arg(&self.key_id)
            .arg(&self.key)
            .output()
            .map_err(|e| crate::Error::B2(format!("Failed to run b2 authorize-account: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::Error::B2(format!(
                "Failed to authorize B2 account: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// ファイルを B2 にアップロード
    pub fn upload_file(
        &self,
        bucket: &str,
        local_path: &Path,
        remote_path: &str,
    ) -> Result<String> {
        // まず認証
        self.authorize()?;

        let output = Command::new("b2")
            .arg("upload-file")
            .arg("--noProgress")
            .arg(bucket)
            .arg(local_path)
            .arg(remote_path)
            .output()
            .map_err(|e| crate::Error::B2(format!("Failed to upload file: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::Error::B2(format!("Upload failed: {}", stderr)));
        }

        // 出力から file ID を取得（JSON パース）
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.trim().to_string())
    }

    /// ファイルを B2 からダウンロード
    pub fn download_file_by_name(
        &self,
        bucket: &str,
        remote_path: &str,
        local_path: &Path,
    ) -> Result<()> {
        // まず認証
        self.authorize()?;

        let output = Command::new("b2")
            .arg("download-file-by-name")
            .arg("--noProgress")
            .arg(bucket)
            .arg(remote_path)
            .arg(local_path)
            .output()
            .map_err(|e| crate::Error::B2(format!("Failed to download file: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::Error::B2(format!("Download failed: {}", stderr)));
        }

        Ok(())
    }

    /// ファイルの SHA256 ハッシュを計算
    pub fn calculate_sha256(path: &Path) -> Result<String> {
        use sha2::{Digest, Sha256};
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(path)
            .map_err(|e| crate::Error::B2(format!("Failed to open file for hashing: {}", e)))?;

        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let n = file
                .read(&mut buffer)
                .map_err(|e| crate::Error::B2(format!("Failed to read file for hashing: {}", e)))?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// ディレクトリを再帰的にアップロード
    pub fn upload_directory(
        &self,
        bucket: &str,
        local_dir: &Path,
        remote_prefix: &str,
    ) -> Result<Vec<String>> {
        use walkdir::WalkDir;

        let mut uploaded = Vec::new();

        for entry in WalkDir::new(local_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let local_path = entry.path();
            let relative_path = local_path
                .strip_prefix(local_dir)
                .map_err(|e| crate::Error::B2(format!("Failed to get relative path: {}", e)))?;

            let remote_path = if remote_prefix.is_empty() {
                relative_path.to_string_lossy().to_string()
            } else {
                format!("{}/{}", remote_prefix, relative_path.to_string_lossy())
            };

            let file_id = self.upload_file(bucket, local_path, &remote_path)?;
            uploaded.push(file_id);
        }

        Ok(uploaded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b2_cli_check() {
        // B2 CLI がインストールされているかチェック（環境依存）
        let installed = B2Client::is_installed();
        // 結果だけ確認（エラーにはしない）
        println!("B2 CLI installed: {}", installed);
    }

    #[test]
    fn test_sha256_calculation() -> Result<()> {
        use std::fs::File;
        use std::io::Write;
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();
        drop(file);

        let hash = B2Client::calculate_sha256(&file_path)?;
        // "hello world" の SHA256
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );

        Ok(())
    }
}
