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
}

pub type Result<T> = std::result::Result<T, Error>;
