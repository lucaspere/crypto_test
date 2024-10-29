use api_errors::ApiError;

pub mod api_errors;

// Define a custom error type
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    message: String,
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
