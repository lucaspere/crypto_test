use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    apis::api_models::query::ListGroupsQuery,
    models::groups::{CreateOrUpdateGroup, Group, GroupSettings, GroupUser, GroupWithUsers},
    repositories::token_repository::QUALIFIED_TOKEN_PICKS_FILTER,
};

const GROUP_STATS_CTE: &str = r#"
    WITH token_pick_stats AS (
        SELECT
            group_id,
            COUNT(*) as total_picks,
            COUNT(CASE WHEN hit_date IS NOT NULL THEN 1 END) as hits,
            SUM(
                CASE
                    WHEN highest_market_cap IS NOT NULL
                        AND market_cap_at_call > 0
                    THEN (highest_market_cap::float / market_cap_at_call::float)
                    ELSE 0
                END
            ) as total_returns,
            AVG(
                CASE
                    WHEN highest_market_cap IS NOT NULL
                        AND market_cap_at_call > 0
                    THEN (highest_market_cap::float / market_cap_at_call::float)
                    ELSE 0
                END
            ) as average_returns
        FROM social.token_picks tp
        JOIN social.tokens t ON tp.token_address = t.address
        WHERE 1=1"#;

const GROUP_SELECT_QUERY: &str = r#"
    SELECT
        g.id,
        g.name,
        g.logo_uri,
        g.is_admin,
        g.is_active,
        g.created_at,
        g.settings,
        COALESCE(tp.total_picks, 0) as token_pick_count,
        COALESCE(gu.user_count, 0) as user_count,
        COALESCE(
            CASE
                WHEN tp.total_picks > 0 THEN (tp.hits::float * 100) / tp.total_picks::float
                ELSE 0
            END,
            0
        ) as hit_rate,
        COALESCE(tp.total_returns, 0) as total_returns,
        COALESCE(tp.average_returns, 0) as average_returns
    FROM social.groups g
    LEFT JOIN token_pick_stats tp ON g.id = tp.group_id
    LEFT JOIN group_user_counts gu ON g.id = gu.group_id
"#;

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
        is_admin: &Option<bool>,
        is_active: &Option<bool>,
        settings: &GroupSettings,
    ) -> Result<CreateOrUpdateGroup, sqlx::Error> {
        let settings_json = serde_json::to_value(settings).ok().unwrap_or_default();
        sqlx::query_as::<_, CreateOrUpdateGroup>(
            r#"
            INSERT INTO social.groups (id, name, logo_uri, is_admin, is_active, settings)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE
            SET
                name = CASE
                    WHEN LENGTH($2) > 0 THEN $2
                    ELSE social.groups.name
                END,
                logo_uri = CASE
                    WHEN $3 IS NOT NULL THEN $3
                    ELSE social.groups.logo_uri
                END,
                is_admin = CASE
                    WHEN $4 IS NOT NULL THEN $4
                    ELSE social.groups.is_admin
                END,
                is_active = CASE
                    WHEN $5 IS NOT NULL THEN $5
                    ELSE social.groups.is_active
                END,
                settings = CASE
                    WHEN $6 IS NOT NULL THEN $6
                    ELSE social.groups.settings
                END
            RETURNING id, name, logo_uri, created_at, is_admin, is_active, settings
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(logo_uri)
        .bind(is_admin)
        .bind(is_active)
        .bind(settings_json)
        .fetch_one(self.db.as_ref())
        .await
    }

    pub async fn get_group(&self, id: i64) -> Result<Option<Group>, sqlx::Error> {
        let query = format!(
            r#"
            {GROUP_STATS_CTE}
            {QUALIFIED_TOKEN_PICKS_FILTER}
            GROUP BY group_id
            ),
            group_user_counts AS (
                SELECT
                    group_id,
                    COUNT(DISTINCT user_id) as user_count
                FROM social.group_users
                GROUP BY group_id
            )
            {GROUP_SELECT_QUERY}
            WHERE g.id = $1
            "#
        );

        sqlx::query_as::<_, Group>(&query)
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
            ON CONFLICT (group_id, user_id) DO UPDATE
            SET group_id = EXCLUDED.group_id
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

    pub async fn list_groups(&self, params: &ListGroupsQuery) -> Result<Vec<Group>, sqlx::Error> {
        let mut query = format!(
            r#"
            {GROUP_STATS_CTE}
            {QUALIFIED_TOKEN_PICKS_FILTER}
            GROUP BY group_id
            ),
            group_user_counts AS (
                SELECT
                    group_id,
                    COUNT(DISTINCT user_id) as user_count
                FROM social.group_users
                GROUP BY group_id
            )
            {GROUP_SELECT_QUERY}
            "#
        );

        if let Some(_user_id) = params.user_id {
            query.push_str(
                " WHERE g.id IN (SELECT group_id FROM social.group_users WHERE user_id = $1)",
            );
        }

        sqlx::query_as::<_, Group>(&query)
            .bind(params.user_id)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn list_group_members(
        &self,
        group_id: i64,
        limit: u32,
        page: u32,
        get_all: bool,
    ) -> Result<(Vec<GroupWithUsers>, String, i64), sqlx::Error> {
        // Get total count first
        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(DISTINCT gu.user_id)
            FROM social.group_users gu
            JOIN public.user u ON gu.user_id = u.id
            WHERE gu.group_id = $1"#,
        )
        .bind(group_id)
        .fetch_one(self.db.as_ref())
        .await?;

        // Base query without pagination
        let base_query = r#"SELECT DISTINCT ON (gu.user_id) gu.*, u.username, g.name
            FROM social.group_users gu
            JOIN public.user u ON gu.user_id = u.id
            JOIN social.groups g ON gu.group_id = g.id
            WHERE gu.group_id = $1
            ORDER BY gu.user_id, gu.joined_at DESC"#;

        // Add pagination only if get_all is false
        let query = if get_all {
            base_query.to_string()
        } else {
            format!("{} LIMIT $2 OFFSET $3", base_query)
        };

        // Execute query based on get_all parameter
        let members = if get_all {
            sqlx::query_as::<_, GroupWithUsers>(&query)
                .bind(group_id)
                .fetch_all(self.db.as_ref())
                .await?
        } else {
            sqlx::query_as::<_, GroupWithUsers>(&query)
                .bind(group_id)
                .bind(limit as i64)
                .bind(((page - 1) * limit) as i64)
                .fetch_all(self.db.as_ref())
                .await?
        };

        let group_name = sqlx::query_scalar(r#"SELECT name FROM social.groups WHERE id = $1"#)
            .bind(group_id)
            .fetch_one(self.db.as_ref())
            .await?;

        Ok((members, group_name, total))
    }

    pub async fn group_exists(&self, group_id: i64) -> Result<bool, sqlx::Error> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM social.groups WHERE id = $1)")
                .bind(group_id)
                .fetch_one(self.db.as_ref())
                .await?;

        Ok(exists)
    }
}
