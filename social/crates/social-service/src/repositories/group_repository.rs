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
                COALESCE(tp_count.count, 0) as token_pick_count,
                COALESCE(gu_count.count, 0) as user_count,
                COALESCE(tp_hit_rate.hit_rate, 0) as hit_rate
            FROM social.groups g
            LEFT JOIN (
                SELECT group_id, COUNT(*) as count
                FROM social.token_picks
                GROUP BY group_id
            ) tp_count ON g.id = tp_count.group_id
            LEFT JOIN (
                SELECT group_id, COUNT(DISTINCT user_id) as count
                FROM social.group_users
                GROUP BY group_id
            ) gu_count ON g.id = gu_count.group_id
            LEFT JOIN (
                SELECT group_id, COUNT(hit_date) as hit_rate
                FROM social.token_picks
                GROUP BY group_id
            ) tp_hit_rate ON g.id = tp_hit_rate.group_id
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
                COALESCE(tp_count.count, 0) as token_pick_count,
                COALESCE(gu_count.count, 0) as user_count,
                COALESCE(tp_hit_rate.hit_rate, 0) as hit_rate
            FROM social.groups g
            LEFT JOIN (
                SELECT group_id, COUNT(*) as count
                FROM social.token_picks
                GROUP BY group_id
            ) tp_count ON g.id = tp_count.group_id
            LEFT JOIN (
                SELECT group_id, COUNT(DISTINCT user_id) as count
                FROM social.group_users
                GROUP BY group_id
            ) gu_count ON g.id = gu_count.group_id
            LEFT JOIN (
                SELECT group_id, COUNT(hit_date) as hit_rate
                FROM social.token_picks
                GROUP BY group_id
            ) tp_hit_rate ON g.id = tp_hit_rate.group_id"#,
        )
        .fetch_all(self.db.as_ref())
        .await
    }

    pub async fn list_group_members(
        &self,
        group_id: i64,
        limit: u32,
        page: u32,
    ) -> Result<Vec<GroupWithUsers>, sqlx::Error> {
        sqlx::query_as::<_, GroupWithUsers>(
            r#"SELECT gu.*, u.username FROM social.group_users gu
            JOIN public.user u ON gu.user_id = u.id
            WHERE gu.group_id = $1
            ORDER BY gu.joined_at DESC
            LIMIT $2 OFFSET $3"#,
        )
        .bind(group_id)
        .bind(limit as i64)
        .bind(((page - 1) * limit) as i64)
        .fetch_all(self.db.as_ref())
        .await
    }
}
