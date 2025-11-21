pub mod adapter;
pub mod backend;
pub mod config;
pub mod error;
pub mod expertise;
pub mod exporter;
pub mod version;

pub use backend::{Backend, GitBackend, LocalBackend};
pub use config::{BackendConfig, Config, DeploymentConfig, ScopeConfig};
pub use error::{Error, Result};
pub use expertise::{
    Activation, ActivationMode, Expertise, ExpertiseMetadata, InputRequirement, KnowledgeFramework,
    OutputSchema, Scope, Visibility,
};
pub use exporter::{
    ClaudeCodeCommandExporter, ClaudeCodeExporter, ClaudeCodeSkillExporter, CursorExporter,
    DeployedFile, Exporter,
};
pub use version::Version;
