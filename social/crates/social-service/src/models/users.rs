use axum::{http::StatusCode, response::IntoResponse, Json};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub telegram_id: i64,
    pub selected_wallet_id: Option<Uuid>,
    pub accepted_tos: Option<NaiveDateTime>,
    pub waitlisted: bool,
    pub accepted_insights_risk: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub telegram_id: i64,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        UserResponse {
            id: user.id,
            username: user.username,
            telegram_id: user.telegram_id,
        }
    }
}

impl IntoResponse for UserResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
