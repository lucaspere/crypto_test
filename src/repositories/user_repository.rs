use crate::models::users::User;
use chrono::Utc;
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

    pub async fn follow_user(
        &self,
        follower_id: Uuid,
        followed_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
        "INSERT INTO social.user_follows (follower_id, followed_id, created_at) VALUES ($1, $2, $3)",
        follower_id,
        followed_id,
        Utc::now()
        )
        .execute(self.db.as_ref())
        .await?;

        Ok(())
    }

    pub async fn unfollow_user(
        &self,
        follower_id: Uuid,
        followed_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "DELETE FROM social.user_follows WHERE follower_id = $1 AND followed_id = $2",
            follower_id,
            followed_id
        )
        .execute(self.db.as_ref())
        .await?;

        Ok(())
    }

    pub async fn get_followers(&self, user_id: Uuid) -> Result<Vec<User>, sqlx::Error> {
        let followers = sqlx::query_as::<_, User>(
            r#"
        SELECT u.id, u.username, u.telegram_id, u.created_at, u.updated_at
        FROM users u
        INNER JOIN social.user_follows uf ON u.id = uf.follower_id
        WHERE uf.followed_id = $1
        "#,
        )
        .bind(user_id)
        .fetch_all(self.db.as_ref())
        .await?;

        Ok(followers)
    }

    pub async fn get_following(&self, user_id: Uuid) -> Result<Vec<User>, sqlx::Error> {
        let following = sqlx::query_as::<_, User>(
            r#"
        SELECT u.id, u.username, u.telegram_id, u.created_at, u.updated_at
        FROM users u
        INNER JOIN social.user_follows uf ON u.id = uf.followed_id
        WHERE uf.follower_id = $1
        "#,
        )
        .bind(user_id)
        .fetch_all(self.db.as_ref())
        .await?;

        Ok(following)
    }
}
