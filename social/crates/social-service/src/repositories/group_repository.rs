use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub struct GroupRepository {
    db: Arc<PgPool>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct Group {
    pub id: i64,
    pub name: String,
    pub logo_uri: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct GroupUser {
    pub group_id: i64,
    pub user_id: Uuid,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

impl GroupRepository {
    pub fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }

    pub async fn upsert_group(
        &self,
        id: i64,
        name: &str,
        logo_uri: &Option<String>,
    ) -> Result<Group, sqlx::Error> {
        sqlx::query_as::<_, Group>(
            r#"
            INSERT INTO social.groups (id, name, logo_uri)
            VALUES ($1, $2, $3)
            ON CONFLICT (name) DO UPDATE
            SET name = EXCLUDED.name
            RETURNING id, name, logo_uri, created_at
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(logo_uri)
        .fetch_one(self.db.as_ref())
        .await
    }

    pub async fn get_group(&self, id: i64) -> Result<Option<Group>, sqlx::Error> {
        sqlx::query_as::<_, Group>(
            r#"
            SELECT id, name, logo_uri, created_at
            FROM social.groups
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.db.as_ref())
        .await
    }

    pub async fn add_user_to_group(
        &self,
        group_id: i64,
        user_id: Uuid,
    ) -> Result<GroupUser, sqlx::Error> {
        sqlx::query_as::<_, GroupUser>(
            r#"
            INSERT INTO social.group_users (group_id, user_id)
            VALUES ($1, $2)
            ON CONFLICT (group_id, user_id) DO NOTHING
            RETURNING group_id, user_id, joined_at
            "#,
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_one(self.db.as_ref())
        .await
    }
}
