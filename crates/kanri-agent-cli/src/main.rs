use anyhow::Result;
use chrono::Local;
use clap::{Parser, Subcommand};
use colored::*;
use kanri_agent_core::{
    Backend, ClaudeCodeExporter, Config, Expertise, Exporter, LocalBackend, Scope, Version,
};
use std::fs;
use std::path::PathBuf;

mod builtin;

#[derive(Parser)]
#[command(name = "kanri-agent")]
#[command(author, version, about = "AI Expertise Management Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 初期設定
    Init,

    /// ビルトイン Expertise をセットアップ
    SetupBuiltin,

    /// 新しい Expertise を作成
    New {
        /// Expertise ID
        id: String,

        /// ファイルから作成（.yaml, .yml, .txt, .md）
        #[arg(long)]
        file: String,

        /// Scope (personal/company/project/admin)
        #[arg(long, default_value = "personal")]
        scope: String,

        /// グループ（階層的、例: backend/api/integration）
        #[arg(long)]
        group: Option<String>,

        /// Name（表示名）- YAML以外の場合に上書き可能
        #[arg(long)]
        name: Option<String>,
    },

    /// Expertise 一覧を表示
    List {
        /// Scope でフィルタ
        #[arg(long)]
        scope: Option<String>,

        /// グループでフィルタ（階層的）
        #[arg(long)]
        group: Option<String>,

        /// タグでフィルタ
        #[arg(long)]
        tag: Option<String>,
    },

    /// Expertise を表示
    Show {
        /// Expertise ID (scope:id 形式も可)
        id: String,
    },

    /// Expertise を編集
    Edit {
        /// Expertise ID
        id: String,
    },

    /// Expertise を削除
    Delete {
        /// Expertise ID
        id: String,

        /// 確認をスキップ
        #[arg(long)]
        yes: bool,
    },

    /// 展開先にデプロイ
    Deploy {
        /// 特定の Scope のみデプロイ
        #[arg(long)]
        scope: Option<String>,

        /// 特定のターゲットのみ
        #[arg(long)]
        target: Option<String>,

        /// 特定の Expertise のみデプロイ (scope:id 形式)
        #[arg(long)]
        id: Option<String>,

        /// 特定バージョンをデプロイ
        #[arg(long)]
        version: Option<String>,
    },

    /// 展開状態を表示
    Status,

    /// 展開先をクリーン
    Clean {
        /// 特定のターゲットのみ
        #[arg(long)]
        target: Option<String>,
    },

    /// バージョン管理
    Version {
        /// Expertise ID
        id: String,

        #[command(subcommand)]
        action: Option<VersionAction>,
    },

    /// Expertise スキーマを表示
    Schema {
        /// 特定フィールドの説明のみ表示
        #[arg(long)]
        field: Option<String>,

        /// 出力フォーマット (yaml/json/example)
        #[arg(long, default_value = "yaml")]
        format: String,
    },

    /// Expertise をツリー表示
    Tree {
        /// Scope でフィルタ
        #[arg(long)]
        scope: Option<String>,

        /// グループでフィルタ
        #[arg(long)]
        group: Option<String>,

        /// 表示方法 (group/tag/flat)
        #[arg(long, default_value = "group")]
        by: String,
    },

    /// タグ管理
    Tag {
        /// Expertise ID
        id: String,

        #[command(subcommand)]
        action: TagAction,
    },

    /// グループ管理
    Group {
        /// Expertise ID
        id: String,

        #[command(subcommand)]
        action: GroupAction,
    },
}

#[derive(Subcommand)]
enum VersionAction {
    /// バージョン一覧を表示
    List,

    /// バージョンアップ
    Bump {
        /// バンプタイプ (major/minor/patch)
        #[arg(long, default_value = "patch")]
        level: String,
    },
}

#[derive(Subcommand)]
enum TagAction {
    /// タグを追加
    Add {
        /// 追加するタグ（複数指定可）
        tags: Vec<String>,
    },

    /// タグ一覧を表示
    List,

    /// タグを完全置換
    Set {
        /// 新しいタグリスト（複数指定可）
        tags: Vec<String>,
    },

    /// タグを削除
    Remove {
        /// 削除するタグ（複数指定可）
        tags: Vec<String>,
    },
}

#[derive(Subcommand)]
enum GroupAction {
    /// グループを追加（階層的に展開）
    Add {
        /// グループパス（例: backend/api/integration）
        group: String,
    },

    /// グループ一覧を表示
    List,

    /// グループを設定（完全置換、階層的に展開）
    Set {
        /// グループパス（例: backend/api/integration）
        group: String,
    },

    /// グループを削除
    Remove {
        /// グループパス（例: backend/api/integration）
        group: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init()?,
        Commands::SetupBuiltin => setup_builtin()?,
        Commands::New {
            id,
            file,
            scope,
            group,
            name,
        } => new_expertise(&id, &file, &scope, group, name)?,
        Commands::List { scope, group, tag } => list_expertises(scope, group, tag)?,
        Commands::Show { id } => show_expertise(&id)?,
        Commands::Edit { id } => edit_expertise(&id)?,
        Commands::Delete { id, yes } => delete_expertise(&id, yes)?,
        Commands::Deploy {
            scope,
            target,
            id,
            version,
        } => deploy(scope, target, id, version)?,
        Commands::Status => show_status()?,
        Commands::Clean { target } => clean(target)?,
        Commands::Version { id, action } => version_command(&id, action)?,
        Commands::Schema { field, format } => show_schema(field, &format)?,
        Commands::Tree { scope, group, by } => show_tree(scope, group, &by)?,
        Commands::Tag { id, action } => tag_command(&id, action)?,
        Commands::Group { id, action } => group_command(&id, action)?,
    }

    Ok(())
}

fn init() -> Result<()> {
    println!("{}", "🚀 kanri-agent を初期化中...".cyan().bold());

    let config = Config::create_default();
    config.save()?;

    let config_path = Config::config_path()?;
    println!(
        "{} 設定ファイルを作成しました: {}",
        "✅".green(),
        config_path.display()
    );

    // キャッシュディレクトリを作成
    let backend = LocalBackend::default()?;
    let personal_dir = backend.target_dir().join("personal");
    fs::create_dir_all(&personal_dir)?;

    println!("{} キャッシュディレクトリを作成しました", "✅".green());

    Ok(())
}

fn new_expertise(
    id: &str,
    file_path: &str,
    scope_str: &str,
    group: Option<String>,
    name: Option<String>,
) -> Result<()> {
    let scope = Scope::from_str(scope_str)?;

    // ファイルから作成
    let (mut expertise, format) = create_from_file(id, scope, file_path)?;

    // name が指定されていれば上書き（Plain形式の場合のみ）
    if let Some(n) = name {
        if format == "Plain" {
            expertise.name = n;
        }
    }

    // グループを設定（YAML形式の場合はファイル内の値を尊重）
    if let Some(g) = group {
        if format == "Plain" || expertise.metadata.groups.is_empty() {
            expertise.set_hierarchical_groups(&g);
        }
    }

    let backend = LocalBackend::default()?;
    backend.push(&expertise)?;

    println!(
        "{} Expertise を作成しました: {} ({}) [format: {}]",
        "✅".green(),
        expertise.prefixed_id().cyan().bold(),
        expertise.name,
        format.bright_black()
    );

    println!(
        "\n💡 編集するには: {}",
        format!("kanri-agent edit {}", id).dimmed()
    );

    Ok(())
}

fn create_from_file(id: &str, scope: Scope, file_path: &str) -> Result<(Expertise, &'static str)> {
    let content = fs::read_to_string(file_path)?;

    // YAML ファイルとしてパース
    if file_path.ends_with(".yaml") || file_path.ends_with(".yml") {
        let mut expertise = Expertise::from_yaml(&content)?;
        // ID と scope は引数で指定されたものを優先
        expertise.id = id.to_string();
        expertise.metadata.scope = scope;
        return Ok((expertise, "YAML"));
    }

    // txt/md ファイルの場合のみ String として扱う
    if file_path.ends_with(".txt") || file_path.ends_with(".md") {
        let name = std::path::Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(id)
            .to_string();

        let mut expertise = Expertise::new(id.to_string(), name, scope);
        expertise.body.synthesis_logic = content;

        return Ok((expertise, "Plain"));
    }

    // それ以外はエラー
    anyhow::bail!(
        "Unsupported file format: {}. Only .yaml, .yml, .txt, .md are supported.",
        file_path
    )
}

fn list_expertises(
    scope_filter: Option<String>,
    group_filter: Option<String>,
    tag_filter: Option<String>,
) -> Result<()> {
    let backend = LocalBackend::default()?;

    let scopes = if let Some(s) = scope_filter {
        vec![Scope::from_str(&s)?]
    } else {
        // Default: search all scopes
        Scope::all().collect::<Vec<_>>()
    };

    let mut total = 0;

    for scope in scopes {
        let mut expertises = backend.pull(scope)?;

        // グループでフィルタ
        if let Some(ref g) = group_filter {
            expertises.retain(|e| e.belongs_to_group(g));
        }

        // タグでフィルタ
        if let Some(ref t) = tag_filter {
            expertises.retain(|e| e.metadata.tags.contains(t));
        }

        if !expertises.is_empty() {
            println!("\n{}", format!("📚 {}", scope.as_str()).cyan().bold());

            for exp in expertises {
                let group_info = if let Some(g) = exp.primary_group() {
                    format!(" [{}]", g).dimmed()
                } else {
                    "".normal()
                };

                println!(
                    "  • {}{} - {}",
                    exp.prefixed_id().bright_blue(),
                    group_info,
                    exp.name.dimmed()
                );
                total += 1;
            }
        }
    }

    if total == 0 {
        println!("{}", "ℹ Expertise が見つかりませんでした".yellow());
        println!("💡 作成するには: {}", "kanri-agent new my-skill".dimmed());
    } else {
        println!("\n{} {} 件の Expertise", "✅".green(), total);
    }

    Ok(())
}

fn show_expertise(id: &str) -> Result<()> {
    let backend = LocalBackend::default()?;

    // scope:id 形式をパース
    let (scope, expertise_id) = if id.contains(':') {
        let parts: Vec<&str> = id.split(':').collect();
        (Scope::from_str(parts[0])?, parts[1])
    } else {
        // scope 指定なしの場合は全 scope を検索
        let mut found = None;
        // Search in all scopes to find the expertise
        for scope in Scope::all() {
            if let Some(exp) = backend.get(scope, id)? {
                found = Some(exp);
                break;
            }
        }

        if let Some(exp) = found {
            print_expertise(&exp);
            return Ok(());
        } else {
            anyhow::bail!("Expertise not found: {}", id);
        }
    };

    if let Some(exp) = backend.get(scope, expertise_id)? {
        print_expertise(&exp);
    } else {
        anyhow::bail!("Expertise not found: {}", id);
    }

    Ok(())
}

fn print_expertise(exp: &Expertise) {
    println!("{}", "═".repeat(60).dimmed());
    println!(
        "{} {}",
        "ID:".bright_white().bold(),
        exp.prefixed_id().cyan()
    );
    println!("{} {}", "Name:".bright_white().bold(), exp.name);
    println!("{} {}", "Version:".bright_white().bold(), exp.version);
    println!(
        "{} {}",
        "Scope:".bright_white().bold(),
        exp.metadata.scope.as_str()
    );
    println!(
        "{} {}",
        "Author:".bright_white().bold(),
        exp.metadata.author
    );
    println!("{}", "═".repeat(60).dimmed());
    println!("\n{}\n", exp.body.synthesis_logic);
}

fn edit_expertise(id: &str) -> Result<()> {
    let backend = LocalBackend::default()?;

    // 検索
    let mut found_scope = None;
    for scope in [Scope::Personal, Scope::Company, Scope::Project] {
        if (backend.get(scope, id)?).is_some() {
            found_scope = Some(scope);
            break;
        }
    }

    let scope = found_scope.ok_or_else(|| anyhow::anyhow!("Expertise not found: {}", id))?;

    // ファイルパスを構築
    let home = std::env::var("HOME")?;
    let exp_path = PathBuf::from(home)
        .join(".kanri-agent/cache")
        .join(scope.as_str())
        .join(id)
        .join("expertise.yaml");

    // エディタで開く
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    println!("{} エディタで開きます: {}", "📝".cyan(), editor.dimmed());

    std::process::Command::new(&editor)
        .arg(&exp_path)
        .status()?;

    println!("{} 編集を完了しました", "✅".green());

    Ok(())
}

fn delete_expertise(id: &str, yes: bool) -> Result<()> {
    let backend = LocalBackend::default()?;

    // 検索
    let mut found_scope = None;
    for scope in [Scope::Personal, Scope::Company, Scope::Project] {
        if (backend.get(scope, id)?).is_some() {
            found_scope = Some(scope);
            break;
        }
    }

    let scope = found_scope.ok_or_else(|| anyhow::anyhow!("Expertise not found: {}", id))?;

    if !yes {
        print!("本当に削除しますか? [y/N]: ");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "キャンセルされました".yellow());
            return Ok(());
        }
    }

    backend.delete(scope, id)?;

    println!("{} 削除しました: {}:{}", "✅".green(), scope.as_str(), id);

    Ok(())
}

fn deploy(
    scope_filter: Option<String>,
    target_filter: Option<String>,
    id_filter: Option<String>,
    version_filter: Option<String>,
) -> Result<()> {
    println!("{}", "🚀 デプロイ中...".cyan().bold());

    let backend = LocalBackend::default()?;

    // ターゲットディレクトリを決定
    let base_dir = match target_filter.as_deref() {
        Some("global") => {
            let home = std::env::var("HOME")
                .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
            PathBuf::from(home).join(".claude")
        }
        Some("local") | None => PathBuf::from(".claude"),
        Some(other) => {
            return Err(anyhow::anyhow!(
                "Invalid target: {}. Use 'local' or 'global'",
                other
            ))
        }
    };

    let exporter = ClaudeCodeExporter::new(base_dir.clone());

    println!(
        "  {} {}",
        "📁".cyan(),
        format!("ターゲット: {}", base_dir.display()).dimmed()
    );

    // 特定 ID が指定された場合
    if let Some(id_spec) = id_filter {
        let (scope, id) = if id_spec.contains(':') {
            let parts: Vec<&str> = id_spec.split(':').collect();
            (Scope::from_str(parts[0])?, parts[1])
        } else {
            // Scope を検索
            let mut found = None;
            // Search in all scopes to find the expertise
            for s in [
                Scope::Personal,
                Scope::Company,
                Scope::Project,
                Scope::Admin,
            ] {
                if backend.get(s, &id_spec)?.is_some() {
                    found = Some(s);
                    break;
                }
            }
            let s = found.ok_or_else(|| anyhow::anyhow!("Expertise not found: {}", id_spec))?;
            (s, id_spec.as_str())
        };

        // 特定バージョンを取得
        let exp = if let Some(ver) = version_filter {
            backend.get_version(scope, id, &ver)?.ok_or_else(|| {
                anyhow::anyhow!("Version not found: {}:{} v{}", scope.as_str(), id, ver)
            })?
        } else {
            backend
                .get(scope, id)?
                .ok_or_else(|| anyhow::anyhow!("Expertise not found: {}:{}", scope.as_str(), id))?
        };

        let deployed_files = exporter.export(&exp, true)?;

        for deployed in deployed_files {
            if let Some(parent) = deployed.path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(&deployed.path, &deployed.content)?;

            println!(
                "  {} {} v{} -> {}",
                "✅".green(),
                exp.prefixed_id().cyan(),
                exp.version.dimmed(),
                deployed.path.display().to_string().dimmed()
            );
        }

        println!("\n{} デプロイ完了", "✅".green());
        return Ok(());
    }

    // 全体デプロイ
    let scopes = if let Some(s) = scope_filter {
        vec![Scope::from_str(&s)?]
    } else {
        // Default: search all scopes
        Scope::all().collect::<Vec<_>>()
    };

    let mut deployed_count = 0;

    for scope in scopes {
        let expertises = backend.pull(scope)?;

        for exp in expertises {
            let deployed_files = exporter.export(&exp, true)?;

            for deployed in deployed_files {
                // ディレクトリを作成
                if let Some(parent) = deployed.path.parent() {
                    fs::create_dir_all(parent)?;
                }

                fs::write(&deployed.path, &deployed.content)?;

                println!(
                    "  {} {} -> {}",
                    "✅".green(),
                    exp.prefixed_id().cyan(),
                    deployed.path.display().to_string().dimmed()
                );

                deployed_count += 1;
            }
        }
    }

    println!(
        "\n{} {} 件のファイルを展開しました",
        "✅".green(),
        deployed_count
    );

    Ok(())
}

fn show_status() -> Result<()> {
    let backend = LocalBackend::default()?;

    println!("{}", "📊 現在の状態".cyan().bold());
    println!();

    // Display stats for all scopes
    for scope in Scope::all() {
        let expertises = backend.pull(scope)?;
        println!(
            "{}: {} 件",
            scope.as_str().bright_white(),
            expertises.len().to_string().yellow()
        );
    }

    Ok(())
}

fn clean(target_filter: Option<String>) -> Result<()> {
    println!("{}", "🧹 展開先をクリーン中...".cyan().bold());

    // ターゲットディレクトリを決定
    let is_global = matches!(target_filter.as_deref(), Some("global"));
    let base_dir = match target_filter.as_deref() {
        Some("global") => {
            let home = std::env::var("HOME")
                .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
            PathBuf::from(home).join(".claude")
        }
        Some("local") | None => PathBuf::from(".claude"),
        Some(other) => {
            return Err(anyhow::anyhow!(
                "Invalid target: {}. Use 'local' or 'global'",
                other
            ))
        }
    };

    println!(
        "  {} {}",
        "📁".cyan(),
        format!("ターゲット: {}", base_dir.display()).dimmed()
    );

    // グローバルターゲットの場合はバックアップを作成
    if is_global && base_dir.exists() {
        let timestamp = Local::now().format("%Y%m%d-%H%M%S");
        let home = std::env::var("HOME")
            .map_err(|_| anyhow::anyhow!("HOME environment variable not set"))?;
        let backup_dir = PathBuf::from(home)
            .join(".kanri-agent")
            .join("backups")
            .join(format!("claude-{}", timestamp));

        println!(
            "  {} {}",
            "💾".cyan(),
            "グローバル設定をバックアップ中...".dimmed()
        );

        // バックアップディレクトリを作成
        fs::create_dir_all(&backup_dir)?;

        // .claude/ 全体をコピー
        copy_dir_all(&base_dir, &backup_dir)?;

        println!(
            "  {} バックアップ先: {}",
            "✅".green(),
            backup_dir.display().to_string().dimmed()
        );
    }

    let dirs = vec![base_dir.join("commands"), base_dir.join("skills")];

    let mut cleaned_count = 0;

    for dir in dirs {
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
            println!("{} {} を削除しました", "✅".green(), dir.display());
            cleaned_count += 1;
        }
    }

    if cleaned_count == 0 {
        println!("{}", "ℹ クリーンする対象がありません".yellow());
    } else {
        println!(
            "\n{} {} ディレクトリをクリーンしました",
            "✅".green(),
            cleaned_count
        );
    }

    Ok(())
}

/// ディレクトリを再帰的にコピーするヘルパー関数
fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

fn show_schema(field: Option<String>, format: &str) -> Result<()> {
    use schemars::schema_for;

    match format.to_lowercase().as_str() {
        "json" => {
            // JSON Schema 形式で出力
            let schema = schema_for!(Expertise);
            let json = serde_json::to_string_pretty(&schema)?;
            println!("{}", json);
        }
        "example" => {
            // サンプルYAMLを出力
            let example = create_example_expertise();
            let yaml = example.to_yaml()?;
            println!("{}", "📋 Expertise サンプル".cyan().bold());
            println!();
            println!("{}", yaml);
        }
        "yaml" | _ => {
            // YAML形式の説明を表示
            if let Some(field_name) = field {
                show_field_description(&field_name)?;
            } else {
                show_full_schema()?;
            }
        }
    }

    Ok(())
}

fn create_example_expertise() -> Expertise {
    use kanri_agent_core::ActivationMode;

    let mut expertise = Expertise::new(
        "example-skill".to_string(),
        "Example Skill".to_string(),
        Scope::Personal,
    );

    expertise.description = "This is an example expertise".to_string();
    expertise.activation.mode = ActivationMode::Both;
    expertise.activation.triggers =
        vec!["example trigger".to_string(), "another trigger".to_string()];
    expertise.activation.auto = false;
    expertise.metadata.tags = vec!["example".to_string(), "demo".to_string()];
    expertise.set_hierarchical_groups("category/subcategory");
    expertise
        .parameters
        .insert("key".to_string(), "value".to_string());
    expertise.body.synthesis_logic =
        "# Example Content\n\nYour expertise content goes here...".to_string();

    expertise
}

fn show_full_schema() -> Result<()> {
    println!("{}", "📚 Expertise スキーマ".cyan().bold());
    println!();
    println!("{}", "主要フィールド:".bright_white().bold());
    println!();

    let fields = vec![
        ("id", "一意識別子（例: my-skill）"),
        ("name", "表示名（例: My Skill）"),
        (
            "version",
            "バージョン（セマンティックバージョニング、例: 1.0.0）",
        ),
        ("description", "説明文"),
        (
            "metadata",
            "メタデータ（scope, visibility, groups, author, tags等）",
        ),
        ("activation", "発動条件（mode, triggers, auto, contexts）"),
        ("inheritance", "継承設定（オプション、extends, overrides）"),
        ("parameters", "パラメータ（キー・バリューのマップ）"),
        ("content", "本体コンテンツ（Markdown形式）"),
    ];

    for (field, desc) in fields {
        println!("  {} {}", field.bright_blue().bold(), desc.dimmed());
    }

    println!();
    println!("{}", "詳細を見るには:".dimmed());
    println!(
        "  {} kanri-agent schema --field <フィールド名>",
        "💡".cyan()
    );
    println!();
    println!("{}", "サンプルを見るには:".dimmed());
    println!("  {} kanri-agent schema --format example", "💡".cyan());

    Ok(())
}

fn show_field_description(field: &str) -> Result<()> {
    match field.to_lowercase().as_str() {
        "metadata" => {
            println!("{}", "metadata フィールド".cyan().bold());
            println!();
            println!("Expertise のメタデータを保持します。");
            println!();
            println!("{}", "サブフィールド:".bright_white());
            println!("  {} personal/company/project", "scope:".bright_blue());
            println!("  {} private/team/public", "visibility:".bright_blue());
            println!("  {} グループの配列（階層的）", "groups:".bright_blue());
            println!("  {} 作成者", "author:".bright_blue());
            println!("  {} 作成日時", "created:".bright_blue());
            println!("  {} 更新日時", "updated:".bright_blue());
            println!("  {} タグの配列", "tags:".bright_blue());
        }
        "activation" => {
            println!("{}", "activation フィールド".cyan().bold());
            println!();
            println!("Expertise の発動条件を定義します。");
            println!();
            println!("{}", "サブフィールド:".bright_white());
            println!("  {} command/skill/both", "mode:".bright_blue());
            println!("  {} トリガーワードの配列", "triggers:".bright_blue());
            println!("  {} 自動有効化するか（true/false）", "auto:".bright_blue());
            println!(
                "  {} 有効なコンテキスト（scope配列）",
                "contexts:".bright_blue()
            );
        }
        "inheritance" => {
            println!("{}", "inheritance フィールド（オプション）".cyan().bold());
            println!();
            println!("他の Expertise を継承する場合に使用します。");
            println!();
            println!("{}", "サブフィールド:".bright_white());
            println!("  {} 継承元のExpertise ID", "extends:".bright_blue());
            println!(
                "  {} 上書きするセクションの配列",
                "overrides:".bright_blue()
            );
        }
        "groups" => {
            println!("{}", "groups フィールド（metadata内）".cyan().bold());
            println!();
            println!("階層的なグループ分類を行います。");
            println!();
            println!("{}", "例:".bright_white());
            println!("  groups:");
            println!("    - backend");
            println!("    - backend/api");
            println!("    - backend/api/integration");
            println!();
            println!("{}", "💡 --group オプションで自動展開されます:".dimmed());
            println!("  kanri-agent new my-skill --group backend/api/integration");
        }
        _ => {
            println!("{}", format!("❌ 不明なフィールド: {}", field).red());
            println!();
            println!("{}", "利用可能なフィールド:".dimmed());
            println!("  metadata, activation, inheritance, groups");
        }
    }

    Ok(())
}

fn show_tree(scope_filter: Option<String>, group_filter: Option<String>, by: &str) -> Result<()> {
    use std::collections::BTreeMap;

    let backend = LocalBackend::default()?;

    let scopes = if let Some(s) = scope_filter {
        vec![Scope::from_str(&s)?]
    } else {
        // Default: search all scopes
        Scope::all().collect::<Vec<_>>()
    };

    println!("{}", "🌳 Expertise ツリー".cyan().bold());
    println!();

    match by.to_lowercase().as_str() {
        "group" => {
            // グループ別にツリー表示
            for scope in scopes {
                let mut expertises = backend.pull(scope)?;

                // グループフィルタ
                if let Some(ref g) = group_filter {
                    expertises.retain(|e| e.belongs_to_group(g));
                }

                if expertises.is_empty() {
                    continue;
                }

                println!(
                    "{} ({} items)",
                    scope.as_str().bright_white().bold(),
                    expertises.len().to_string().yellow()
                );

                // グループ階層を構築（トップレベルのグループのみ）
                let mut group_tree: BTreeMap<String, Vec<&Expertise>> = BTreeMap::new();

                for exp in &expertises {
                    if let Some(groups) = exp.metadata.groups.first() {
                        // 最初のグループ（トップレベル）をキーにする
                        group_tree.entry(groups.to_string()).or_default().push(exp);
                    } else {
                        group_tree
                            .entry("(ungrouped)".to_string())
                            .or_default()
                            .push(exp);
                    }
                }

                // ツリー表示
                let keys: Vec<_> = group_tree.keys().cloned().collect();
                for (top_idx, top_group) in keys.iter().enumerate() {
                    let is_last_top = top_idx == keys.len() - 1;
                    let top_branch = if is_last_top { "└─" } else { "├─" };
                    let exps = &group_tree[top_group];

                    println!("{} {}/", top_branch.dimmed(), top_group.bright_white());

                    // このトップグループに属する全expertiseを階層的に整理
                    let mut sub_groups: BTreeMap<String, Vec<&Expertise>> = BTreeMap::new();

                    for exp in exps.iter() {
                        if let Some(primary) = exp.primary_group() {
                            sub_groups.entry(primary.to_string()).or_default().push(exp);
                        } else {
                            sub_groups
                                .entry(top_group.to_string())
                                .or_default()
                                .push(exp);
                        }
                    }

                    let sub_keys: Vec<_> = sub_groups.keys().cloned().collect();
                    for (sub_idx, sub_group) in sub_keys.iter().enumerate() {
                        let is_last_sub = sub_idx == sub_keys.len() - 1;
                        let sub_exps = &sub_groups[sub_group];

                        // サブグループの階層深さを計算
                        let depth = sub_group.split('/').count();
                        let indent_str = "  ".repeat(depth.saturating_sub(1));
                        let sub_branch = if is_last_sub { "  └─" } else { "  ├─" };
                        let continuation = if is_last_top { " " } else { "│" };

                        if depth > 1 {
                            let parts: Vec<&str> = sub_group.split('/').collect();
                            println!(
                                "{}{}{} {}/",
                                continuation,
                                indent_str,
                                sub_branch.dimmed(),
                                parts.last().unwrap().bright_white()
                            );
                        }

                        // Expertiseを表示
                        for (exp_idx, exp) in sub_exps.iter().enumerate() {
                            let is_last_exp = exp_idx == sub_exps.len() - 1;
                            let exp_branch = if is_last_exp { "└─" } else { "├─" };
                            let exp_indent = "  ".repeat(depth);
                            let exp_continuation = if is_last_sub { "  " } else { "│ " };
                            let parent_continuation = if is_last_top { " " } else { "│" };

                            println!(
                                "{}{}{}{} {} (v{})",
                                parent_continuation,
                                exp_indent,
                                exp_continuation.dimmed(),
                                exp_branch.dimmed(),
                                exp.prefixed_id().cyan(),
                                exp.version.dimmed()
                            );
                        }
                    }
                }

                println!();
            }
        }
        "tag" => {
            // タグ別にツリー表示
            for scope in scopes {
                let expertises = backend.pull(scope)?;

                if expertises.is_empty() {
                    continue;
                }

                println!(
                    "{} ({} items)",
                    scope.as_str().bright_white().bold(),
                    expertises.len().to_string().yellow()
                );

                // タグごとにグループ化
                let mut tag_map: BTreeMap<String, Vec<&Expertise>> = BTreeMap::new();

                for exp in &expertises {
                    if exp.metadata.tags.is_empty() {
                        tag_map
                            .entry("(untagged)".to_string())
                            .or_default()
                            .push(exp);
                    } else {
                        for tag in &exp.metadata.tags {
                            tag_map.entry(tag.clone()).or_default().push(exp);
                        }
                    }
                }

                // ツリー表示
                let keys: Vec<_> = tag_map.keys().cloned().collect();
                for (idx, tag) in keys.iter().enumerate() {
                    let is_last = idx == keys.len() - 1;
                    let branch = if is_last { "└─" } else { "├─" };
                    let exps = &tag_map[tag];

                    println!(
                        "{} {} ({} items)",
                        branch.dimmed(),
                        tag.bright_white(),
                        exps.len()
                    );

                    for (exp_idx, exp) in exps.iter().enumerate() {
                        let is_last_exp = exp_idx == exps.len() - 1;
                        let exp_branch = if is_last_exp { "└─" } else { "├─" };
                        let child_indent = if is_last { "  " } else { "│ " };

                        println!(
                            "  {}{} {} (v{})",
                            child_indent.dimmed(),
                            exp_branch.dimmed(),
                            exp.prefixed_id().cyan(),
                            exp.version.dimmed()
                        );
                    }
                }

                println!();
            }
        }
        "flat" | _ => {
            // フラット表示
            for scope in scopes {
                let mut expertises = backend.pull(scope)?;

                // グループフィルタ
                if let Some(ref g) = group_filter {
                    expertises.retain(|e| e.belongs_to_group(g));
                }

                if expertises.is_empty() {
                    continue;
                }

                println!(
                    "{} ({} items)",
                    scope.as_str().bright_white().bold(),
                    expertises.len().to_string().yellow()
                );

                for (idx, exp) in expertises.iter().enumerate() {
                    let is_last = idx == expertises.len() - 1;
                    let branch = if is_last { "└─" } else { "├─" };

                    let group_info = if let Some(g) = exp.primary_group() {
                        format!(" [{}]", g).dimmed()
                    } else {
                        "".normal()
                    };

                    println!(
                        "{} {}{} (v{})",
                        branch.dimmed(),
                        exp.prefixed_id().cyan(),
                        group_info,
                        exp.version.dimmed()
                    );
                }

                println!();
            }
        }
    }

    Ok(())
}

fn tag_command(id: &str, action: TagAction) -> Result<()> {
    let backend = LocalBackend::default()?;

    // Expertise を検索
    let (scope, mut exp) = find_expertise(&backend, id)?;

    match action {
        TagAction::Add { tags } => {
            for tag in &tags {
                if !exp.metadata.tags.contains(tag) {
                    exp.metadata.tags.push(tag.clone());
                }
            }

            exp.metadata.updated = chrono::Utc::now();
            backend.push(&exp)?;

            println!(
                "{} {} にタグを追加しました: {}",
                "✅".green(),
                exp.prefixed_id().cyan().bold(),
                tags.join(", ").bright_white()
            );
        }
        TagAction::List => {
            println!(
                "{} {} のタグ:",
                "🏷️".cyan(),
                exp.prefixed_id().cyan().bold()
            );

            if exp.metadata.tags.is_empty() {
                println!("{}", "  (タグなし)".dimmed());
            } else {
                for tag in &exp.metadata.tags {
                    println!("  • {}", tag.bright_white());
                }
            }
        }
        TagAction::Set { tags } => {
            exp.metadata.tags = tags.clone();
            exp.metadata.updated = chrono::Utc::now();
            backend.push(&exp)?;

            println!(
                "{} {} のタグを設定しました: {}",
                "✅".green(),
                exp.prefixed_id().cyan().bold(),
                tags.join(", ").bright_white()
            );
        }
        TagAction::Remove { tags } => {
            exp.metadata.tags.retain(|t| !tags.contains(t));
            exp.metadata.updated = chrono::Utc::now();
            backend.push(&exp)?;

            println!(
                "{} {} からタグを削除しました: {}",
                "✅".green(),
                exp.prefixed_id().cyan().bold(),
                tags.join(", ").bright_white()
            );
        }
    }

    Ok(())
}

fn group_command(id: &str, action: GroupAction) -> Result<()> {
    let backend = LocalBackend::default()?;

    // Expertise を検索
    let (scope, mut exp) = find_expertise(&backend, id)?;

    match action {
        GroupAction::Add { group } => {
            // 階層的に展開して追加
            let groups = expand_hierarchical_groups(&group);
            for g in &groups {
                if !exp.metadata.groups.contains(g) {
                    exp.metadata.groups.push(g.clone());
                }
            }

            exp.metadata.updated = chrono::Utc::now();
            backend.push(&exp)?;

            println!(
                "{} {} にグループを追加しました: {}",
                "✅".green(),
                exp.prefixed_id().cyan().bold(),
                group.bright_white()
            );
            println!("  展開されたグループ: {}", groups.join(", ").dimmed());
        }
        GroupAction::List => {
            println!(
                "{} {} のグループ:",
                "📁".cyan(),
                exp.prefixed_id().cyan().bold()
            );

            if exp.metadata.groups.is_empty() {
                println!("{}", "  (グループなし)".dimmed());
            } else {
                for group in &exp.metadata.groups {
                    println!("  • {}", group.bright_white());
                }
            }
        }
        GroupAction::Set { group } => {
            // 階層的に展開して完全置換
            exp.set_hierarchical_groups(&group);
            exp.metadata.updated = chrono::Utc::now();
            backend.push(&exp)?;

            println!(
                "{} {} のグループを設定しました: {}",
                "✅".green(),
                exp.prefixed_id().cyan().bold(),
                group.bright_white()
            );
            println!(
                "  展開されたグループ: {}",
                exp.metadata.groups.join(", ").dimmed()
            );
        }
        GroupAction::Remove { group } => {
            // 階層的に展開して削除
            let groups = expand_hierarchical_groups(&group);
            exp.metadata.groups.retain(|g| !groups.contains(g));
            exp.metadata.updated = chrono::Utc::now();
            backend.push(&exp)?;

            println!(
                "{} {} からグループを削除しました: {}",
                "✅".green(),
                exp.prefixed_id().cyan().bold(),
                group.bright_white()
            );
        }
    }

    Ok(())
}

/// Search for an expertise by ID across all scopes
fn find_expertise(backend: &LocalBackend, id: &str) -> Result<(Scope, Expertise)> {
    for scope in Scope::all() {
        if let Some(exp) = backend.get(scope, id)? {
            return Ok((scope, exp));
        }
    }

    anyhow::bail!("Expertise not found: {}", id)
}

fn expand_hierarchical_groups(group: &str) -> Vec<String> {
    let parts: Vec<&str> = group.split('/').collect();
    let mut groups = Vec::new();

    for i in 1..=parts.len() {
        groups.push(parts[..i].join("/"));
    }

    groups
}

fn version_command(id: &str, action: Option<VersionAction>) -> Result<()> {
    let backend = LocalBackend::default()?;

    // Expertise を検索
    let mut found_scope = None;
    let mut expertise = None;

    for scope in [Scope::Personal, Scope::Company, Scope::Project] {
        if let Some(exp) = backend.get(scope, id)? {
            found_scope = Some(scope);
            expertise = Some(exp);
            break;
        }
    }

    let scope = found_scope.ok_or_else(|| anyhow::anyhow!("Expertise not found: {}", id))?;
    let mut exp = expertise.unwrap();

    match action {
        None | Some(VersionAction::List) => {
            // バージョン一覧を表示
            println!(
                "{} {} のバージョン履歴",
                "📜".cyan(),
                exp.prefixed_id().cyan().bold()
            );
            println!();

            let versions = backend.list_versions(scope, id)?;

            if versions.is_empty() {
                println!("{}", "ℹ バージョン履歴がありません".yellow());
            } else {
                for ver in &versions {
                    let marker = if ver == &exp.version {
                        " (current)".green()
                    } else {
                        "".normal()
                    };
                    println!("  • v{}{}", ver.bright_white(), marker);
                }
            }

            // 現在のバージョンも表示
            if !versions.contains(&exp.version) {
                println!(
                    "  • v{} {}",
                    exp.version.bright_white(),
                    "(current)".green()
                );
            }
        }
        Some(VersionAction::Bump { level }) => {
            // バージョンアップ
            let current_version = Version::parse(&exp.version)?;
            let new_version = match level.to_lowercase().as_str() {
                "major" => current_version.bump_major(),
                "minor" => current_version.bump_minor(),
                "patch" => current_version.bump_patch(),
                _ => anyhow::bail!("Invalid bump level: {}. Use major/minor/patch", level),
            };

            exp.version = new_version.to_string();
            exp.metadata.updated = chrono::Utc::now();

            backend.push(&exp)?;

            println!(
                "{} {} のバージョンを更新: {} → {}",
                "✅".green(),
                exp.prefixed_id().cyan().bold(),
                current_version.to_string().dimmed(),
                new_version.to_string().green().bold()
            );

            println!("\n💡 展開するには: {}", "kanri-agent deploy".dimmed());
        }
    }

    Ok(())
}

/// Setup built-in admin expertises from embedded binary data
fn setup_builtin() -> Result<()> {
    println!(
        "{}",
        "🚀 ビルトイン Expertise をセットアップ中...".cyan().bold()
    );

    let backend = LocalBackend::default()?;
    let mut installed_count = 0;
    let mut updated_count = 0;

    for (filename, yaml_content) in builtin::BUILTIN_EXPERTISES {
        // Parse YAML content
        let expertise = Expertise::from_yaml(yaml_content)
            .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", filename, e))?;

        // Check if already exists
        let exists = backend.get(Scope::Admin, &expertise.id)?.is_some();

        // Push to backend (will overwrite if exists)
        backend.push(&expertise)?;

        if exists {
            updated_count += 1;
            println!(
                "  {} {} (v{}) を更新しました",
                "🔄".yellow(),
                expertise.prefixed_id().cyan().bold(),
                expertise.version.dimmed()
            );
        } else {
            installed_count += 1;
            println!(
                "  {} {} (v{}) をインストールしました",
                "✅".green(),
                expertise.prefixed_id().cyan().bold(),
                expertise.version.dimmed()
            );
        }
    }

    println!();
    if installed_count > 0 {
        println!(
            "{} {} 件の Expertise をインストールしました",
            "✅".green(),
            installed_count
        );
    }
    if updated_count > 0 {
        println!(
            "{} {} 件の Expertise を更新しました",
            "🔄".yellow(),
            updated_count
        );
    }

    println!();
    println!("{}", "次のステップ:".bright_white().bold());
    println!(
        "  1️⃣ 確認: {}",
        "kanri-agent list --scope admin".dimmed()
    );
    println!(
        "  2️⃣ デプロイ: {}",
        "kanri-agent deploy --scope admin --target global".dimmed()
    );

    Ok(())
}
