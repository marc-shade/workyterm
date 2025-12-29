//! Error types for WorkyTerm

use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkyError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("LLM provider error: {0}")]
    Provider(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("No API key configured for {0}")]
    MissingApiKey(String),

    #[error("Task cancelled by user")]
    Cancelled,

    #[error("All workers failed: {0}")]
    AllWorkersFailed(String),
}

pub type Result<T> = std::result::Result<T, WorkyError>;
