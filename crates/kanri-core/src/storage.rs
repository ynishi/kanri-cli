use std::path::Path;

use crate::Result;

/// クラウドストレージクライアントの共通インターフェース
pub trait StorageClient {
    /// 認証を行う
    fn authorize(&self) -> Result<()>;

    /// ファイルをアップロード
    fn upload_file(&self, bucket: &str, local_path: &Path, remote_path: &str) -> Result<String>;

    /// ディレクトリを再帰的にアップロード
    fn upload_directory(
        &self,
        bucket: &str,
        local_dir: &Path,
        remote_prefix: &str,
    ) -> Result<Vec<String>>;

    /// ファイルをダウンロード
    fn download_file_by_name(
        &self,
        bucket: &str,
        remote_path: &str,
        local_path: &Path,
    ) -> Result<()>;

    /// ファイル一覧を取得
    fn list_files(&self, bucket: &str, prefix: &str) -> Result<Vec<String>>;
}
