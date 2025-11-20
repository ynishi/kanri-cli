use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Backend error: {0}")]
    Backend(String),

    #[error("Exporter error: {0}")]
    Exporter(String),

    #[error("Expertise not found: {0}")]
    NotFound(String),

    #[error("Invalid scope: {0}")]
    InvalidScope(String),
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Yaml(err.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Yaml(err.to_string())
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::Yaml(err.to_string())
    }
}
