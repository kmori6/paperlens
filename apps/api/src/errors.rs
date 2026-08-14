use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("paper source error: {0}")]
    PaperSource(String),
    #[error("LLM error: {0}")]
    Llm(String),
    #[error("storage error: {0}")]
    Storage(String),
    #[error("parse error: {0}")]
    Parse(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AppError::Configuration(_) | AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::PaperSource(_) => StatusCode::BAD_GATEWAY,
            AppError::Llm(_) => StatusCode::BAD_GATEWAY,
            AppError::Storage(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Parse(_) => StatusCode::UNPROCESSABLE_ENTITY,
        };

        let payload = json!({ "error": self.to_string() });
        (status, Json(payload)).into_response()
    }
}
