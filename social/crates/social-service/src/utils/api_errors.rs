use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("User not found")]
    UserNotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Internal server error: {0}")]
    InternalServerError(String),

    #[error("User already followed")]
    UserAlreadyFollowed,

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Teloxide error: {0}")]
    TeloxideError(#[from] teloxide::RequestError),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl ApiError {
    pub fn code(&self) -> StatusCode {
        match self {
            ApiError::UserNotFound => StatusCode::NOT_FOUND,
            ApiError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::RequestError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::UserAlreadyFollowed => StatusCode::CONFLICT,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::RedisError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::TeloxideError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.code(), self.to_string()).into_response()
    }
}
