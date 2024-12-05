use serde::Serialize;
use utoipa::ToSchema;

/// The API error response structure
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorPayload {
    /// The error message
    pub message: String,
    /// The HTTP status code
    pub code: u16,
    /// The error type identifier
    pub r#type: String,
    /// Additional error details (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}
