use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("User not found")]
    UserNotFound,

    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),

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
            ApiError::RequestError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.code(), self.to_string()).into_response()
    }
}
