use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("unsupported protocol schema version: {0}")]
    UnsupportedSchemaVersion(i64),
    #[error("invalid event: {0}")]
    InvalidEvent(String),
    #[error("credential missing for provider {0}")]
    CredentialMissing(String),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("operation cancelled")]
    Cancelled,
    #[error(transparent)]
    Database(#[from] rusqlite::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type CoreResult<T> = Result<T, CoreError>;
