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

    pub async fn find_by_telegram_user_id(
        &self,
        telegram_user_id: i64,
    ) -> Result<Option<User>, sqlx::Error> {
        let query = r#"
        SELECT
            u.*,
            JSON_AGG(JSON_BUILD_OBJECT('chain', wac.chain_id, 'address', wa.address)) AS wallet_addresses
        FROM
            public.user u
        LEFT JOIN
            public.wallet_account wa ON u.selected_wallet_id = wa.wallet_id
        LEFT JOIN
            public.wallet_account_chain wac ON wac.wallet_account_address = wa.address
        WHERE u.telegram_id = $1
        GROUP BY u.id
        "#;
        sqlx::query_as::<_, User>(query)
            .bind(telegram_user_id)
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        let query = r#"
        SELECT
            u.*,
            JSON_AGG(JSON_BUILD_OBJECT('chain', wac.chain_id, 'address', wa.address)) AS wallet_addresses
        FROM
            public.user u
        LEFT JOIN
            public.wallet_account wa ON u.selected_wallet_id = wa.wallet_id
        LEFT JOIN
            public.wallet_account_chain wac ON wac.wallet_account_address = wa.address
        WHERE u.id = $1
        GROUP BY u.id
        "#;
        sqlx::query_as::<_, User>(query)
            .bind(id)
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn follow_user(
        &self,
        follower_id: Uuid,
        followed_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO social.user_follows (follower_id, followed_id, created_at) VALUES ($1, $2, $3)"
        )
        .bind(follower_id)
        .bind(followed_id)
        .bind(Utc::now())
        .execute(self.db.as_ref())
        .await?;

        Ok(())
    }

    pub async fn unfollow_user(
        &self,
        follower_id: Uuid,
        followed_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM social.user_follows WHERE follower_id = $1 AND followed_id = $2")
            .bind(follower_id)
            .bind(followed_id)
            .execute(self.db.as_ref())
            .await?;

        Ok(())
    }

    pub async fn get_followers(&self, user_id: Uuid) -> Result<Vec<User>, sqlx::Error> {
        let query = r#"
        SELECT
            u.*,
            JSON_AGG(JSON_BUILD_OBJECT('chain', wac.chain_id, 'address', wa.address)) AS wallet_addresses
        FROM
            public.user u
        LEFT JOIN
            public.wallet_account wa ON u.selected_wallet_id = wa.wallet_id
        LEFT JOIN
            public.wallet_account_chain wac ON wac.wallet_account_address = wa.address
        INNER JOIN social.user_follows uf ON u.id = uf.follower_id
        WHERE uf.followed_id = $1
        GROUP BY u.id
        "#;

        let followers = sqlx::query_as::<_, User>(query)
            .bind(user_id)
            .fetch_all(self.db.as_ref())
            .await?;

        Ok(followers)
    }

    pub async fn get_following(&self, user_id: Uuid) -> Result<Vec<User>, sqlx::Error> {
        let query = r#"
        SELECT
            u.*,
            JSON_AGG(JSON_BUILD_OBJECT('chain', wac.chain_id, 'address', wa.address)) AS wallet_addresses
        FROM
            public.user u
        LEFT JOIN
            public.wallet_account wa ON u.selected_wallet_id = wa.wallet_id
        LEFT JOIN
            public.wallet_account_chain wac ON wac.wallet_account_address = wa.address
        INNER JOIN social.user_follows uf ON u.id = uf.followed_id
        WHERE uf.follower_id = $1
        GROUP BY u.id
        "#;

        let following = sqlx::query_as::<_, User>(query)
            .bind(user_id)
            .fetch_all(self.db.as_ref())
            .await?;

        Ok(following)
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        let query = r#"
        SELECT
            u.*,
            JSON_AGG(JSON_BUILD_OBJECT('chain', wac.chain_id, 'address', wa.address)) AS wallet_addresses
        FROM
            public.user u
        LEFT JOIN
            public.wallet_account wa ON u.selected_wallet_id = wa.wallet_id
        LEFT JOIN
            public.wallet_account_chain wac ON wac.wallet_account_address = wa.address
        WHERE u.username = $1
        GROUP BY u.id
        "#;
        let user = sqlx::query_as::<_, User>(query)
            .bind(username)
            .fetch_optional(self.db.as_ref())
            .await?;

        Ok(user)
    }

    pub async fn list_followers(&self, username: &str) -> Result<Vec<User>, sqlx::Error> {
        let query = r#"
        SELECT
            u.*,
            JSON_AGG(JSON_BUILD_OBJECT('chain', wac.chain_id, 'address', wa.address)) AS wallet_addresses
        FROM
            public.user u
        LEFT JOIN
            public.wallet_account wa ON u.selected_wallet_id = wa.wallet_id
        LEFT JOIN
            public.wallet_account_chain wac ON wac.wallet_account_address = wa.address
        INNER JOIN social.user_follows uf ON u.id = uf.follower_id
        WHERE uf.followed_id = (SELECT id FROM public.user WHERE username = $1)
        GROUP BY u.id
        "#;
        sqlx::query_as::<_, User>(&query)
            .bind(username)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_users(&self) -> Result<Vec<User>, sqlx::Error> {
        let query = r#"
        SELECT
            u.*,
            JSON_AGG(JSON_BUILD_OBJECT('chain', wac.chain_id, 'address', wa.address)) AS wallet_addresses
        FROM
            public.user u
        LEFT JOIN
            public.wallet_account wa ON u.selected_wallet_id = wa.wallet_id
        LEFT JOIN
            public.wallet_account_chain wac ON wac.wallet_account_address = wa.address
        GROUP BY u.id
        "#;
        sqlx::query_as::<_, User>(query)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn is_following(
        &self,
        follower_id: Uuid,
        followed_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let query = r#"
        SELECT COUNT(*) FROM social.user_follows WHERE follower_id = $1 AND followed_id = $2
        "#;
        let count = sqlx::query_scalar::<_, i64>(query)
            .bind(follower_id)
            .bind(followed_id)
            .fetch_one(self.db.as_ref())
            .await?;

        Ok(count > 0)
    }
}
