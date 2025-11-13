use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "kanri")]
#[command(author, version, about = "Mac ローカル環境管理ツール", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
        /// アーカイブ ID
        archive_id: String,

        /// 復元先ディレクトリ（デフォルト: 元の場所）
        #[arg(short, long)]
        to: Option<PathBuf>,

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
                clean_generic(&cleaner, "*.cabal or stack.yaml", search, delete, interactive)?
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
            } => {
                archive_large_files(
                    path,
                    min_size_gb,
                    extensions,
                    files_only,
                    dirs_only,
                    to,
                    delete_after,
                    dry_run,
                )?
            }
        },
        Commands::Restore {
            archive_id,
            to,
            dry_run,
        } => restore_archive(archive_id, to, dry_run)?,
        Commands::ListArchives => list_archives()?,
        Commands::Config { action } => match action {
            ConfigAction::Show => show_config()?,
            ConfigAction::InitB2 {
                bucket,
                key_id,
                key,
            } => init_b2_config(bucket, key_id, key)?,
        },
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
        print!(
            "\n{} 本当に削除しますか? (y/N): ",
            "⚠".yellow().bold()
        );
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
        println!("{}", "✨ node_modules ディレクトリが見つかりませんでした".green());
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
        print!(
            "\n{} 本当に削除しますか? (y/N): ",
            "⚠".yellow().bold()
        );
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

fn clean_docker(search: bool, delete: bool, interactive: bool, all: bool, volumes: bool) -> Result<()> {
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
        print!(
            "\n{} 本当に削除しますか? (y/N): ",
            "⚠".yellow().bold()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "キャンセルされました".yellow());
            return Ok(());
        }
    }

    // 実行モード
    println!("{}", "🗑️  Docker システムをクリーンアップ中...".red().bold());

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

fn clean_cache(search: bool, delete: bool, interactive: bool, min_size: u64, safe_only: bool) -> Result<()> {
    // Experimental 警告
    println!("{}", "⚠️  EXPERIMENTAL FEATURE".yellow().bold());
    println!(
        "{}",
        "このコマンドは実験的な機能です。削除前に必ず内容を確認してください。"
            .yellow()
    );
    println!();

    println!("{}", "💾 Mac アプリケーションキャッシュをスキャン中...".cyan().bold());
    println!(
        "{}",
        format!("最小サイズ: {} GB 以上", min_size).dimmed()
    );

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

    // インタラクティブモード
    if interactive {
        println!(
            "\n{} {}",
            "⚠".red().bold(),
            "削除するキャッシュを確認してください。".yellow()
        );
        println!(
            "{}",
            "アプリケーションによっては再ダウンロードが必要になる場合があります。"
                .dimmed()
        );
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

    let pb = ProgressBar::new(caches.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let cleaned = kanri_core::cache::clean_caches(&caches)?;

    for cache_name in &cleaned {
        pb.inc(1);
        pb.set_message(cache_name.to_string());
    }

    pb.finish_and_clear();

    println!(
        "\n{} {} 件のキャッシュをクリーンしました ({}削除)",
        "✅".green(),
        cleaned.len().to_string().green().bold(),
        kanri_core::utils::format_size(total_size).green().bold()
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
        print!(
            "\n{} 本当に削除しますか? (y/N): ",
            "⚠".yellow().bold()
        );
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
    use kanri_core::{archive, b2, config, large_files};

    println!("{}", "📦 アーカイブ処理を開始...".cyan().bold());

    // 設定読み込み
    let config = config::Config::load()?;
    let bucket = config.get_b2_bucket()?;
    let (key_id, key) = config.get_b2_credentials()?;

    // B2 CLI チェック
    if !b2::B2Client::is_installed() {
        eprintln!("{}", "❌ B2 CLI がインストールされていません".red());
        eprintln!(
            "{}",
            "インストール: pip install b2 または brew install b2-tools".yellow()
        );
        return Ok(());
    }

    let b2_client = b2::B2Client::new(key_id, key);

    // 大きなファイルを検索
    let min_size = min_size_gb * 1024 * 1024 * 1024;
    let ext_vec: Option<Vec<String>> = extensions.map(|s| s.split(',').map(|e| e.trim().to_string()).collect());

    let (include_files, include_dirs) = match (files_only, dirs_only) {
        (true, true) => {
            eprintln!("{}", "Error: --files-only and --dirs-only cannot be used together".red());
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
        kanri_core::utils::format_size(items.iter().map(|i| i.size).sum()).cyan().bold()
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
        println!("\n{}", "ℹ Dry-run モード: 実際のアップロードは行いません".yellow());
        println!("\n{}", "アップロード予定:".cyan().bold());
        for item in &items {
            let relative_path = item.path.strip_prefix(&path).unwrap_or(item.path.as_path());
            let remote_path = format!("{}/{}", versioned_path, relative_path.to_string_lossy());
            println!("  {} -> {}", item.path.display(), remote_path.green());
        }
        return Ok(());
    }

    // アーカイブ作成
    let mut archive_record = archive::Archive::new("large-files".to_string(), versioned_path.clone());

    // アップロード
    println!("\n{}", "⬆️ B2 にアップロード中...".cyan().bold());

    for item in &items {
        // 検索パスからの相対パスを保持
        let relative_path = item.path.strip_prefix(&path).unwrap_or(item.path.as_path());
        let remote_path = format!("{}/{}", versioned_path, relative_path.to_string_lossy());

        println!("  📤 {} -> {}", item.path.display(), remote_path.green());

        if item.is_dir {
            let _files = b2_client.upload_directory(&bucket, &item.path, &remote_path)?;
        } else {
            let _file_id = b2_client.upload_file(&bucket, &item.path, &remote_path)?;
        }

        let archive_item = archive::ArchiveItem::from_file(&item.path, remote_path)?;
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

fn restore_archive(archive_id: String, to: Option<PathBuf>, dry_run: bool) -> Result<()> {
    use kanri_core::{archive, b2, config};

    println!("{}", "📥 アーカイブ復元処理を開始...".cyan().bold());

    // アーカイブインデックスを読み込み
    let index = archive::ArchiveIndex::load()?;
    let archive = index
        .find_by_id(&archive_id)
        .ok_or_else(|| anyhow::anyhow!("Archive ID {} not found", archive_id))?;

    println!(
        "アーカイブ: {} (作成日時: {})",
        archive.id.cyan().bold(),
        archive.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    println!("アイテム数: {}", archive.items.len());
    println!(
        "合計サイズ: {}",
        kanri_core::utils::format_size(archive.total_size)
    );

    if dry_run {
        println!("\n{}", "ℹ Dry-run モード: 実際のダウンロードは行いません".yellow());
        for item in &archive.items {
            let restore_path = if let Some(ref dest) = to {
                dest.join(item.local_path.file_name().unwrap())
            } else {
                item.local_path.clone()
            };
            println!("  {} -> {}", item.b2_path, restore_path.display());
        }
        return Ok(());
    }

    // 設定読み込み
    let config = config::Config::load()?;
    let bucket = config.get_b2_bucket()?;
    let (key_id, key) = config.get_b2_credentials()?;

    let b2_client = b2::B2Client::new(key_id, key);

    // ダウンロード
    println!("\n{}", "⬇️ B2 からダウンロード中...".cyan().bold());

    for item in &archive.items {
        let restore_path = if let Some(ref dest) = to {
            dest.join(item.local_path.file_name().unwrap())
        } else {
            item.local_path.clone()
        };

        println!("  📥 {} -> {}", item.b2_path, restore_path.display());

        // 親ディレクトリを作成
        if let Some(parent) = restore_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        b2_client.download_file_by_name(&bucket, &item.b2_path, &restore_path)?;

        // SHA256 検証
        if !item.is_dir && !item.sha256.is_empty() {
            let downloaded_hash = b2::B2Client::calculate_sha256(&restore_path)?;
            if downloaded_hash != item.sha256 {
                eprintln!("    {} SHA256 mismatch!", "⚠️".yellow());
                eprintln!("      Expected: {}", item.sha256);
                eprintln!("      Got:      {}", downloaded_hash);
            } else {
                println!("    {}", "✅ 整合性確認済み".green());
            }
        }

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
        println!("設定するには: {}", "kanri config init-b2 --bucket <bucket-name>".cyan());
    }

    println!();
    println!(
        "設定ファイル: {}",
        config::Config::config_path()?.display()
    );

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

    config.save()?;

    println!(
        "{}",
        "✅ B2 設定を保存しました".green().bold()
    );
    println!("  Bucket: {}", bucket.cyan());
    println!();
    println!("{}", "💡 認証情報は環境変数で設定することを推奨します:".yellow());
    println!("  export B2_APPLICATION_KEY_ID=<your-key-id>");
    println!("  export B2_APPLICATION_KEY=<your-key>");

    Ok(())
}
