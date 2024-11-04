use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, FromRow, Serialize, Deserialize, Default)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub telegram_id: i64,
    pub selected_wallet_id: Option<Uuid>,
    // pub accepted_tos: Option<DateTime<Utc>>,
    pub waitlisted: bool,
    // pub accepted_insights_risk: Option<DateTime<Utc>>,
    // pub created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub telegram_id: i64,
    pub bio: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username.clone(),
            telegram_id: user.telegram_id,
            bio: None,
            name: Some(user.username),
            avatar_url: None,
        }
    }
}

impl IntoResponse for UserResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
