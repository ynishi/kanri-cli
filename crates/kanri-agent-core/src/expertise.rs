use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Expertise: AI エージェントに与える専門性・知識・能力の最小単位
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Expertise {
    /// 一意識別子
    pub id: String,

    /// バージョン（セマンティックバージョニング）
    pub version: String,

    /// 表示名
    pub name: String,

    /// 説明
    pub description: String,

    /// メタデータ
    pub metadata: ExpertiseMetadata,

    /// 発動条件
    pub activation: Activation,

    /// 継承設定（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inheritance: Option<Inheritance>,

    /// パラメータ（汎用メタデータ）
    /// LLMが読むため、型情報は不要。シンプルな key-value で十分。
    #[serde(default)]
    pub parameters: HashMap<String, String>,

    /// 知識フレームワーク（In-to-Out変換ユニット）
    pub body: KnowledgeFramework,
}

impl Expertise {
    pub fn new(id: String, name: String, scope: Scope) -> Self {
        Self {
            id: id.clone(),
            version: "0.1.0".to_string(),
            name: name.clone(),
            description: String::new(),
            metadata: ExpertiseMetadata {
                scope,
                visibility: Visibility::Private,
                groups: Vec::new(),
                author: "user".to_string(),
                created: Utc::now(),
                updated: Utc::now(),
                tags: Vec::new(),
            },
            activation: Activation {
                mode: ActivationMode::default(),
                triggers: Vec::new(),
                auto: false,
                contexts: vec![scope],
            },
            inheritance: None,
            parameters: HashMap::new(),
            body: KnowledgeFramework {
                input_requirements: Vec::new(),
                output_schema: OutputSchema {
                    description: "The output format for this expertise".to_string(),
                    sections: Vec::new(),
                },
                dependencies: Vec::new(),
                synthesis_logic: format!("# {}\n\nAdd your synthesis logic here...", name),
            },
        }
    }

    /// YAML としてシリアライズ
    pub fn to_yaml(&self) -> crate::Result<String> {
        serde_yaml::to_string(self).map_err(|e| crate::Error::Yaml(e.to_string()))
    }

    /// YAML からデシリアライズ
    pub fn from_yaml(yaml: &str) -> crate::Result<Self> {
        serde_yaml::from_str(yaml).map_err(|e| crate::Error::Yaml(e.to_string()))
    }

    /// Prefix 付きの ID を取得
    pub fn prefixed_id(&self) -> String {
        format!("{}:{}", self.metadata.scope.as_str(), self.id)
    }

    /// グループを設定（階層的に展開）
    /// 例: "backend/api/integration" → ["backend", "backend/api", "backend/api/integration"]
    pub fn set_hierarchical_groups(&mut self, group_path: &str) {
        if group_path.is_empty() {
            return;
        }

        let parts: Vec<&str> = group_path.split('/').collect();
        let mut accumulated = Vec::new();

        for (i, part) in parts.iter().enumerate() {
            if i == 0 {
                accumulated.push(part.to_string());
            } else {
                accumulated.push(format!("{}/{}", accumulated.last().unwrap(), part));
            }
        }

        self.metadata.groups = accumulated;
    }

    /// 特定のグループに属しているかチェック（階層的）
    pub fn belongs_to_group(&self, group: &str) -> bool {
        self.metadata
            .groups
            .iter()
            .any(|g| g == group || g.starts_with(&format!("{}/", group)))
    }

    /// 最も詳細なグループを取得
    pub fn primary_group(&self) -> Option<&str> {
        self.metadata.groups.last().map(|s| s.as_str())
    }
}

/// メタデータ
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExpertiseMetadata {
    /// スコープ
    pub scope: Scope,

    /// 可視性
    pub visibility: Visibility,

    /// グループ（階層的、例: ["backend", "backend/api", "backend/api/integration"]）
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<String>,

    /// 作成者
    pub author: String,

    /// 作成日時
    pub created: DateTime<Utc>,

    /// 更新日時
    pub updated: DateTime<Utc>,

    /// タグ
    #[serde(default)]
    pub tags: Vec<String>,
}

/// スコープ
///
/// Expertiseの管理範囲を定義します。
/// - `Personal`: 個人用（ユーザー固有の知識・実験的パターン）
/// - `Company`: チーム/組織用（共有されるベストプラクティス）
/// - `Project`: プロジェクト固有（特定プロジェクトに依存する知識）
/// - `Admin`: 公式/管理用（kanri-agent公式の汎用的なドキュメント）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Personal,
    Company,
    Project,
    Admin,
}

impl Scope {
    /// 全てのScopeを返すイテレータ
    ///
    /// # Example
    /// ```
    /// use kanri_agent_core::Scope;
    ///
    /// for scope in Scope::all() {
    ///     println!("{}", scope.as_str());
    /// }
    /// ```
    pub fn all() -> impl Iterator<Item = Scope> {
        [
            Scope::Personal,
            Scope::Company,
            Scope::Project,
            Scope::Admin,
        ]
        .into_iter()
    }

    /// Scopeを文字列に変換
    pub fn as_str(&self) -> &'static str {
        match self {
            Scope::Personal => "personal",
            Scope::Company => "company",
            Scope::Project => "project",
            Scope::Admin => "admin",
        }
    }

    /// 文字列からScopeをパース
    ///
    /// # Errors
    /// 無効なscope文字列の場合はエラーを返します
    pub fn from_str(s: &str) -> crate::Result<Self> {
        match s.to_lowercase().as_str() {
            "personal" => Ok(Scope::Personal),
            "company" => Ok(Scope::Company),
            "project" => Ok(Scope::Project),
            "admin" => Ok(Scope::Admin),
            _ => Err(crate::Error::InvalidScope(s.to_string())),
        }
    }
}

/// 可視性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Private,
    Team,
    Public,
}

/// 発動モード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ActivationMode {
    /// ユーザーが明示的に呼ぶ（/command）
    #[default]
    Command,
    /// エージェントが自律的に呼ぶ（skill）
    Skill,
    /// 両方
    Both,
}


impl ActivationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActivationMode::Command => "command",
            ActivationMode::Skill => "skill",
            ActivationMode::Both => "both",
        }
    }
}

/// 発動条件
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Activation {
    /// 発動モード
    #[serde(default)]
    pub mode: ActivationMode,

    /// トリガーワード
    #[serde(default)]
    pub triggers: Vec<String>,

    /// 自動有効化
    #[serde(default)]
    pub auto: bool,

    /// 有効なコンテキスト（Scope）
    #[serde(default)]
    pub contexts: Vec<Scope>,
}

/// 継承設定
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Inheritance {
    /// 継承元 Expertise ID
    pub extends: String,

    /// 上書きするセクション
    #[serde(default)]
    pub overrides: Vec<String>,
}

/// 知識フレームワーク（In-to-Out変換の定義）
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[derive(Default)]
pub struct KnowledgeFramework {
    /// インプット要件（この処理に必要な情報）
    /// 基本的に UserIntent（会話履歴・ユーザー入力）から取得するデータを定義。
    /// 他のSkillから取得するデータは `dependencies` で表現。
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub input_requirements: Vec<InputRequirement>,

    /// アウトプットスキーマ（生成すべき構造）
    pub output_schema: OutputSchema,

    /// 依存する他のExpertise（知識の部品）
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<String>,

    /// 核心ロジック（インプットと依存先の成果物をどう統合・加工するか）
    pub synthesis_logic: String,
}


/// インプット要件
/// 会話履歴やユーザー入力から抽出すべき情報を定義。
/// Claude Code Skill export 時に「コンテキスト抽出セクション」として展開される。
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InputRequirement {
    /// 名前
    pub name: String,

    /// 説明（LLMが読むため、自然言語で詳細に記述）
    pub description: String,

    /// フォーマット（例: "markdown", "json", "text"）
    pub format: String,
}

/// アウトプットスキーマ
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[derive(Default)]
pub struct OutputSchema {
    /// 説明
    pub description: String,

    /// セクション定義
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<SectionDefinition>,
}


/// セクション定義
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SectionDefinition {
    /// タイトル
    pub title: String,

    /// 説明
    pub description: String,

    /// 必須ポイント
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required_points: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expertise_creation() {
        let exp = Expertise::new(
            "test-skill".to_string(),
            "Test Skill".to_string(),
            Scope::Personal,
        );

        assert_eq!(exp.id, "test-skill");
        assert_eq!(exp.name, "Test Skill");
        assert_eq!(exp.version, "0.1.0");
        assert_eq!(exp.metadata.scope, Scope::Personal);
    }

    #[test]
    fn test_expertise_yaml_roundtrip() {
        let exp = Expertise::new(
            "test-skill".to_string(),
            "Test Skill".to_string(),
            Scope::Personal,
        );

        let yaml = exp.to_yaml().unwrap();
        let parsed = Expertise::from_yaml(&yaml).unwrap();

        assert_eq!(exp.id, parsed.id);
        assert_eq!(exp.name, parsed.name);
        assert_eq!(exp.metadata.scope, parsed.metadata.scope);
    }

    #[test]
    fn test_prefixed_id() {
        let exp = Expertise::new(
            "my-skill".to_string(),
            "My Skill".to_string(),
            Scope::Company,
        );

        assert_eq!(exp.prefixed_id(), "company:my-skill");
    }

    #[test]
    fn test_scope_from_str() {
        assert_eq!(Scope::from_str("personal").unwrap(), Scope::Personal);
        assert_eq!(Scope::from_str("company").unwrap(), Scope::Company);
        assert_eq!(Scope::from_str("project").unwrap(), Scope::Project);
        assert!(Scope::from_str("invalid").is_err());
    }

    #[test]
    fn test_set_hierarchical_groups() {
        let mut exp = Expertise::new("test".to_string(), "Test".to_string(), Scope::Personal);

        exp.set_hierarchical_groups("backend/api/integration");

        assert_eq!(exp.metadata.groups.len(), 3);
        assert_eq!(exp.metadata.groups[0], "backend");
        assert_eq!(exp.metadata.groups[1], "backend/api");
        assert_eq!(exp.metadata.groups[2], "backend/api/integration");
    }

    #[test]
    fn test_belongs_to_group() {
        let mut exp = Expertise::new("test".to_string(), "Test".to_string(), Scope::Company);

        exp.set_hierarchical_groups("backend/api/integration");

        assert!(exp.belongs_to_group("backend"));
        assert!(exp.belongs_to_group("backend/api"));
        assert!(exp.belongs_to_group("backend/api/integration"));
        assert!(!exp.belongs_to_group("frontend"));
    }

    #[test]
    fn test_primary_group() {
        let mut exp = Expertise::new("test".to_string(), "Test".to_string(), Scope::Personal);

        assert_eq!(exp.primary_group(), None);

        exp.set_hierarchical_groups("backend/api");
        assert_eq!(exp.primary_group(), Some("backend/api"));
    }
}
