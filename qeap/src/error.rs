use std::{io, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Init(#[from] InitError),
    #[error("Failed to open {0} for reading: {1}")]
    Open(PathBuf, io::Error),
    #[error("Failed to parse {0} as JSON: {1}")]
    JsonParse(PathBuf, serde_json::Error),
    #[error("Failed to save initial state: {0}")]
    Save(#[from] SaveError),
}

#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("{0}")]
    Init(#[from] InitError),
    #[error("Failed to open {0} for writing: {1}")]
    Open(PathBuf, io::Error),
    #[error("Failed to parse {0} as JSON: {1}")]
    JsonWrite(PathBuf, serde_json::Error),
}

#[derive(Debug, thiserror::Error)]
#[error("Failed to initialize root directory: {0}")]
pub struct InitError(#[from] io::Error);
