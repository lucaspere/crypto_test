use crate::models::users::User;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct UserRepository {
    db: Arc<PgPool>,
}

impl UserRepository {
    pub fn new(db: Arc<PgPool>) -> Self {
        UserRepository { db }
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(self.db.as_ref())
            .await
    }

    // Add other database operations as needed
}
