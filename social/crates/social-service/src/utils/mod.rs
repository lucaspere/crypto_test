use api_errors::ApiError;

pub mod api_errors;
pub mod math;
pub mod redis_keys;
pub mod serde_utils;
pub mod time;

/// The API error response.
#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct ErrorResponse {
    /// The error message.
    message: String,
    /// The error code.
    code: Option<u16>,
}

impl From<ApiError> for ErrorResponse {
    fn from(err: ApiError) -> Self {
        ErrorResponse {
            message: err.to_string(),
            code: Some(err.code().as_u16()),
        }
    }
}
