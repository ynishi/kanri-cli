use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use kanri_core::Cleanable;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "kanri")]
#[command(author, version, about = "Mac ローカル環境管理ツール", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, ValueEnum)]
enum RestoreMode {
    /// 最新版のみを復元（デフォルト）
    Latest,
    /// 特定バージョンを復元（--version と併用）
    Version,
    /// タイムスタンプ付きでそのまま復元
    Raw,
}

#[derive(Subcommand)]
enum Commands {
    /// クリーンアップコマンド
    Clean {
        #[command(subcommand)]
        target: CleanTarget,
    },

    /// ファイル・ディレクトリを B2 にアーカイブ
    Archive {
        #[command(subcommand)]
        target: ArchiveTarget,
    },

    /// B2 からアーカイブを復元
    Restore {
        /// B2 上のアーカイブパス（プレフィックス）
        #[arg(long)]
        from: String,

        /// 復元先ディレクトリ
        #[arg(long, default_value = ".")]
        to: String,

        /// 復元モード
        #[arg(long, value_enum, default_value = "latest")]
        mode: RestoreMode,

        /// 特定バージョンを指定（--mode version と併用）
        #[arg(long)]
        version: Option<String>,

        /// Dry-run モード
        #[arg(long)]
        dry_run: bool,
    },

    /// アーカイブ一覧を表示
    ListArchives,

    /// 設定を初期化
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// シェル補完スクリプトを生成
    Completions {
        /// シェルの種類
        #[arg(value_enum)]
        shell: Shell,
    },

    /// システム全体の診断を実行（削除可能な項目をサマリー表示）
    Diagnose {
        /// JSON形式で出力
        #[arg(long)]
        json: bool,

        /// 最小サイズ閾値（GB）
        #[arg(long)]
        threshold: Option<f64>,

        /// 検索開始ディレクトリ（デフォルト: カレントディレクトリ）
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum CleanTarget {
    /// Rust プロジェクトの target ディレクトリをクリーン
    Rust {
        /// 検索開始ディレクトリ（デフォルト: カレントディレクトリ）
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// 検索・表示のみ（デフォルト動作）
        #[arg(short, long)]
        search: bool,

        /// 削除を実行
        #[arg(short, long)]
        delete: bool,

        /// インタラクティブモード（削除前に確認）
        #[arg(short, long)]
        interactive: bool,
    },

    /// Node.js プロジェクトの node_modules ディレクトリをクリーン
    Node {
        /// 検索開始ディレクトリ（デフォルト: カレントディレクトリ）
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// 検索・表示のみ（デフォルト動作）
        #[arg(short, long)]
        search: bool,

        /// 削除を実行
        #[arg(short, long)]
        delete: bool,

        /// インタラクティブモード（削除前に確認）
        #[arg(short, long)]
        interactive: bool,
    },

    /// Docker の未使用データをクリーン
    Docker {
        /// 検索・表示のみ（デフォルト動作）
        #[arg(short, long)]
        search: bool,

        /// 削除を実行
        #[arg(short, long)]
        delete: bool,

        /// インタラクティブモード（削除前に確認）
        #[arg(short, long)]
        interactive: bool,

        /// 使用されていないイメージもすべて削除
        #[arg(short, long)]
        all: bool,

        /// ボリュームも削除
        #[arg(short, long)]
        volumes: bool,
    },

    /// Flutter プロジェクトの build/.dart_tool をクリーン
    Flutter {
        /// 検索開始ディレクトリ（デフォルト: カレントディレクトリ）
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// 検索・表示のみ（デフォルト動作）
        #[arg(short, long)]
        search: bool,

        /// 削除を実行
        #[arg(short, long)]
        delete: bool,

        /// インタラクティブモード（削除前に確認）
        #[arg(short, long)]
        interactive: bool,
    },

    /// Mac アプリケーションキャッシュをクリーン (⚠️ Experimental)
    Cache {
        /// 検索・表示のみ（デフォルト動作）
        #[arg(short, long)]
        search: bool,

        /// 削除を実行
        #[arg(short, long)]
        delete: bool,

        /// インタラクティブモード（削除前に確認）
        #[arg(short, long)]
        interactive: bool,

        /// 最小サイズ（GB単位、デフォルト: 1GB）
        #[arg(long, default_value = "1")]
        min_size: u64,

        /// 安全なキャッシュのみ表示
        #[arg(long)]
        safe_only: bool,
    },

    /// Python 仮想環境をクリーン
    Python {
        /// 検索開始ディレクトリ（デフォルト: カレントディレクトリ）
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// 検索・表示のみ（デフォルト動作）
        #[arg(short, long)]
        search: bool,

        /// 削除を実行
        #[arg(short, long)]
        delete: bool,

        /// インタラクティブモード（削除前に確認）
        #[arg(short, long)]
        interactive: bool,
    },

    /// Go モジュールキャッシュをクリーン
    Go {
        /// 検索・表示のみ（デフォルト動作）
        #[arg(short, long)]
        search: bool,

        /// 削除を実行
        #[arg(short, long)]
        delete: bool,

        /// インタラクティブモード（削除前に確認）
        #[arg(short, long)]
        interactive: bool,
    },

    /// Gradle キャッシュをクリーン
    Gradle {
        /// 検索・表示のみ（デフォルト動作）
        #[arg(short, long)]
        search: bool,

        /// 削除を実行
        #[arg(short, long)]
        delete: bool,

        /// インタラクティブモード（削除前に確認）
        #[arg(short, long)]
        interactive: bool,
    },

    /// Haskell ビルド成果物をクリーン
    Haskell {
        /// 検索開始ディレクトリ（デフォルト: カレントディレクトリ）
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// 検索・表示のみ（デフォルト動作）
        #[arg(short, long)]
        search: bool,

        /// 削除を実行
        #[arg(short, long)]
        delete: bool,

        /// インタラクティブモード（削除前に確認）
        #[arg(short, long)]
        interactive: bool,
    },

    /// Xcode DerivedData をクリーン
    Xcode {
        /// 検索・表示のみ（デフォルト動作）
        #[arg(short, long)]
        search: bool,

        /// 削除を実行
        #[arg(short, long)]
        delete: bool,

        /// インタラクティブモード（削除前に確認）
        #[arg(short, long)]
        interactive: bool,
    },

    /// 大きなファイル・ディレクトリをクリーン
    LargeFiles {
        /// 検索開始ディレクトリ（デフォルト: カレントディレクトリ）
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// 最小サイズ（GB）（デフォルト: 2GB）
        #[arg(long, default_value = "2")]
        min_size_gb: u64,

        /// 拡張子フィルタ（カンマ区切り、例: .ckpt,.pth,.safetensors）
        #[arg(long)]
        extensions: Option<String>,

        /// ファイルのみを検索（デフォルト: ディレクトリとファイル両方）
        #[arg(long)]
        files_only: bool,

        /// ディレクトリのみを検索（デフォルト: ディレクトリとファイル両方）
        #[arg(long)]
        dirs_only: bool,

        /// 検索・表示のみ（デフォルト動作）
        #[arg(short, long)]
        search: bool,

        /// 削除を実行
        #[arg(short, long)]
        delete: bool,

        /// インタラクティブモード（削除前に確認）
        #[arg(short, long)]
        interactive: bool,
    },
}

#[derive(Subcommand)]
enum ArchiveTarget {
    /// 大きなファイルをアーカイブ
    LargeFiles {
        /// 検索開始ディレクトリ（デフォルト: カレントディレクトリ）
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// 最小サイズ（GB）
        #[arg(long, default_value = "2")]
        min_size_gb: u64,

        /// 拡張子フィルタ（カンマ区切り）
        #[arg(long)]
        extensions: Option<String>,

        /// ファイルのみ
        #[arg(long)]
        files_only: bool,

        /// ディレクトリのみ
        #[arg(long)]
        dirs_only: bool,

        /// アーカイブ先パス（B2 バケット内）
        #[arg(long)]
        to: String,

        /// アップロード成功後にローカルファイルを削除
        #[arg(long)]
        delete_after: bool,

        /// Dry-run モード
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// 設定を表示
    Show,

    /// B2 設定を初期化
    InitB2 {
        /// B2 バケット名
        #[arg(long)]
        bucket: String,

        /// Application Key ID（オプション、環境変数推奨）
        #[arg(long)]
        key_id: Option<String>,

        /// Application Key（オプション、環境変数推奨）
        #[arg(long)]
        key: Option<String>,
    },

    /// B2 認証をテスト
    TestB2,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Clean { target } => match target {
            CleanTarget::Rust {
                path,
                search,
                delete,
                interactive,
            } => clean_rust(&path, search, delete, interactive)?,
            CleanTarget::Node {
                path,
                search,
                delete,
                interactive,
            } => clean_node(&path, search, delete, interactive)?,
            CleanTarget::Docker {
                search,
                delete,
                interactive,
                all,
                volumes,
            } => clean_docker(search, delete, interactive, all, volumes)?,
            CleanTarget::Flutter {
                path,
                search,
                delete,
                interactive,
            } => clean_flutter(&path, search, delete, interactive)?,
            CleanTarget::Cache {
                search,
                delete,
                interactive,
                min_size,
                safe_only,
            } => clean_cache(search, delete, interactive, min_size, safe_only)?,
            CleanTarget::Python {
                path,
                search,
                delete,
                interactive,
            } => {
                let cleaner = kanri_core::python::PythonCleaner::new(path);
                clean_generic(&cleaner, "package.json", search, delete, interactive)?
            }
            CleanTarget::Go {
                search,
                delete,
                interactive,
            } => {
                let cleaner = kanri_core::go::GoCleaner::new();
                clean_generic(&cleaner, "Go module cache", search, delete, interactive)?
            }
            CleanTarget::Gradle {
                search,
                delete,
                interactive,
            } => {
                let cleaner = kanri_core::gradle::GradleCleaner::new();
                clean_generic(&cleaner, "Gradle cache", search, delete, interactive)?
            }
            CleanTarget::Haskell {
                path,
                search,
                delete,
                interactive,
            } => {
                let cleaner = kanri_core::haskell::HaskellCleaner::new(path);
                clean_generic(
                    &cleaner,
                    "*.cabal or stack.yaml",
                    search,
                    delete,
                    interactive,
                )?
            }
            CleanTarget::Xcode {
                search,
                delete,
                interactive,
            } => {
                let cleaner = kanri_core::xcode::XcodeCleaner::new();
                clean_generic(&cleaner, "DerivedData", search, delete, interactive)?
            }
            CleanTarget::LargeFiles {
                path,
                min_size_gb,
                extensions,
                files_only,
                dirs_only,
                search,
                delete,
                interactive,
            } => {
                let min_size = min_size_gb * 1024 * 1024 * 1024; // GB to bytes
                let ext_vec = extensions.map(|s| {
                    s.split(',')
                        .map(|e| e.trim().to_string())
                        .collect::<Vec<_>>()
                });

                // files_only と dirs_only が両方指定された場合はエラー
                let (include_files, include_dirs) = match (files_only, dirs_only) {
                    (true, true) => {
                        eprintln!("Error: --files-only and --dirs-only cannot be used together");
                        std::process::exit(1);
                    }
                    (true, false) => (true, false),
                    (false, true) => (false, true),
                    (false, false) => (true, true),
                };

                let mut cleaner = kanri_core::large_files::LargeFilesCleaner::new(path, min_size);
                if let Some(exts) = ext_vec {
                    cleaner = cleaner.with_extensions(exts);
                }
                cleaner = cleaner.with_include_dirs(include_dirs);
                cleaner = cleaner.with_include_files(include_files);

                clean_generic(&cleaner, "large items", search, delete, interactive)?
            }
        },
        Commands::Archive { target } => match target {
            ArchiveTarget::LargeFiles {
                path,
                min_size_gb,
                extensions,
                files_only,
                dirs_only,
                to,
                delete_after,
                dry_run,
            } => archive_large_files(
                path,
                min_size_gb,
                extensions,
                files_only,
                dirs_only,
                to,
                delete_after,
                dry_run,
            )?,
        },
        Commands::Restore {
            from,
            to,
            mode,
            version,
            dry_run,
        } => restore_archive(&from, &to, mode, version.as_deref(), dry_run)?,
        Commands::ListArchives => list_archives()?,
        Commands::Config { action } => match action {
            ConfigAction::Show => show_config()?,
            ConfigAction::InitB2 {
                bucket,
                key_id,
                key,
            } => init_b2_config(bucket, key_id, key)?,
            ConfigAction::TestB2 => test_b2_auth()?,
        },
        Commands::Completions { shell } => {
            generate_completions(shell)?;
        }
        Commands::Diagnose {
            json,
            threshold,
            path,
        } => {
            run_diagnostics(&path, json, threshold)?;
        }
    }

    Ok(())
}

fn clean_rust(search_path: &PathBuf, search: bool, delete: bool, interactive: bool) -> Result<()> {
    println!("{}", "🦀 Rust プロジェクトをスキャン中...".cyan().bold());

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Cargo.toml を検索中...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let projects = kanri_core::rust::find_rust_projects(search_path)?;
    spinner.finish_and_clear();

    if projects.is_empty() {
        println!("{}", "✨ target ディレクトリが見つかりませんでした".green());
        return Ok(());
    }

    let total_size: u64 = projects.iter().map(|p| p.size).sum();

    println!(
        "\n{} 件の Rust プロジェクトを発見 (合計: {})\n",
        projects.len().to_string().yellow().bold(),
        kanri_core::utils::format_size(total_size).yellow().bold()
    );

    // プロジェクト一覧を表示
    for (i, project) in projects.iter().enumerate() {
        println!(
            "  {}. {} - {}",
            (i + 1).to_string().dimmed(),
            project.root.display().to_string().bright_blue(),
            project.formatted_size().yellow()
        );
    }

    // 検索モード（デフォルトまたは --search）
    if search || (!delete && !interactive) {
        println!(
            "\n{} {}",
            "ℹ".cyan(),
            "検索モード: 削除対象を表示しています".dimmed()
        );
        println!(
            "{} {}",
            "💡".cyan(),
            "削除するには --delete (-d) を指定してください".dimmed()
        );
        println!(
            "{} {}",
            "💡".cyan(),
            "確認しながら削除するには --interactive (-i) を指定してください".dimmed()
        );
        return Ok(());
    }

    // インタラクティブモード
    if interactive {
        print!("\n{} 本当に削除しますか? (y/N): ", "⚠".yellow().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "キャンセルされました".yellow());
            return Ok(());
        }
    }

    // 実行モード
    println!("\n{}", "🗑️  削除中...".red().bold());

    let pb = ProgressBar::new(projects.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let cleaned = kanri_core::rust::clean_projects(&projects)?;

    for project in &cleaned {
        pb.inc(1);
        pb.set_message(format!("{}", project.display()));
    }

    pb.finish_and_clear();

    println!(
        "\n{} {} 件のプロジェクトをクリーンしました ({}削除)",
        "✅".green(),
        cleaned.len().to_string().green().bold(),
        kanri_core::utils::format_size(total_size).green().bold()
    );

    Ok(())
}

fn clean_node(search_path: &PathBuf, search: bool, delete: bool, interactive: bool) -> Result<()> {
    println!("{}", "📦 Node.js プロジェクトをスキャン中...".cyan().bold());

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("package.json を検索中...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let projects = kanri_core::node::find_node_projects(search_path)?;
    spinner.finish_and_clear();

    if projects.is_empty() {
        println!(
            "{}",
            "✨ node_modules ディレクトリが見つかりませんでした".green()
        );
        return Ok(());
    }

    let total_size: u64 = projects.iter().map(|p| p.size).sum();

    println!(
        "\n{} 件の Node.js プロジェクトを発見 (合計: {})\n",
        projects.len().to_string().yellow().bold(),
        kanri_core::utils::format_size(total_size).yellow().bold()
    );

    // プロジェクト一覧を表示
    for (i, project) in projects.iter().enumerate() {
        println!(
            "  {}. {} - {}",
            (i + 1).to_string().dimmed(),
            project.root.display().to_string().bright_blue(),
            project.formatted_size().yellow()
        );
    }

    // 検索モード（デフォルトまたは --search）
    if search || (!delete && !interactive) {
        println!(
            "\n{} {}",
            "ℹ".cyan(),
            "検索モード: 削除対象を表示しています".dimmed()
        );
        println!(
            "{} {}",
            "💡".cyan(),
            "削除するには --delete (-d) を指定してください".dimmed()
        );
        println!(
            "{} {}",
            "💡".cyan(),
            "確認しながら削除するには --interactive (-i) を指定してください".dimmed()
        );
        return Ok(());
    }

    // インタラクティブモード
    if interactive {
        print!("\n{} 本当に削除しますか? (y/N): ", "⚠".yellow().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "キャンセルされました".yellow());
            return Ok(());
        }
    }

    // 実行モード
    println!("\n{}", "🗑️  削除中...".red().bold());

    let pb = ProgressBar::new(projects.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let cleaned = kanri_core::node::clean_projects(&projects)?;

    for project in &cleaned {
        pb.inc(1);
        pb.set_message(format!("{}", project.display()));
    }

    pb.finish_and_clear();

    println!(
        "\n{} {} 件のプロジェクトをクリーンしました ({}削除)",
        "✅".green(),
        cleaned.len().to_string().green().bold(),
        kanri_core::utils::format_size(total_size).green().bold()
    );

    Ok(())
}

fn clean_docker(
    search: bool,
    delete: bool,
    interactive: bool,
    all: bool,
    volumes: bool,
) -> Result<()> {
    println!("{}", "🐳 Docker システムをチェック中...".cyan().bold());

    // Docker がインストールされているかチェック
    if !kanri_core::docker::is_docker_installed() {
        println!("{}", "❌ Docker がインストールされていません".red());
        return Ok(());
    }

    // Docker デーモンが起動しているかチェック
    if !kanri_core::docker::is_docker_running() {
        println!("{}", "❌ Docker デーモンが起動していません".red());
        println!("{}", "💡 Docker Desktop を起動してください".dimmed());
        return Ok(());
    }

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Docker システム情報を取得中...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let info = kanri_core::docker::get_system_info()?;
    spinner.finish_and_clear();

    println!(
        "\n{} 削除可能: {}\n",
        "📊".cyan(),
        info.reclaimable.yellow().bold()
    );

    let mut prune_options = Vec::new();
    if all {
        prune_options.push("--all (未使用イメージもすべて削除)");
    }
    if volumes {
        prune_options.push("--volumes (ボリュームも削除)");
    }

    if !prune_options.is_empty() {
        println!("{} オプション:", "⚙".cyan());
        for opt in &prune_options {
            println!("  - {}", opt.dimmed());
        }
        println!();
    }

    // 検索モード（デフォルトまたは --search）
    if search || (!delete && !interactive) {
        println!(
            "{} {}",
            "ℹ".cyan(),
            "検索モード: 削除対象を表示しています".dimmed()
        );
        println!(
            "{} {}",
            "💡".cyan(),
            "削除するには --delete (-d) を指定してください".dimmed()
        );
        println!(
            "{} {}",
            "💡".cyan(),
            "確認しながら削除するには --interactive (-i) を指定してください".dimmed()
        );
        return Ok(());
    }

    // インタラクティブモード
    if interactive {
        print!("\n{} 本当に削除しますか? (y/N): ", "⚠".yellow().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "キャンセルされました".yellow());
            return Ok(());
        }
    }

    // 実行モード
    println!(
        "{}",
        "🗑️  Docker システムをクリーンアップ中...".red().bold()
    );

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.set_message("docker system prune を実行中...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let output = kanri_core::docker::clean_system(all, volumes)?;
    spinner.finish_and_clear();

    println!("\n{}", "✅ クリーンアップ完了".green().bold());
    println!("\n{}", output.dimmed());

    Ok(())
}

fn clean_flutter(
    search_path: &PathBuf,
    search: bool,
    delete: bool,
    interactive: bool,
) -> Result<()> {
    println!("{}", "🦋 Flutter プロジェクトをスキャン中...".cyan().bold());

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("pubspec.yaml を検索中...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let projects = kanri_core::flutter::find_flutter_projects(search_path)?;
    spinner.finish_and_clear();

    if projects.is_empty() {
        println!(
            "{}",
            "✨ Flutter プロジェクトが見つかりませんでした".green()
        );
        return Ok(());
    }

    let total_size: u64 = projects.iter().map(|p| p.size).sum();

    println!(
        "\n{} 件の Flutter プロジェクトを発見 (合計: {})\n",
        projects.len().to_string().yellow().bold(),
        kanri_core::utils::format_size(total_size).yellow().bold()
    );

    // プロジェクト一覧を表示
    for (i, project) in projects.iter().enumerate() {
        println!(
            "  {}. {} - {}",
            (i + 1).to_string().dimmed(),
            project.root.display().to_string().bright_blue(),
            project.formatted_size().yellow()
        );
    }

    // 検索モード（デフォルトまたは --search）
    if search || (!delete && !interactive) {
        println!(
            "\n{} {}",
            "ℹ".cyan(),
            "検索モード: 削除対象を表示しています".dimmed()
        );
        println!(
            "{} {}",
            "💡".cyan(),
            "削除するには --delete (-d) を指定してください".dimmed()
        );
        println!(
            "{} {}",
            "💡".cyan(),
            "確認しながら削除するには --interactive (-i) を指定してください".dimmed()
        );
        return Ok(());
    }

    // インタラクティブモード
    if interactive {
        print!("\n{} 本当に削除しますか? (y/N): ", "⚠".yellow().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "キャンセルされました".yellow());
            return Ok(());
        }
    }

    // 実行モード
    println!("\n{}", "🗑️  削除中...".red().bold());

    let pb = ProgressBar::new(projects.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let cleaned = kanri_core::flutter::clean_projects(&projects)?;

    for project in &cleaned {
        pb.inc(1);
        pb.set_message(format!("{}", project.display()));
    }

    pb.finish_and_clear();

    println!(
        "\n{} {} 件のプロジェクトをクリーンしました ({}削除)",
        "✅".green(),
        cleaned.len().to_string().green().bold(),
        kanri_core::utils::format_size(total_size).green().bold()
    );

    Ok(())
}

fn clean_cache(
    search: bool,
    delete: bool,
    interactive: bool,
    min_size: u64,
    safe_only: bool,
) -> Result<()> {
    // Experimental 警告
    println!("{}", "⚠️  EXPERIMENTAL FEATURE".yellow().bold());
    println!(
        "{}",
        "このコマンドは実験的な機能です。削除前に必ず内容を確認してください。".yellow()
    );
    println!();

    println!(
        "{}",
        "💾 Mac アプリケーションキャッシュをスキャン中..."
            .cyan()
            .bold()
    );
    println!("{}", format!("最小サイズ: {} GB 以上", min_size).dimmed());

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("~/Library/Caches を検索中...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let mut caches = kanri_core::cache::scan_user_caches(min_size)?;
    spinner.finish_and_clear();

    if safe_only {
        caches.retain(|c| c.is_safe);
    }

    if caches.is_empty() {
        println!(
            "{}",
            format!("✨ {} GB 以上のキャッシュが見つかりませんでした", min_size).green()
        );
        return Ok(());
    }

    let total_size: u64 = caches.iter().map(|c| c.size).sum();

    println!(
        "\n{} 件のキャッシュを発見 (合計: {})\n",
        caches.len().to_string().yellow().bold(),
        kanri_core::utils::format_size(total_size).yellow().bold()
    );

    // キャッシュ一覧を表示
    for (i, cache) in caches.iter().enumerate() {
        let safety_icon = if cache.is_safe { "✓" } else { "⚠" };
        let safety_color = if cache.is_safe {
            cache.safety_label().green()
        } else {
            cache.safety_label().yellow()
        };

        println!(
            "  {}. {} {} - {} {}",
            (i + 1).to_string().dimmed(),
            safety_icon,
            cache.name.bright_blue(),
            cache.formatted_size().yellow(),
            safety_color
        );
    }

    // 検索モード（デフォルトまたは --search）
    if search || (!delete && !interactive) {
        println!(
            "\n{} {}",
            "ℹ".cyan(),
            "検索モード: 削除対象を表示しています".dimmed()
        );
        println!(
            "{} {}",
            "💡".cyan(),
            "削除するには --delete (-d) を指定してください".dimmed()
        );
        println!(
            "{} {}",
            "💡".cyan(),
            "確認しながら削除するには --interactive (-i) を指定してください".dimmed()
        );
        println!(
            "{} {}",
            "💡".cyan(),
            "安全なキャッシュのみ表示するには --safe-only を指定してください".dimmed()
        );
        return Ok(());
    }

    // インタラクティブモード: 各キャッシュごとに確認
    let caches_to_delete = if interactive {
        println!("\n{}", "各キャッシュについて個別に確認します".cyan());
        println!(
            "{}",
            "(y)削除 / (n)スキップ / (q)中断 / (a)全て削除".dimmed()
        );
        println!();

        let mut selected_caches = Vec::new();
        let mut delete_all = false;

        for cache in &caches {
            if delete_all {
                selected_caches.push(cache.clone());
                continue;
            }

            let safety_icon = if cache.is_safe { "✓" } else { "⚠" };
            let safety_color = if cache.is_safe {
                cache.safety_label().green()
            } else {
                cache.safety_label().yellow()
            };

            print!(
                "{} {} {} - {} {} を削除しますか? (y/n/q/a): ",
                safety_icon,
                cache.name.bright_blue(),
                cache.formatted_size().yellow(),
                safety_color,
                "".dimmed()
            );
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let choice = input.trim().to_lowercase();

            match choice.as_str() {
                "y" | "yes" => {
                    selected_caches.push(cache.clone());
                    println!("  {} 削除対象に追加", "✓".green());
                }
                "n" | "no" | "" => {
                    println!("  {} スキップ", "→".dimmed());
                }
                "q" | "quit" => {
                    println!("\n{}", "中断しました".yellow());
                    if selected_caches.is_empty() {
                        return Ok(());
                    }
                    break;
                }
                "a" | "all" => {
                    selected_caches.push(cache.clone());
                    delete_all = true;
                    println!("  {} 以降すべて削除対象に追加", "✓".green());
                }
                _ => {
                    println!("  {} 無効な入力、スキップします", "⚠".yellow());
                }
            }
        }

        if selected_caches.is_empty() {
            println!("\n{}", "削除対象がありません".yellow());
            return Ok(());
        }

        println!(
            "\n{} {} 件のキャッシュを削除します",
            "📋".cyan(),
            selected_caches.len()
        );
        selected_caches
    } else {
        // 非インタラクティブモード（--delete）は全て削除
        caches.clone()
    };

    // 実行モード
    println!("\n{}", "🗑️  削除中...".red().bold());

    let pb = ProgressBar::new(caches_to_delete.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let cleaned = kanri_core::cache::clean_caches(&caches_to_delete)?;

    for cache_name in &cleaned {
        pb.inc(1);
        pb.set_message(cache_name.to_string());
    }

    pb.finish_and_clear();

    // 削除したキャッシュの合計サイズを計算
    let deleted_size: u64 = caches_to_delete.iter().map(|c| c.size).sum();

    println!(
        "\n{} {} 件のキャッシュをクリーンしました ({}削除)",
        "✅".green(),
        cleaned.len().to_string().green().bold(),
        kanri_core::utils::format_size(deleted_size).green().bold()
    );

    Ok(())
}

/// Cleanable trait ベースの汎用クリーン関数
fn clean_generic(
    cleaner: &impl kanri_core::Cleanable,
    search_target: &str,
    search: bool,
    delete: bool,
    interactive: bool,
) -> Result<()> {
    println!(
        "{}",
        format!("{} {} をスキャン中...", cleaner.icon(), cleaner.name())
            .cyan()
            .bold()
    );

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("{} を検索中...", search_target));
    spinner.enable_steady_tick(std::time::Duration::from_millis(100));

    let items = cleaner.scan()?;
    spinner.finish_and_clear();

    if items.is_empty() {
        println!(
            "{}",
            format!("✨ {} が見つかりませんでした", search_target).green()
        );
        return Ok(());
    }

    let total_size: u64 = items.iter().map(|item| item.size).sum();

    println!(
        "\n{} 件を発見 (合計: {})\n",
        items.len().to_string().yellow().bold(),
        kanri_core::utils::format_size(total_size).yellow().bold()
    );

    // 一覧を表示
    for (i, item) in items.iter().enumerate() {
        let display = if let Some(safety_label) = item.safety_label() {
            let safety_icon = if item.is_safe() { "✓" } else { "⚠" };
            let safety_color = if item.is_safe() {
                safety_label.green()
            } else {
                safety_label.yellow()
            };
            format!(
                "  {}. {} {} - {} {}",
                (i + 1).to_string().dimmed(),
                safety_icon,
                item.name.bright_blue(),
                item.formatted_size().yellow(),
                safety_color
            )
        } else {
            format!(
                "  {}. {} - {}",
                (i + 1).to_string().dimmed(),
                item.name.bright_blue(),
                item.formatted_size().yellow()
            )
        };
        println!("{}", display);
    }

    // 検索モード（デフォルトまたは --search）
    if search || (!delete && !interactive) {
        println!(
            "\n{} {}",
            "ℹ".cyan(),
            "検索モード: 削除対象を表示しています".dimmed()
        );
        println!(
            "{} {}",
            "💡".cyan(),
            "削除するには --delete (-d) を指定してください".dimmed()
        );
        println!(
            "{} {}",
            "💡".cyan(),
            "確認しながら削除するには --interactive (-i) を指定してください".dimmed()
        );
        return Ok(());
    }

    // インタラクティブモード
    if interactive {
        print!("\n{} 本当に削除しますか? (y/N): ", "⚠".yellow().bold());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "キャンセルされました".yellow());
            return Ok(());
        }
    }

    // 実行モード
    println!("\n{}", "🗑️  削除中...".red().bold());

    let pb = ProgressBar::new(items.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let cleaned = kanri_core::cleanable::clean_items(&items)?;

    for item_name in &cleaned {
        pb.inc(1);
        pb.set_message(item_name.to_string());
    }

    pb.finish_and_clear();

    println!(
        "\n{} {} 件をクリーンしました ({}削除)",
        "✅".green(),
        cleaned.len().to_string().green().bold(),
        kanri_core::utils::format_size(total_size).green().bold()
    );

    Ok(())
}

// ========== Archive / Restore Functions ==========

fn archive_large_files(
    path: PathBuf,
    min_size_gb: u64,
    extensions: Option<String>,
    files_only: bool,
    dirs_only: bool,
    to: String,
    delete_after: bool,
    dry_run: bool,
) -> Result<()> {
    use kanri_core::{archive, config, large_files};

    println!("{}", "📦 アーカイブ処理を開始...".cyan().bold());

    // 設定読み込み
    let config = config::Config::load()?;
    let bucket = config.get_b2_bucket()?;
    let backend = config.get_storage_backend();

    // Storage client を作成
    let storage_client = config.create_storage_client()?;

    // 認証
    println!(
        "{}",
        format!("🔐 {} 認証中...", backend.to_uppercase()).cyan()
    );
    storage_client.authorize()?;

    // 大きなファイルを検索
    let min_size = min_size_gb * 1024 * 1024 * 1024;
    let ext_vec: Option<Vec<String>> =
        extensions.map(|s| s.split(',').map(|e| e.trim().to_string()).collect());

    let (include_files, include_dirs) = match (files_only, dirs_only) {
        (true, true) => {
            eprintln!(
                "{}",
                "Error: --files-only and --dirs-only cannot be used together".red()
            );
            return Ok(());
        }
        (true, false) => (true, false),
        (false, true) => (false, true),
        (false, false) => (true, true),
    };

    let items = large_files::find_large_items(
        &path,
        min_size,
        ext_vec.as_deref(),
        include_dirs,
        include_files,
    )?;

    if items.is_empty() {
        println!("{}", "ℹ アーカイブ対象が見つかりませんでした".yellow());
        return Ok(());
    }

    println!(
        "\n{} 件のアイテムが見つかりました (合計: {})",
        items.len().to_string().cyan().bold(),
        kanri_core::utils::format_size(items.iter().map(|i| i.size).sum())
            .cyan()
            .bold()
    );

    // リスト表示
    for (i, item) in items.iter().enumerate().take(10) {
        let type_label = if item.is_dir { "dir" } else { "file" };
        println!(
            "  {}. {} ({}) - {}",
            i + 1,
            item.path.display(),
            type_label,
            kanri_core::utils::format_size(item.size)
        );
    }
    if items.len() > 10 {
        println!("  ... 他 {} 件", items.len() - 10);
    }

    // タイムスタンプ付きパスを生成（自動バージョニング）
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let versioned_path = format!("{}/{}", to, timestamp);

    println!(
        "\n{} {}",
        "📍 アーカイブ先:".cyan().bold(),
        versioned_path.cyan()
    );

    if dry_run {
        println!(
            "\n{}",
            "ℹ Dry-run モード: 実際のアップロードは行いません".yellow()
        );
        println!("\n{}", "アップロード予定:".cyan().bold());
        for item in &items {
            let relative_path = item.path.strip_prefix(&path).unwrap_or(item.path.as_path());
            let remote_path = PathBuf::from(&versioned_path).join(relative_path);
            println!(
                "  {} -> {}",
                item.path.display(),
                remote_path.display().to_string().green()
            );
        }
        return Ok(());
    }

    // アーカイブ作成
    let mut archive_record =
        archive::Archive::new("large-files".to_string(), versioned_path.clone());

    // アップロード
    println!("\n{}", "⬆️ B2 にアップロード中...".cyan().bold());

    for item in &items {
        let relative_path = item.path.strip_prefix(&path).unwrap_or(item.path.as_path());
        let remote_path = PathBuf::from(&versioned_path).join(relative_path);
        let remote_path_str = remote_path.to_string_lossy();

        println!(
            "  📤 {} -> {}",
            item.path.display(),
            remote_path.display().to_string().green()
        );

        if item.is_dir {
            let _files = storage_client.upload_directory(&bucket, &item.path, &remote_path_str)?;
        } else {
            let _file_id = storage_client.upload_file(&bucket, &item.path, &remote_path_str)?;
        }

        let archive_item =
            archive::ArchiveItem::from_file(&item.path, remote_path_str.to_string())?;
        archive_record.add_item(archive_item);

        println!("    {}", "✅ 完了".green());
    }

    // アーカイブインデックスに追加
    let mut index = archive::ArchiveIndex::load()?;
    index.add_archive(archive_record.clone());
    index.save()?;

    println!(
        "\n{} アーカイブ完了 (ID: {})",
        "✅".green(),
        archive_record.id.green().bold()
    );

    // delete_after が指定されている場合は削除
    if delete_after {
        println!("\n{}", "🗑️ ローカルファイルを削除中...".yellow());
        for item in &items {
            if item.path.exists() {
                if item.is_dir {
                    std::fs::remove_dir_all(&item.path)?;
                } else {
                    std::fs::remove_file(&item.path)?;
                }
                println!("  {} {}", "✅".green(), item.path.display());
            }
        }
        println!("{}", "✅ ローカルファイルを削除しました".green());
    }

    Ok(())
}

fn restore_archive(
    from: &str,
    to: &str,
    mode: RestoreMode,
    version: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    use kanri_core::config;
    use std::collections::HashMap;

    println!("{}", "📥 アーカイブ復元処理を開始...".cyan().bold());

    // 設定読み込み
    let config = config::Config::load()?;
    let bucket = config.get_b2_bucket()?;
    let backend = config.get_storage_backend();

    let storage_client = config.create_storage_client()?;

    // 認証
    println!(
        "{}",
        format!("🔐 {} 認証中...", backend.to_uppercase()).cyan()
    );
    storage_client.authorize()?;

    // ファイル一覧を取得
    println!(
        "{}",
        format!("📋 {} からファイル一覧を取得中...", backend.to_uppercase()).cyan()
    );
    let all_files = storage_client.list_files(&bucket, from)?;

    if all_files.is_empty() {
        println!("{}", "⚠️ 該当するファイルが見つかりませんでした".yellow());
        return Ok(());
    }

    println!("  {} {} 個のファイルを検出", "✅".green(), all_files.len());

    // タイムスタンプを抽出するヘルパー関数
    fn extract_timestamp(path: &str) -> Option<String> {
        // YYYYMMDD_HHMMSS パターンを探す
        for part in path.split('/') {
            if part.len() == 15 && part.chars().nth(8) == Some('_') {
                let before_underscore = &part[..8];
                let after_underscore = &part[9..];
                if before_underscore.chars().all(|c| c.is_ascii_digit())
                    && after_underscore.chars().all(|c| c.is_ascii_digit())
                {
                    return Some(part.to_string());
                }
            }
        }
        None
    }

    // タイムスタンプを除去するヘルパー関数
    fn remove_timestamp(path: &str, timestamp: &str) -> String {
        path.replace(&format!("/{}/", timestamp), "/")
    }

    // モードに応じてファイルをフィルタリング
    let files_to_restore: Vec<(String, String)> = match mode {
        RestoreMode::Latest => {
            // タイムスタンプを除いた相対パスでグループ化
            let mut file_groups: HashMap<String, Vec<String>> = HashMap::new();

            for file in &all_files {
                if let Some(timestamp) = extract_timestamp(file) {
                    // タイムスタンプを除去した正規化パス
                    let normalized = remove_timestamp(file, &timestamp);
                    file_groups
                        .entry(normalized)
                        .or_default()
                        .push(file.clone());
                }
            }

            // 各グループで最新のタイムスタンプを持つファイルを選択
            let mut selected_files = Vec::new();
            for (_normalized_path, mut files) in file_groups {
                files.sort_by(|a, b| b.cmp(a)); // 降順ソート（最新が先頭）
                if let Some(latest_file) = files.first() {
                    // タイムスタンプを除去した復元先パスを生成
                    if let Some(timestamp) = extract_timestamp(latest_file) {
                        let restore_path = remove_timestamp(latest_file, &timestamp);
                        // from プレフィックスを除去
                        let restore_path = restore_path
                            .strip_prefix(from)
                            .unwrap_or(&restore_path)
                            .trim_start_matches('/');
                        selected_files.push((latest_file.clone(), restore_path.to_string()));
                    }
                }
            }

            selected_files
        }
        RestoreMode::Version => {
            // 特定バージョンを指定
            let version_str =
                version.ok_or_else(|| anyhow::anyhow!("--version が指定されていません"))?;

            all_files
                .iter()
                .filter(|file| file.contains(&format!("/{}/", version_str)))
                .map(|file| {
                    // タイムスタンプを除去した復元先パス
                    let restore_path = if let Some(timestamp) = extract_timestamp(file) {
                        remove_timestamp(file, &timestamp)
                    } else {
                        file.to_string()
                    };
                    let restore_path = restore_path
                        .strip_prefix(from)
                        .unwrap_or(&restore_path)
                        .trim_start_matches('/');
                    (file.clone(), restore_path.to_string())
                })
                .collect()
        }
        RestoreMode::Raw => {
            // タイムスタンプ付きでそのまま復元
            all_files
                .iter()
                .map(|file| {
                    let restore_path = file
                        .strip_prefix(from)
                        .unwrap_or(file)
                        .trim_start_matches('/');
                    (file.clone(), restore_path.to_string())
                })
                .collect()
        }
    };

    if files_to_restore.is_empty() {
        println!("{}", "⚠️ 復元対象のファイルがありません".yellow());
        return Ok(());
    }

    // モード表示
    let mode_str = match mode {
        RestoreMode::Latest => "最新版のみ復元".to_string(),
        RestoreMode::Version => format!("バージョン {} を復元", version.unwrap()),
        RestoreMode::Raw => "タイムスタンプ付きでフル復元".to_string(),
    };
    println!("\n{} {}", "📦 復元モード:".cyan(), mode_str);
    println!(
        "{} {} 個のファイルを復元",
        "📥".cyan(),
        files_to_restore.len()
    );

    // Dry-run モード
    if dry_run {
        println!(
            "\n{}",
            "ℹ  Dry-run モード: 実際のダウンロードは行いません".yellow()
        );
        println!("\n{}", "ダウンロード予定:".cyan().bold());
        for (remote_file, local_path) in &files_to_restore {
            let full_local_path = std::path::Path::new(to).join(local_path);
            println!(
                "  {} -> {}",
                remote_file,
                full_local_path.display().to_string().green()
            );
        }
        return Ok(());
    }

    // 実際にダウンロード
    println!("\n{}", "⬇️  B2 からダウンロード中...".cyan().bold());

    for (remote_file, local_path) in &files_to_restore {
        let full_local_path = std::path::Path::new(to).join(local_path);

        println!("  📥 {} -> {}", remote_file, full_local_path.display());

        // 親ディレクトリを作成
        if let Some(parent) = full_local_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        storage_client.download_file_by_name(&bucket, remote_file, &full_local_path)?;
        println!("    {}", "✅ 完了".green());
    }

    println!("\n{}", "✅ 復元完了".green());

    Ok(())
}

fn list_archives() -> Result<()> {
    use kanri_core::archive;

    let index = archive::ArchiveIndex::load()?;

    if index.archives.is_empty() {
        println!("{}", "ℹ アーカイブが見つかりませんでした".yellow());
        return Ok(());
    }

    println!(
        "{}",
        format!("📦 アーカイブ一覧 ({} 件)", index.archives.len())
            .cyan()
            .bold()
    );

    for archive in &index.archives {
        println!("\n{}", "─".repeat(80).dimmed());
        println!("ID:         {}", archive.id.cyan().bold());
        println!(
            "作成日時:   {}",
            archive.created_at.format("%Y-%m-%d %H:%M:%S")
        );
        println!("クリーナー: {}", archive.cleaner);
        println!("保存先:     {}", archive.destination);
        println!("アイテム数: {}", archive.items.len());
        println!(
            "合計サイズ: {}",
            kanri_core::utils::format_size(archive.total_size)
        );
    }

    Ok(())
}

fn show_config() -> Result<()> {
    use kanri_core::config;

    let config = config::Config::load()?;

    println!("{}", "⚙️ 現在の設定".cyan().bold());
    println!();

    if let Some(b2) = &config.b2 {
        println!("{}:", "B2 Configuration".green().bold());
        println!("  Bucket: {}", b2.bucket);
        println!(
            "  Application Key ID: {}",
            b2.application_key_id
                .as_ref()
                .map(|_| "****")
                .unwrap_or("(環境変数)")
        );
        println!(
            "  Application Key: {}",
            b2.application_key
                .as_ref()
                .map(|_| "****")
                .unwrap_or("(環境変数)")
        );
    } else {
        println!("{}", "B2 が設定されていません".yellow());
        println!(
            "設定するには: {}",
            "kanri config init-b2 --bucket <bucket-name>".cyan()
        );
    }

    println!();
    println!("設定ファイル: {}", config::Config::config_path()?.display());

    Ok(())
}

fn init_b2_config(bucket: String, key_id: Option<String>, key: Option<String>) -> Result<()> {
    use kanri_core::config;

    let mut config = config::Config::load().unwrap_or_default();

    config.b2 = Some(config::B2Config {
        bucket: bucket.clone(),
        application_key_id: key_id,
        application_key: key,
    });

    config.save_with_template()?;

    println!("{}", "✅ B2 設定を保存しました".green().bold());
    println!("  Bucket: {}", bucket.cyan());
    println!();
    println!(
        "{}",
        "💡 認証情報は環境変数で設定することを推奨します:".yellow()
    );
    println!("  export B2_APPLICATION_KEY_ID=<your-key-id>");
    println!("  export B2_APPLICATION_KEY=<your-key>");

    Ok(())
}

fn test_b2_auth() -> Result<()> {
    use kanri_core::{b2, config};

    println!("{}", "🔐 B2 認証テスト...".cyan().bold());
    println!();

    // B2 CLI チェック
    if !b2::B2Client::is_installed() {
        eprintln!("{}", "❌ B2 CLI がインストールされていません".red());
        eprintln!(
            "{}",
            "インストール: pip install b2 または brew install b2-tools".yellow()
        );
        return Ok(());
    }
    println!("{}", "✅ B2 CLI インストール確認済み".green());

    // 設定読み込み
    let config = config::Config::load()?;

    // バケット確認
    match config.get_b2_bucket() {
        Ok(bucket) => println!("{} {}", "✅ バケット設定:".green(), bucket.cyan()),
        Err(e) => {
            eprintln!("{} {}", "❌ バケット未設定:".red(), e);
            return Ok(());
        }
    }

    // 認証情報確認
    let (key_id, key) = match config.get_b2_credentials() {
        Ok((id, k)) => {
            println!("{}", "✅ 認証情報取得成功".green());
            println!("  Key ID: {}***", &id.chars().take(8).collect::<String>());
            (id, k)
        }
        Err(e) => {
            eprintln!("{} {}", "❌ 認証情報取得失敗:".red(), e);
            eprintln!();
            eprintln!("{}", "環境変数を設定してください:".yellow());
            eprintln!("  export B2_APPLICATION_KEY_ID=<your-key-id>");
            eprintln!("  export B2_APPLICATION_KEY=<your-key>");
            return Ok(());
        }
    };

    // B2Client 作成（空チェック）
    println!();
    println!("{}", "🔑 B2 認証を試行中...".cyan());
    let b2_client = match b2::B2Client::new(key_id, key) {
        Ok(client) => {
            println!("{}", "✅ 認証情報の形式チェック OK".green());
            client
        }
        Err(e) => {
            eprintln!("{} {}", "❌ 認証情報エラー:".red(), e);
            return Ok(());
        }
    };

    // 実際に認証を試す
    match b2_client.authorize() {
        Ok(_) => {
            println!();
            println!("{}", "✅ B2 認証成功！".green().bold());
            println!("{}", "認証情報は正しく設定されています。".green());
        }
        Err(e) => {
            println!();
            eprintln!("{}", "❌ B2 認証失敗".red().bold());
            eprintln!();
            eprintln!("{} {}", "エラー詳細:".yellow(), e);
            eprintln!();
            eprintln!("{}", "考えられる原因:".yellow());
            eprintln!("  1. Application Key ID または Application Key が間違っている");
            eprintln!("  2. キーの権限が不足している（readFiles, writeFiles が必要）");
            eprintln!("  3. ネットワーク接続の問題");
            eprintln!();
            eprintln!("{}", "確認方法:".cyan());
            eprintln!("  1. B2 コンソールで新しいキーを発行");
            eprintln!("  2. 環境変数を再設定:");
            eprintln!("     export B2_APPLICATION_KEY_ID=<new-key-id>");
            eprintln!("     export B2_APPLICATION_KEY=<new-key>");
            eprintln!("  3. 再度テスト: kanri config test-b2");
        }
    }

    Ok(())
}

fn generate_completions(shell: Shell) -> Result<()> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();

    generate(shell, &mut cmd, bin_name, &mut io::stdout());

    Ok(())
}

// ========== Diagnostic Functions ==========

#[derive(Debug, Serialize, Deserialize)]
struct DiagnosticCategory {
    name: String,
    icon: String,
    count: usize,
    total_size: u64,
    command_hint: String,
    is_large: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiagnosticReport {
    categories: Vec<DiagnosticCategory>,
    total_size: u64,
    timestamp: String,
}

fn run_diagnostics(path: &PathBuf, json: bool, threshold: Option<f64>) -> Result<()> {
    if !json {
        println!("{}", "🔍 システム診断を実行中...".cyan().bold());
        println!();
    }

    let threshold_bytes = threshold.map(|gb| (gb * 1024.0 * 1024.0 * 1024.0) as u64);

    let mut categories = Vec::new();

    // Rust プロジェクト
    if let Ok(projects) = kanri_core::rust::find_rust_projects(path) {
        let total_size: u64 = projects.iter().map(|p| p.size).sum();
        if threshold_bytes.is_none() || total_size >= threshold_bytes.unwrap() {
            categories.push(DiagnosticCategory {
                name: "Rust プロジェクト".to_string(),
                icon: "🦀".to_string(),
                count: projects.len(),
                total_size,
                command_hint: format!("kanri clean rust -p {} -i", path.display()),
                is_large: total_size > 5 * 1024 * 1024 * 1024, // 5GB以上
            });
        }
    }

    // Node.js プロジェクト
    if let Ok(projects) = kanri_core::node::find_node_projects(path) {
        let total_size: u64 = projects.iter().map(|p| p.size).sum();
        if threshold_bytes.is_none() || total_size >= threshold_bytes.unwrap() {
            categories.push(DiagnosticCategory {
                name: "Node.js プロジェクト".to_string(),
                icon: "📦".to_string(),
                count: projects.len(),
                total_size,
                command_hint: format!("kanri clean node -p {} -i", path.display()),
                is_large: total_size > 10 * 1024 * 1024 * 1024, // 10GB以上
            });
        }
    }

    // Flutter プロジェクト
    if let Ok(projects) = kanri_core::flutter::find_flutter_projects(path) {
        let total_size: u64 = projects.iter().map(|p| p.size).sum();
        if threshold_bytes.is_none() || total_size >= threshold_bytes.unwrap() {
            categories.push(DiagnosticCategory {
                name: "Flutter プロジェクト".to_string(),
                icon: "🦋".to_string(),
                count: projects.len(),
                total_size,
                command_hint: format!("kanri clean flutter -p {} -i", path.display()),
                is_large: total_size > 5 * 1024 * 1024 * 1024,
            });
        }
    }

    // Python 仮想環境
    let python_cleaner = kanri_core::python::PythonCleaner::new(path.clone());
    if let Ok(items) = python_cleaner.scan() {
        let total_size: u64 = items.iter().map(|p| p.size).sum();
        if threshold_bytes.is_none() || total_size >= threshold_bytes.unwrap() {
            categories.push(DiagnosticCategory {
                name: "Python 仮想環境".to_string(),
                icon: "🐍".to_string(),
                count: items.len(),
                total_size,
                command_hint: format!("kanri clean python -p {} -i", path.display()),
                is_large: total_size > 3 * 1024 * 1024 * 1024,
            });
        }
    }

    // Haskell プロジェクト
    let haskell_cleaner = kanri_core::haskell::HaskellCleaner::new(path.clone());
    if let Ok(items) = haskell_cleaner.scan() {
        let total_size: u64 = items.iter().map(|p| p.size).sum();
        if threshold_bytes.is_none() || total_size >= threshold_bytes.unwrap() {
            categories.push(DiagnosticCategory {
                name: "Haskell プロジェクト".to_string(),
                icon: "λ".to_string(),
                count: items.len(),
                total_size,
                command_hint: format!("kanri clean haskell -p {} -i", path.display()),
                is_large: total_size > 2 * 1024 * 1024 * 1024,
            });
        }
    }

    // Docker
    if kanri_core::docker::is_docker_installed() && kanri_core::docker::is_docker_running() {
        if let Ok(info) = kanri_core::docker::get_system_info() {
            // reclaimable は "X.X GB" のような形式なので、パースする
            if let Some(size_str) = info.reclaimable.split_whitespace().next() {
                if let Ok(size_gb) = size_str.parse::<f64>() {
                    let total_size = (size_gb * 1024.0 * 1024.0 * 1024.0) as u64;
                    if threshold_bytes.is_none() || total_size >= threshold_bytes.unwrap() {
                        categories.push(DiagnosticCategory {
                            name: "Docker".to_string(),
                            icon: "🐳".to_string(),
                            count: 1,
                            total_size,
                            command_hint: "kanri clean docker -i".to_string(),
                            is_large: total_size > 5 * 1024 * 1024 * 1024,
                        });
                    }
                }
            }
        }
    }

    // Go モジュールキャッシュ
    let go_cleaner = kanri_core::go::GoCleaner::new();
    if let Ok(items) = go_cleaner.scan() {
        let total_size: u64 = items.iter().map(|p| p.size).sum();
        if threshold_bytes.is_none() || total_size >= threshold_bytes.unwrap() {
            categories.push(DiagnosticCategory {
                name: "Go モジュールキャッシュ".to_string(),
                icon: "🐹".to_string(),
                count: items.len(),
                total_size,
                command_hint: "kanri clean go -i".to_string(),
                is_large: total_size > 2 * 1024 * 1024 * 1024,
            });
        }
    }

    // Gradle キャッシュ
    let gradle_cleaner = kanri_core::gradle::GradleCleaner::new();
    if let Ok(items) = gradle_cleaner.scan() {
        let total_size: u64 = items.iter().map(|p| p.size).sum();
        if threshold_bytes.is_none() || total_size >= threshold_bytes.unwrap() {
            categories.push(DiagnosticCategory {
                name: "Gradle キャッシュ".to_string(),
                icon: "🐘".to_string(),
                count: items.len(),
                total_size,
                command_hint: "kanri clean gradle -i".to_string(),
                is_large: total_size > 3 * 1024 * 1024 * 1024,
            });
        }
    }

    // Xcode DerivedData
    let xcode_cleaner = kanri_core::xcode::XcodeCleaner::new();
    if let Ok(items) = xcode_cleaner.scan() {
        let total_size: u64 = items.iter().map(|p| p.size).sum();
        if threshold_bytes.is_none() || total_size >= threshold_bytes.unwrap() {
            categories.push(DiagnosticCategory {
                name: "Xcode DerivedData".to_string(),
                icon: "🍎".to_string(),
                count: items.len(),
                total_size,
                command_hint: "kanri clean xcode -i".to_string(),
                is_large: total_size > 5 * 1024 * 1024 * 1024,
            });
        }
    }

    // アプリケーションキャッシュ (1GB以上)
    if let Ok(caches) = kanri_core::cache::scan_user_caches(1) {
        let total_size: u64 = caches.iter().map(|c| c.size).sum();
        if threshold_bytes.is_none() || total_size >= threshold_bytes.unwrap() {
            categories.push(DiagnosticCategory {
                name: "アプリケーションキャッシュ (1GB以上)".to_string(),
                icon: "💾".to_string(),
                count: caches.len(),
                total_size,
                command_hint: "kanri clean cache -i".to_string(),
                is_large: total_size > 10 * 1024 * 1024 * 1024,
            });
        }
    }

    // 大きなファイル (2GB以上)
    let min_size = 2 * 1024 * 1024 * 1024; // 2GB
    if let Ok(large_items) = kanri_core::large_files::find_large_items(
        path, min_size, None, // extensions
        true, // include_dirs
        true, // include_files
    ) {
        let total_size: u64 = large_items.iter().map(|i| i.size).sum();
        if threshold_bytes.is_none() || total_size >= threshold_bytes.unwrap() {
            categories.push(DiagnosticCategory {
                name: "大きなファイル (2GB以上)".to_string(),
                icon: "📁".to_string(),
                count: large_items.len(),
                total_size,
                command_hint: format!(
                    "kanri archive large-files -p {} --to archive/large-files --delete-after --dry-run",
                    path.display()
                ),
                is_large: total_size > 10 * 1024 * 1024 * 1024,
            });
        }
    }

    // 空のカテゴリ（count=0 または total_size=0）を除外
    categories.retain(|c| c.count > 0 && c.total_size > 0);

    // 総計
    let total_size: u64 = categories.iter().map(|c| c.total_size).sum();

    let report = DiagnosticReport {
        categories,
        total_size,
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        print_diagnostic_report(&report);
    }

    Ok(())
}

fn print_diagnostic_report(report: &DiagnosticReport) {
    if report.categories.is_empty() {
        println!(
            "{}",
            "✨ クリーンアップ可能な項目が見つかりませんでした".green()
        );
        return;
    }

    println!("{}", "━".repeat(60).dimmed());
    println!("{}", "📊 クリーンアップ可能な項目".cyan().bold());
    println!();

    for category in &report.categories {
        let size_str = kanri_core::utils::format_size(category.total_size);
        let warning = if category.is_large {
            " ⚠️  (大)".yellow().to_string()
        } else {
            "".to_string()
        };

        println!("{} {}", category.icon, category.name.bright_white().bold());
        println!("  • {} 件", category.count.to_string().cyan());
        println!("  • 合計: {}{}", size_str.yellow().bold(), warning);
        println!();
    }

    println!("{}", "━".repeat(60).dimmed());
    println!("{}", "📈 サマリー".cyan().bold());
    println!();
    println!(
        "  合計削除可能: {}",
        kanri_core::utils::format_size(report.total_size)
            .yellow()
            .bold()
    );
    println!();

    // 2GB以上のカテゴリのみを「次のアクション」に表示（サイズ順）
    let mut actionable_categories: Vec<&DiagnosticCategory> = report
        .categories
        .iter()
        .filter(|c| c.total_size >= 2 * 1024 * 1024 * 1024) // 2GB以上
        .collect();

    // サイズの大きい順にソート
    actionable_categories.sort_by(|a, b| b.total_size.cmp(&a.total_size));

    if !actionable_categories.is_empty() {
        println!("{}", "💡 次のアクション (2GB以上):".cyan().bold());
        for category in actionable_categories.iter().take(5) {
            println!("  • {}", category.command_hint.dimmed());
        }
        if actionable_categories.len() > 5 {
            println!("  • ... 他 {} 件", actionable_categories.len() - 5);
        }
    }

    println!();
    println!("{}", format!("診断実行日時: {}", report.timestamp).dimmed());
}
