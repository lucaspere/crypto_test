use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::groups::{CreateOrUpdateGroup, Group, GroupUser, GroupWithUsers};

pub struct GroupRepository {
    db: Arc<PgPool>,
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
    ) -> Result<CreateOrUpdateGroup, sqlx::Error> {
        sqlx::query_as::<_, CreateOrUpdateGroup>(
            r#"
            INSERT INTO social.groups (id, name, logo_uri)
            VALUES ($1, $2, $3)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name, logo_uri = EXCLUDED.logo_uri
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
            SELECT
                g.id,
                g.name,
                g.logo_uri,
                g.created_at,
                COALESCE(tp_count.total_picks, 0) as token_pick_count,
                COALESCE(gu_count.user_count, 0) as user_count,
                CASE
                    WHEN COALESCE(tp_count.total_picks, 0) = 0 THEN 0
                    ELSE COALESCE(tp_count.hits::float / tp_count.total_picks::float, 0)
                END as hit_rate,
                COALESCE(tp_count.total_returns, 0) as total_returns
            FROM social.groups g
            LEFT JOIN (
                SELECT
                    group_id,
                    COUNT(*) as total_picks,
                    SUM(CASE WHEN hit_date IS NOT NULL THEN 1 ELSE 0 END) as hits,
                    SUM(CASE
                        WHEN highest_market_cap IS NOT NULL AND market_cap_at_call > 0
                        THEN highest_market_cap::float / market_cap_at_call::float
                        ELSE 0
                    END) as total_returns
                FROM social.token_picks
                GROUP BY group_id
            ) tp_count ON g.id = tp_count.group_id
            LEFT JOIN (
                SELECT group_id, COUNT(DISTINCT user_id) as user_count
                FROM social.group_users
                GROUP BY group_id
            ) gu_count ON g.id = gu_count.group_id
            WHERE g.id = $1
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

    pub async fn remove_user_from_group(
        &self,
        group_id: i64,
        user_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM social.group_users WHERE group_id = $1 AND user_id = $2")
            .bind(group_id)
            .bind(user_id)
            .execute(self.db.as_ref())
            .await?;

        Ok(())
    }

    pub async fn get_group_user(
        &self,
        group_id: i64,
        user_id: Uuid,
    ) -> Result<Option<GroupUser>, sqlx::Error> {
        sqlx::query_as::<_, GroupUser>(
            "SELECT * FROM social.group_users WHERE group_id = $1 AND user_id = $2",
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_optional(self.db.as_ref())
        .await
    }

    pub async fn list_user_groups(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<CreateOrUpdateGroup>, sqlx::Error> {
        sqlx::query_as::<_, CreateOrUpdateGroup>(
            "SELECT * FROM social.groups WHERE id IN (SELECT group_id FROM social.group_users WHERE user_id = $1)",
        )
        .bind(user_id)
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn list_groups(&self) -> Result<Vec<Group>, sqlx::Error> {
        sqlx::query_as::<_, Group>(
            r#"SELECT
                g.id,
                g.name,
                g.logo_uri,
                g.created_at,
                COALESCE(tp_count.total_picks, 0) as token_pick_count,
                COALESCE(gu_count.user_count, 0) as user_count,
                CASE
                    WHEN COALESCE(tp_count.total_picks, 0) = 0 THEN 0
                    ELSE COALESCE(tp_count.hits::float / tp_count.total_picks::float, 0)
                END as hit_rate,
                COALESCE(tp_count.total_returns, 0) as total_returns
            FROM social.groups g
            LEFT JOIN (
                SELECT
                    group_id,
                    COUNT(*) as total_picks,
                    SUM(CASE WHEN hit_date IS NOT NULL THEN 1 ELSE 0 END) as hits,
                    SUM(CASE
                        WHEN highest_market_cap IS NOT NULL AND market_cap_at_call > 0
                        THEN highest_market_cap::float / market_cap_at_call::float
                        ELSE 0
                    END) as total_returns
                FROM social.token_picks
                GROUP BY group_id
            ) tp_count ON g.id = tp_count.group_id
            LEFT JOIN (
                SELECT group_id, COUNT(DISTINCT user_id) as user_count
                FROM social.group_users
                GROUP BY group_id
            ) gu_count ON g.id = gu_count.group_id"#,
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn list_group_members(
        &self,
        group_id: i64,
        limit: u32,
        page: u32,
    ) -> Result<(Vec<GroupWithUsers>, i64), sqlx::Error> {
        let offset = ((page - 1) * limit) as i64;

        // Get total count first
        let total = sqlx::query_scalar!(
            r#"SELECT COUNT(DISTINCT gu.user_id)
            FROM social.group_users gu
            JOIN public.user u ON gu.user_id = u.id
            WHERE gu.group_id = $1"#,
            group_id
        )
        .fetch_one(self.db.as_ref())
        .await?;

        // Get paginated results
        let members = sqlx::query_as::<_, GroupWithUsers>(
            r#"SELECT DISTINCT ON (gu.user_id) gu.*, u.username FROM social.group_users gu
            JOIN public.user u ON gu.user_id = u.id
            WHERE gu.group_id = $1
            ORDER BY gu.user_id, gu.joined_at DESC
            LIMIT $2 OFFSET $3"#,
        )
        .bind(group_id)
        .bind(limit as i64)
        .bind(offset)
        .fetch_all(self.db.as_ref())
        .await?;

        Ok((members, total.unwrap_or(0)))
    }
}
