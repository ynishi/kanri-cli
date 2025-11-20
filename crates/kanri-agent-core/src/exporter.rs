use crate::{Expertise, InputRequirement, Result};
use std::path::PathBuf;

/// 展開されたファイル
#[derive(Debug, Clone)]
pub struct DeployedFile {
    pub path: PathBuf,
    pub content: String,
}

/// Exporter trait: Expertise を特定フォーマットに変換
pub trait Exporter: Send + Sync {
    /// Expertise を展開形式に変換（複数ファイルの可能性あり）
    fn export(&self, expertise: &Expertise, use_prefix: bool) -> Result<Vec<DeployedFile>>;
}

/// ClaudeCodeCommandExporter: Claude Code commands 形式（フラットファイル）
pub struct ClaudeCodeCommandExporter {
    base_dir: PathBuf,
}

impl ClaudeCodeCommandExporter {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// デフォルトパス (.claude/commands/)
    pub fn default() -> Self {
        Self {
            base_dir: PathBuf::from(".claude/commands"),
        }
    }

    fn generate_content(expertise: &Expertise) -> String {
        let mut content = String::new();

        // Title and description
        content.push_str(&format!(
            "# {}\n\n{}\n\n",
            expertise.name, expertise.description
        ));

        // Input requirements section (if exists)
        if !expertise.body.input_requirements.is_empty() {
            content.push_str(&Self::generate_context_extraction_section(
                &expertise.body.input_requirements,
            ));
        }

        // Main synthesis logic
        content.push_str(&expertise.body.synthesis_logic);
        content.push('\n');

        content
    }

    /// input_requirements から「コンテキスト抽出セクション」を生成
    fn generate_context_extraction_section(input_requirements: &[InputRequirement]) -> String {
        let mut section = String::from("## 必要な情報（会話履歴から抽出してください）\n\n");

        for req in input_requirements {
            section.push_str(&format!("**{}**: {}\n", req.name, req.description));
        }

        section.push_str("\n> **NOTE**: 上記情報が会話履歴から明確に特定できない場合は、");
        section.push_str("AskUserQuestionツールで確認してください。\n\n");
        section.push_str("---\n\n");

        section
    }
}

impl Exporter for ClaudeCodeCommandExporter {
    fn export(&self, expertise: &Expertise, use_prefix: bool) -> Result<Vec<DeployedFile>> {
        let filename = if use_prefix {
            format!("{}.md", expertise.prefixed_id())
        } else {
            format!("{}.md", expertise.id)
        };

        let path = self.base_dir.join(&filename);
        let content = Self::generate_content(expertise);

        Ok(vec![DeployedFile { path, content }])
    }
}

/// ClaudeCodeSkillExporter: Claude Code skills 形式（ディレクトリ + SKILL.md）
pub struct ClaudeCodeSkillExporter {
    base_dir: PathBuf,
}

impl ClaudeCodeSkillExporter {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// デフォルトパス (.claude/skills/)
    pub fn default() -> Self {
        Self {
            base_dir: PathBuf::from(".claude/skills"),
        }
    }

    fn generate_content(expertise: &Expertise) -> String {
        let mut content = String::new();

        // YAML frontmatter (required by Claude Code)
        content.push_str("---\n");

        // name: {scope}-{id} (lowercase with hyphens)
        let skill_name = format!("{}-{}", expertise.metadata.scope.as_str(), expertise.id);
        content.push_str(&format!("name: {}\n", skill_name));

        // description: with groups and tags metadata
        let mut description = expertise.description.clone();

        // Add groups info if present
        if !expertise.metadata.groups.is_empty() {
            let groups_str = expertise.metadata.groups.join("/");
            description.push_str(&format!(" [Groups: {}]", groups_str));
        }

        // Add tags info if present
        if !expertise.metadata.tags.is_empty() {
            let tags_str = expertise.metadata.tags.join(", ");
            description.push_str(&format!(" [Tags: {}]", tags_str));
        }

        content.push_str(&format!("description: {}\n", description));
        content.push_str(&format!("version: {}\n", expertise.version));
        content.push_str("---\n\n");

        // Title and description
        content.push_str(&format!(
            "# {}\n\n{}\n\n",
            expertise.name, expertise.description
        ));

        // Input requirements section (if exists)
        if !expertise.body.input_requirements.is_empty() {
            content.push_str(&Self::generate_context_extraction_section(
                &expertise.body.input_requirements,
            ));
        }

        // Main synthesis logic
        content.push_str(&expertise.body.synthesis_logic);
        content.push('\n');

        content
    }

    /// input_requirements から「コンテキスト抽出セクション」を生成
    fn generate_context_extraction_section(input_requirements: &[InputRequirement]) -> String {
        let mut section = String::from("## 必要な情報（会話履歴から抽出してください）\n\n");

        for req in input_requirements {
            section.push_str(&format!("**{}**: {}\n", req.name, req.description));
        }

        section.push_str("\n> **NOTE**: 上記情報が会話履歴から明確に特定できない場合は、");
        section.push_str("AskUserQuestionツールで確認してください。\n\n");
        section.push_str("---\n\n");

        section
    }
}

impl Exporter for ClaudeCodeSkillExporter {
    fn export(&self, expertise: &Expertise, use_prefix: bool) -> Result<Vec<DeployedFile>> {
        let dirname = if use_prefix {
            // Replace colon with hyphen for Claude Code compatibility
            expertise.prefixed_id().replace(":", "-")
        } else {
            expertise.id.clone()
        };

        let path = self.base_dir.join(dirname).join("SKILL.md");
        let content = Self::generate_content(expertise);

        Ok(vec![DeployedFile { path, content }])
    }
}

/// ClaudeCodeExporter: 後方互換性のため残す（Deprecated）
/// activation.mode に応じて Command または Skill として展開
#[deprecated(note = "Use ClaudeCodeCommandExporter or ClaudeCodeSkillExporter directly")]
pub struct ClaudeCodeExporter {
    command_exporter: ClaudeCodeCommandExporter,
    skill_exporter: ClaudeCodeSkillExporter,
}

impl ClaudeCodeExporter {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            command_exporter: ClaudeCodeCommandExporter::new(base_dir.join("commands")),
            skill_exporter: ClaudeCodeSkillExporter::new(base_dir.join("skills")),
        }
    }

    pub fn default() -> Self {
        Self::new(PathBuf::from(".claude"))
    }
}

impl Exporter for ClaudeCodeExporter {
    fn export(&self, expertise: &Expertise, use_prefix: bool) -> Result<Vec<DeployedFile>> {
        use crate::ActivationMode;

        match expertise.activation.mode {
            ActivationMode::Command => self.command_exporter.export(expertise, use_prefix),
            ActivationMode::Skill => self.skill_exporter.export(expertise, use_prefix),
            ActivationMode::Both => {
                let mut files = Vec::new();
                files.extend(self.command_exporter.export(expertise, use_prefix)?);
                files.extend(self.skill_exporter.export(expertise, use_prefix)?);
                Ok(files)
            }
        }
    }
}

/// CursorExporter: Cursor (.cursor/rules/) 形式
pub struct CursorExporter {
    target_dir: PathBuf,
}

impl CursorExporter {
    pub fn new(target_dir: PathBuf) -> Self {
        Self { target_dir }
    }

    /// デフォルトパス (.cursor/rules/)
    pub fn default() -> Self {
        Self {
            target_dir: PathBuf::from(".cursor/rules"),
        }
    }
}

impl Exporter for CursorExporter {
    fn export(&self, expertise: &Expertise, use_prefix: bool) -> Result<Vec<DeployedFile>> {
        let filename = if use_prefix {
            format!("{}_{}.md", expertise.metadata.scope.as_str(), expertise.id)
        } else {
            format!("{}.md", expertise.id)
        };

        let path = self.target_dir.join(&filename);

        // Cursor ルール形式でコンテンツを生成
        // YAML フロントマターを追加
        let frontmatter = format!(
            "---\nscope: {}\nname: {}\nmode: {}\n---\n\n",
            expertise.metadata.scope.as_str(),
            expertise.name,
            expertise.activation.mode.as_str()
        );

        let content = format!(
            "{}# {}\n\n{}\n\n{}\n",
            frontmatter, expertise.name, expertise.description, expertise.body.synthesis_logic
        );

        Ok(vec![DeployedFile { path, content }])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Scope;

    #[test]
    fn test_claude_code_command_exporter() {
        use crate::ActivationMode;

        let exporter = ClaudeCodeCommandExporter::default();
        let mut exp = Expertise::new(
            "my-command".to_string(),
            "My Command".to_string(),
            Scope::Personal,
        );
        exp.activation.mode = ActivationMode::Command;

        let deployed = exporter.export(&exp, true).unwrap();
        assert_eq!(deployed.len(), 1);
        assert!(deployed[0].path.to_string_lossy().contains("commands"));
        assert!(deployed[0]
            .path
            .to_string_lossy()
            .contains("personal:my-command.md"));
        assert!(deployed[0].content.contains("# My Command"));
    }

    #[test]
    fn test_claude_code_skill_exporter() {
        use crate::ActivationMode;

        let exporter = ClaudeCodeSkillExporter::default();
        let mut exp = Expertise::new(
            "auto-skill".to_string(),
            "Auto Skill".to_string(),
            Scope::Company,
        );
        exp.activation.mode = ActivationMode::Skill;

        let deployed = exporter.export(&exp, true).unwrap();
        assert_eq!(deployed.len(), 1);
        assert!(deployed[0].path.to_string_lossy().contains("skills"));
        // Colon is replaced with hyphen for Claude Code compatibility
        assert!(deployed[0]
            .path
            .to_string_lossy()
            .contains("company-auto-skill"));
        assert!(deployed[0].path.to_string_lossy().contains("SKILL.md"));
        assert!(deployed[0].content.contains("# Auto Skill"));
    }

    #[test]
    fn test_claude_code_exporter_both() {
        use crate::ActivationMode;

        let exporter = ClaudeCodeExporter::default();
        let mut exp = Expertise::new("hybrid".to_string(), "Hybrid".to_string(), Scope::Personal);
        exp.activation.mode = ActivationMode::Both;

        let deployed = exporter.export(&exp, true).unwrap();
        assert_eq!(deployed.len(), 2);
        assert!(deployed[0].path.to_string_lossy().contains("commands"));
        assert!(deployed[1].path.to_string_lossy().contains("skills"));
        assert!(deployed[1].path.to_string_lossy().contains("SKILL.md"));
    }

    #[test]
    fn test_cursor_exporter() {
        let exporter = CursorExporter::default();
        let exp = Expertise::new(
            "api-guide".to_string(),
            "API Guide".to_string(),
            Scope::Company,
        );

        let deployed = exporter.export(&exp, true).unwrap();
        assert_eq!(deployed.len(), 1);
        assert!(deployed[0]
            .path
            .to_string_lossy()
            .contains("company_api-guide.md"));
        assert!(deployed[0].content.contains("---"));
        assert!(deployed[0].content.contains("scope: company"));
        assert!(deployed[0].content.contains("# API Guide"));
    }

    #[test]
    fn test_context_extraction_section_generation() {
        use crate::ActivationMode;

        let exporter = ClaudeCodeSkillExporter::default();
        let mut exp = Expertise::new(
            "api-integration-guide".to_string(),
            "API Integration Guide".to_string(),
            Scope::Company,
        );
        exp.activation.mode = ActivationMode::Skill;

        // Add input requirements
        exp.body.input_requirements = vec![
            InputRequirement {
                name: "service_name".to_string(),
                description: "Service name (e.g., GitHub, Slack, CommonSaaS)".to_string(),
                format: "text".to_string(),
            },
            InputRequirement {
                name: "task_type".to_string(),
                description: "実装・バグ修正・レビューのいずれか".to_string(),
                format: "text".to_string(),
            },
        ];

        let deployed = exporter.export(&exp, false).unwrap();
        assert_eq!(deployed.len(), 1);

        let content = &deployed[0].content;
        assert!(content.contains("## 必要な情報（会話履歴から抽出してください）"));
        assert!(content.contains("**service_name**: Service name"));
        assert!(content.contains("**task_type**: 実装・バグ修正・レビューのいずれか"));
        assert!(content.contains("AskUserQuestionツールで確認してください"));
        assert!(content.contains("---"));
    }

    #[test]
    fn test_no_context_extraction_when_no_input_requirements() {
        use crate::ActivationMode;

        let exporter = ClaudeCodeSkillExporter::default();
        let mut exp = Expertise::new(
            "simple-skill".to_string(),
            "Simple Skill".to_string(),
            Scope::Personal,
        );
        exp.activation.mode = ActivationMode::Skill;
        // input_requirements は空

        let deployed = exporter.export(&exp, false).unwrap();
        let content = &deployed[0].content;

        // コンテキスト抽出セクションが生成されていないことを確認
        assert!(!content.contains("## 必要な情報"));
        assert!(!content.contains("会話履歴から抽出"));
    }
}
