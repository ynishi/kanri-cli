use std::process::Command;

use crate::{Error, Result};

/// Docker システム情報
#[derive(Debug, Clone)]
pub struct DockerInfo {
    /// 削除可能なデータのサイズ情報
    pub reclaimable: String,
}

/// Docker がインストールされているかチェック
pub fn is_docker_installed() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Docker デーモンが起動しているかチェック
pub fn is_docker_running() -> bool {
    Command::new("docker")
        .arg("info")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Docker システムの情報を取得（削除可能なサイズなど）
pub fn get_system_info() -> Result<DockerInfo> {
    if !is_docker_installed() {
        return Err(Error::InvalidPath(
            "Docker がインストールされていません".to_string(),
        ));
    }

    if !is_docker_running() {
        return Err(Error::InvalidPath(
            "Docker デーモンが起動していません".to_string(),
        ));
    }

    let output = Command::new("docker")
        .arg("system")
        .arg("df")
        .output()?;

    if !output.status.success() {
        return Err(Error::InvalidPath(
            "Docker システム情報の取得に失敗しました".to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // "Reclaimable" を含む行から情報を抽出
    let reclaimable = stdout
        .lines()
        .find(|line| line.contains("Local Volumes"))
        .and_then(|line| {
            line.split_whitespace()
                .rev()
                .take(2)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join(" ")
                .into()
        })
        .unwrap_or_else(|| "不明".to_string());

    Ok(DockerInfo { reclaimable })
}

/// Docker システムをクリーンアップ（未使用データを削除）
///
/// `all`: true の場合、使用されていないイメージもすべて削除
/// `volumes`: true の場合、ボリュームも削除
pub fn clean_system(all: bool, volumes: bool) -> Result<String> {
    if !is_docker_installed() {
        return Err(Error::InvalidPath(
            "Docker がインストールされていません".to_string(),
        ));
    }

    if !is_docker_running() {
        return Err(Error::InvalidPath(
            "Docker デーモンが起動していません".to_string(),
        ));
    }

    let mut args = vec!["system", "prune", "--force"];

    if all {
        args.push("--all");
    }

    if volumes {
        args.push("--volumes");
    }

    let output = Command::new("docker").args(&args).output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::InvalidPath(format!(
            "Docker クリーンアップに失敗しました: {}",
            stderr
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_docker_installed() {
        // このテストは環境依存なので、インストール状態だけチェック
        let installed = is_docker_installed();
        // Docker がインストールされているかどうかは環境依存
        println!("Docker installed: {}", installed);
    }

    #[test]
    fn test_is_docker_running() {
        // このテストは環境依存なので、実行状態だけチェック
        if is_docker_installed() {
            let running = is_docker_running();
            println!("Docker running: {}", running);
        }
    }
}
