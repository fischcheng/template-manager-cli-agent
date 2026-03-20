use std::io;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TmError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid agent `{0}`")]
    InvalidAgent(String),
    #[error("invalid manifest: {0}")]
    InvalidManifest(String),
    #[error("manifest missing agent `{0}`")]
    MissingAgent(String),
    #[error("home directory is unavailable")]
    HomeDirectoryUnavailable,
    #[error("check failed: {0}")]
    CheckFailed(String),
    #[error("invalid json merge target at {path}: {message}")]
    InvalidJsonMerge { path: PathBuf, message: String },
}

impl TmError {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::CheckFailed(_) => 2,
            _ => 1,
        }
    }
}
