use axum::http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("User not found")]
    UserNotFound,

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Internal server error: {0}")]
    InternalServerError(String),
}

impl ApiError {
    pub fn code(&self) -> StatusCode {
        match self {
            ApiError::UserNotFound => StatusCode::NOT_FOUND,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
