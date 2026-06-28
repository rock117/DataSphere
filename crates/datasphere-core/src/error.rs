use thiserror::Error;

pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("data source not found: {0}")]
    DataSourceNotFound(String),

    #[error("invalid task type: {0}")]
    InvalidTaskType(String),

    #[error("invalid params: {0}")]
    InvalidParams(String),

    #[error("invalid cron expression: {0}")]
    InvalidCron(String),

    #[error("data source error: {0}")]
    DataSource(String),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    ChronoParse(#[from] chrono::ParseError),
}
