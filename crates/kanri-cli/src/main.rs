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
