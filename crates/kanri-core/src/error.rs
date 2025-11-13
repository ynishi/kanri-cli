use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Failed to walk directory: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("B2 error: {0}")]
    B2(String),

    #[error("Archive error: {0}")]
    Archive(String),
}

pub type Result<T> = std::result::Result<T, Error>;
