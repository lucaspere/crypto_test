use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

use super::error_payload::ErrorPayload;

/// Application error types
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("An error occurred while processing the request")]
    RequestError(#[from] reqwest::Error),

    #[error("An error occurred while accessing the database")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Internal server error")]
    InternalServerError(),

    #[error("An error occurred while accessing the cache")]
    RedisError(#[from] redis::RedisError),

    #[error("An error occurred while processing the request")]
    TeloxideError(#[from] teloxide::RequestError),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid file type")]
    InvalidFileType,

    #[error("An error occurred while accessing the storage service")]
    S3Error(String),

    #[error("An error occurred while downloading the file")]
    DownloadError(#[from] teloxide::DownloadError),

    #[error("Token pick not found")]
    TokenPickNotFound,

    /// An error to be returned when a business logic error occurs
    #[error("Business logic error: {0}")]
    BusinessLogicError(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

impl AppError {
    pub fn code(&self) -> StatusCode {
        match self {
            AppError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::InternalServerError() => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::RequestError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::TeloxideError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::InvalidFileType => StatusCode::BAD_REQUEST,
            AppError::S3Error(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::DownloadError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::TokenPickNotFound => StatusCode::NOT_FOUND,
            AppError::BusinessLogicError(_) => StatusCode::CONFLICT,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
        }
    }
    fn error_type(&self) -> String {
        match self {
            AppError::DatabaseError(_) => "DATABASE_ERROR",
            AppError::InternalServerError() => "INTERNAL_SERVER_ERROR",
            AppError::RequestError(_) => "REQUEST_ERROR",
            AppError::BadRequest(_) => "BAD_REQUEST",
            AppError::RedisError(_) => "REDIS_ERROR",
            AppError::TeloxideError(_) => "TELOXIDE_ERROR",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::InvalidFileType => "INVALID_FILE_TYPE",
            AppError::S3Error(_) => "S3_ERROR",
            AppError::DownloadError(_) => "DOWNLOAD_ERROR",
            AppError::TokenPickNotFound => "TOKEN_PICK_NOT_FOUND",
            AppError::BusinessLogicError(_) => "BUSINESS_LOGIC_ERROR",
            AppError::Unauthorized(_) => "UNAUTHORIZED",
        }
        .to_string()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.code();
        let error_response = ErrorPayload {
            message: self.to_string(),
            code: status.as_u16(),
            r#type: self.error_type(),
            details: None,
        };

        (status, Json(error_response)).into_response()
    }
}
