use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::Result;

/// B2 CLI のラッパー
pub struct B2Client {
    key_id: String,
    key: String,
}

impl B2Client {
    pub fn new(key_id: String, key: String) -> Result<Self> {
        if key_id.is_empty() {
            return Err(crate::Error::Config(
                "B2 Application Key ID is empty".into(),
            ));
        }
        if key.is_empty() {
            return Err(crate::Error::Config(
                "B2 Application Key is empty".into(),
            ));
        }
        Ok(Self { key_id, key })
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
    /// B2 CLI v4+ では環境変数経由で認証情報を渡すことを推奨
    pub fn authorize(&self) -> Result<()> {
        let output = Command::new("b2")
            .env("B2_APPLICATION_KEY_ID", &self.key_id)
            .env("B2_APPLICATION_KEY", &self.key)
            .arg("account")
            .arg("authorize")
            .output()
            .map_err(|e| crate::Error::B2(format!("Failed to run b2 account authorize: {}", e)))?;

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
    /// 注意: 事前に authorize() を呼び出しておく必要があります
    pub fn upload_file(
        &self,
        bucket: &str,
        local_path: &Path,
        remote_path: &str,
    ) -> Result<String> {
        let output = Command::new("b2")
            .env("B2_APPLICATION_KEY_ID", &self.key_id)
            .env("B2_APPLICATION_KEY", &self.key)
            .arg("file")
            .arg("upload")
            .arg("--no-progress")
            .arg("--threads")
            .arg("1")
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
    /// 注意: 事前に authorize() を呼び出しておく必要があります
    pub fn download_file_by_name(
        &self,
        bucket: &str,
        remote_path: &str,
        local_path: &Path,
    ) -> Result<()> {
        // B2 URI 形式に変換
        let b2_uri = format!("b2://{}/{}", bucket, remote_path);

        let output = Command::new("b2")
            .env("B2_APPLICATION_KEY_ID", &self.key_id)
            .env("B2_APPLICATION_KEY", &self.key)
            .arg("file")
            .arg("download")
            .arg("--no-progress")
            .arg(&b2_uri)
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

            let remote_path = PathBuf::from(remote_prefix).join(relative_path);
            let remote_path_str = remote_path.to_string_lossy();

            let file_id = self.upload_file(bucket, local_path, &remote_path_str)?;
            uploaded.push(file_id);
        }

        Ok(uploaded)
    }

    /// B2 上のファイル一覧を取得
    /// 注意: 事前に authorize() を呼び出しておく必要があります
    pub fn list_files(&self, bucket: &str, prefix: &str) -> Result<Vec<String>> {
        let output = Command::new("b2")
            .env("B2_APPLICATION_KEY_ID", &self.key_id)
            .env("B2_APPLICATION_KEY", &self.key)
            .arg("file")
            .arg("ls")
            .arg("--recursive")
            .arg(bucket)
            .arg(prefix)
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
                // B2 の ls 出力形式: "filename  size  upload_time"
                // ファイル名部分だけを抽出
                line.split_whitespace().next().unwrap_or("").to_string()
            })
            .filter(|s| !s.is_empty())
            .collect();

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pathbuf_join_for_b2() {
        // PathBufでパスを結合してから文字列化
        let base = PathBuf::from("files/20251114_130523");
        let relative = PathBuf::from("training/lora_output");
        let joined = base.join(relative);
        let result = joined.to_string_lossy();

        assert_eq!(result, "files/20251114_130523/training/lora_output");

        // 空のrelativeパス（PathBuf::join("")は末尾に/を追加する）
        let base = PathBuf::from("files/20251114_130523");
        let relative = PathBuf::from("");
        let joined = base.join(relative);
        let result = joined.to_string_lossy();

        // 空文字列をjoinすると末尾に/が付く（PathBufの仕様）
        assert_eq!(result, "files/20251114_130523/");

        // ネストされたパス
        let base = PathBuf::from("base");
        let relative = PathBuf::from("sub/dir/file.txt");
        let joined = base.join(relative);
        let result = joined.to_string_lossy();

        assert_eq!(result, "base/sub/dir/file.txt");
    }

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
