use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, FixedOffset};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use rust_decimal::Decimal;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    apis::api_models::query::PickLeaderboardSort,
    models::{
        token_picks::{TokenPick, TokenPickResponse, TokenPicksGroup},
        tokens::{Chain, Token},
    },
    utils::{api_errors::ApiError, time::TimePeriod},
};

pub const QUALIFIED_TOKEN_PICKS_FILTER: &str = r#"
    AND t.market_cap > 40000
    AND CASE
        WHEN t.market_cap < 1000000 THEN
            t.liquidity >= (t.volume_24h * 0.04)
        ELSE
            t.liquidity >= 40000
    END
    AND t.liquidity IS NOT NULL
    AND t.volume_24h IS NOT NULL
"#;

pub const TOKEN_PICKS_FILTER_WITH_NULLS: &str = r#"
    AND COALESCE(tp.market_cap_at_call, 0) > 0
    AND (COALESCE(tp.highest_market_cap, 0) > 0 OR COALESCE(tp.highest_multiplier, 0) > 0)
"#;

const GROUP_JSON_BUILDER: &str = r#"
json_build_object(
	'id', g.id,
	'name', g.name,
	'logoUri', g.logo_uri,
	'isAdmin', g.is_admin,
	'isActive', g.is_active
)
"#;

pub enum UserPickLimitScope {
    User(Uuid, TimePeriod),
}

pub struct TokenRepository {
    db: Arc<PgPool>,
}

impl TokenRepository {
    pub fn new(db: Arc<PgPool>) -> Self {
        Self { db }
    }
}

#[derive(Debug, sqlx::FromRow, Default)]
pub struct ListTokenPicksParams {
    pub user_id: Option<Uuid>,
    pub picked_after: Option<DateTime<FixedOffset>>,
    pub page: u32,
    pub limit: u32,
    pub order_by: Option<PickLeaderboardSort>,
    pub order_direction: Option<String>,
    pub get_all: bool,
    pub group_ids: Option<Vec<i64>>,
    pub following: bool,
}

impl TokenRepository {
    pub async fn get_token(
        &self,
        address: &String,
        chain: Option<String>,
    ) -> Result<Option<Token>, sqlx::Error> {
        let chain = chain.unwrap_or(Chain::Solana.to_string());
        let query = r#"
            SELECT * FROM social.tokens WHERE address = $1 AND chain = $2
        "#;

        sqlx::query_as::<_, Token>(query)
            .bind(address)
            .bind(chain)
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn save_token(&self, token: Token) -> Result<Token, sqlx::Error> {
        let query = r#"
            INSERT INTO social.tokens (address, name, symbol, chain, volume_24h, liquidity, logo_uri)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (address, chain) DO UPDATE SET name = $2, symbol = $3, volume_24h = $5, liquidity = $6, logo_uri = $7
			RETURNING *
        "#;

        let token = sqlx::query_as::<_, Token>(query)
            .bind(token.address)
            .bind(token.name)
            .bind(token.symbol)
            .bind(token.chain)
            .bind(token.volume_24h)
            .bind(token.liquidity)
            .bind(token.logo_uri)
            .fetch_one(self.db.as_ref())
            .await?;

        Ok(token)
    }

    pub async fn save_many_tokens(&self, tokens: Vec<Token>) -> Result<(), sqlx::Error> {
        if tokens.is_empty() {
            return Ok(());
        }
        let tokens: Vec<_> = tokens
            .into_par_iter()
            .filter(|token| {
                !token.address.trim().is_empty()
                    && !token.name.trim().is_empty()
                    && !token.symbol.trim().is_empty()
            })
            .collect();

        const COLUMNS: &str =
            "(address, name, symbol, chain, market_cap, volume_24h, liquidity, logo_uri)";
        const PARAMS_PER_ROW: usize = 8;

        let value_indices: Vec<String> = tokens
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let start = i * PARAMS_PER_ROW + 1;
                format!(
                    "(${},${},${},${},${},${},${},${})",
                    start,
                    start + 1,
                    start + 2,
                    start + 3,
                    start + 4,
                    start + 5,
                    start + 6,
                    start + 7
                )
            })
            .collect();

        let query = format!(
            r#"
            INSERT INTO social.tokens {COLUMNS}
            VALUES {values}
            ON CONFLICT (address, chain)
            DO UPDATE SET
                name = CASE
                    WHEN EXCLUDED.name != '' THEN EXCLUDED.name
                    ELSE social.tokens.name
                END,
                symbol = CASE
                    WHEN EXCLUDED.symbol != '' THEN EXCLUDED.symbol
                    ELSE social.tokens.symbol
                END,
                market_cap = CASE
                    WHEN EXCLUDED.market_cap IS NOT NULL THEN EXCLUDED.market_cap
                    ELSE social.tokens.market_cap
                END,
                volume_24h = CASE
                    WHEN EXCLUDED.volume_24h IS NOT NULL THEN EXCLUDED.volume_24h
                    ELSE social.tokens.volume_24h
                END,
                liquidity = CASE
                    WHEN EXCLUDED.liquidity IS NOT NULL THEN EXCLUDED.liquidity
                    ELSE social.tokens.liquidity
                END,
                logo_uri = EXCLUDED.logo_uri
            "#,
            values = value_indices.join(",")
        );

        let mut query_builder = sqlx::query(&query);

        for token in &tokens {
            query_builder = query_builder
                .bind(&token.address)
                .bind(&token.name)
                .bind(&token.symbol)
                .bind(&token.chain)
                .bind(token.market_cap)
                .bind(token.volume_24h)
                .bind(token.liquidity)
                .bind(&token.logo_uri);
        }

        query_builder.execute(self.db.as_ref()).await?;
        Ok(())
    }

    pub async fn list_token_picks(
        &self,
        params: Option<&ListTokenPicksParams>,
        qualified: Option<bool>,
    ) -> Result<(Vec<TokenPick>, i64), sqlx::Error> {
        let mut base_query = format!(
            r#"
            SELECT tp.*,
                   row_to_json(t) AS token,
                   CASE
                       WHEN g.settings->>'privacy' = 'anonymous' THEN NULL
                       ELSE row_to_json(u)
                   END AS user,
                   row_to_json(g) AS group
            FROM social.token_picks tp
            JOIN social.tokens t ON tp.token_address = t.address
            JOIN public.user u ON tp.user_id = u.id
            JOIN social.groups g ON tp.group_id = g.id
            WHERE 1=1
            {TOKEN_PICKS_FILTER_WITH_NULLS}
            "#
        );

        if qualified.unwrap_or(true) {
            base_query += QUALIFIED_TOKEN_PICKS_FILTER;
        }

        let mut where_clauses = Vec::new();
        let mut bind_values = Vec::new();
        let mut bind_idx = 1;

        if let Some(params) = params {
            // Add user_id condition if present
            if let Some(user_id) = params.user_id {
                if params.following {
                    where_clauses.push(format!("tp.user_id IN (SELECT followed_id FROM social.user_follows WHERE follower_id = ${bind_idx})"));
                    bind_values.push(QueryValue::Uuid(user_id));
                    bind_idx += 1;
                } else {
                    where_clauses.push(format!("tp.user_id = ${bind_idx}"));
                    bind_values.push(QueryValue::Uuid(user_id));
                    bind_idx += 1;
                }
            }

            // Add picked_after condition if present
            if let Some(picked_after) = params.picked_after {
                where_clauses.push(format!("tp.call_date >= ${bind_idx}"));
                bind_values.push(QueryValue::Timestamp(picked_after));
            }
        }

        // Add where clauses to base query
        if !where_clauses.is_empty() {
            base_query += " AND ";
            base_query += &where_clauses.join(" AND ");
        }

        // Count query
        let count_query = format!("SELECT COUNT(*) FROM ({}) AS filtered", base_query);

        // Add ordering and pagination to main query
        if let Some(params) = params {
            if let Some(order_by) = &params.order_by {
                let direction = params.order_direction.as_deref().unwrap_or("ASC");
                base_query += &format!(" ORDER BY {} {}", order_by.to_string(), direction);
            } else {
                base_query += " ORDER BY call_date DESC";
            }

            // Only apply pagination if get_all is false
            if !params.get_all {
                let offset = (params.page - 1) * params.limit;
                base_query += &format!(" LIMIT {} OFFSET {}", params.limit, offset);
            }
        }

        let mut tx = self.db.begin().await?;

        // Execute count query with bindings
        let mut count_builder = sqlx::query_scalar(&count_query);
        for value in &bind_values {
            count_builder = match value {
                QueryValue::Timestamp(ts) => count_builder.bind(ts),
                QueryValue::Uuid(uuid) => count_builder.bind(uuid),
                QueryValue::Int64Array(arr) => count_builder.bind(arr),
            };
        }
        let total: i64 = count_builder.fetch_one(&mut *tx).await?;

        // Execute main query with bindings
        let mut query_builder = sqlx::query_as::<_, TokenPick>(&base_query);
        for value in &bind_values {
            query_builder = match value {
                QueryValue::Timestamp(ts) => query_builder.bind(ts),
                QueryValue::Uuid(uuid) => query_builder.bind(uuid),
                QueryValue::Int64Array(arr) => query_builder.bind(arr),
            };
        }

        let picks = query_builder.fetch_all(&mut *tx).await?;
        tx.commit().await?;

        Ok((picks, total))
    }

    pub async fn list_token_picks_group(
        &self,
        params: Option<&ListTokenPicksParams>,
    ) -> Result<(Vec<TokenPick>, i64), sqlx::Error> {
        let mut base_query = format!(
            r#"
            SELECT tp.*,
                   row_to_json(t) AS token,
                   row_to_json(u) AS user,
                   {GROUP_JSON_BUILDER} as group
            FROM social.token_picks tp
            JOIN social.tokens t ON tp.token_address = t.address
            JOIN public.user u ON tp.user_id = u.id
            JOIN social.groups g ON tp.group_id = g.id
            WHERE 1=1
            {TOKEN_PICKS_FILTER_WITH_NULLS}
            {QUALIFIED_TOKEN_PICKS_FILTER}
            "#
        );

        let mut bind_values = Vec::new();
        let mut bind_idx = 1;

        // Build where clauses with proper parameter binding
        if let Some(params) = params {
            if let Some(picked_after) = params.picked_after {
                base_query += &format!(" AND tp.call_date >= ${bind_idx}");
                bind_values.push(QueryValue::Timestamp(picked_after));
                bind_idx += 1;
            }
            if let Some(group_ids) = &params.group_ids {
                base_query += &format!(" AND tp.group_id = ANY(${bind_idx})");
                bind_values.push(QueryValue::Int64Array(group_ids.clone()));
                bind_idx += 1;
            }
            if let Some(user_id) = params.user_id {
                base_query += &format!(" AND tp.user_id = ${bind_idx}");
                bind_values.push(QueryValue::Uuid(user_id));
            }
        }

        // Count query
        let count_query = format!("SELECT COUNT(*) FROM ({}) AS filtered", base_query);

        // Add ordering and pagination
        if let Some(params) = params {
            if let Some(order_by) = &params.order_by {
                let direction = params.order_direction.as_deref().unwrap_or("ASC");
                base_query += &format!(" ORDER BY {} {}", order_by.to_string(), direction);
            } else {
                base_query += " ORDER BY call_date DESC";
            }

            if !params.get_all {
                let offset = (params.page - 1) * params.limit;
                base_query += &format!(" LIMIT {} OFFSET {}", params.limit, offset);
            }
        }

        let mut tx = self.db.begin().await?;

        // Execute count query with bindings
        let mut count_builder = sqlx::query_scalar(&count_query);
        for value in &bind_values {
            count_builder = match value {
                QueryValue::Uuid(uuid) => count_builder.bind(uuid),
                QueryValue::Int64Array(arr) => count_builder.bind(arr),
                QueryValue::Timestamp(ts) => count_builder.bind(ts),
            };
        }
        let total: i64 = count_builder.fetch_one(&mut *tx).await?;

        // Execute main query with bindings
        let mut query_builder = sqlx::query_as::<_, TokenPick>(&base_query);
        for value in &bind_values {
            query_builder = match value {
                QueryValue::Uuid(uuid) => query_builder.bind(uuid),
                QueryValue::Int64Array(arr) => query_builder.bind(arr),
                QueryValue::Timestamp(ts) => query_builder.bind(ts),
            };
        }

        let picks = query_builder.fetch_all(&mut *tx).await?;
        tx.commit().await?;

        Ok((picks, total))
    }

    pub async fn get_token_pick_by_id(&self, id: i64) -> Result<Option<TokenPick>, sqlx::Error> {
        let query = format!(
            r#"
            SELECT tp.*,
                   row_to_json(t) AS token,
                   row_to_json(u) AS user,
                   {GROUP_JSON_BUILDER} as group
            FROM social.token_picks tp
            JOIN social.tokens t ON tp.token_address = t.address
            JOIN public.user u ON tp.user_id = u.id
            JOIN social.groups g ON tp.group_id = g.id
            WHERE tp.id = $1
            "#
        );

        sqlx::query_as::<_, TokenPick>(&query)
            .bind(id)
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn update_token_picks(&self, picks: Vec<TokenPick>) -> Result<(), ApiError> {
        let query = r#"
			UPDATE social.token_picks
			SET highest_market_cap = $1,
				hit_date = $2
			WHERE id = $3
		"#
        .to_string();

        let mut tx = self.db.begin().await?;

        for pick in picks {
            let result = sqlx::query(&query)
                .bind(pick.highest_market_cap)
                .bind(pick.hit_date)
                .bind(pick.id)
                .execute(tx.as_mut())
                .await
                .map_err(|e| {
                    error!("Failed to update token pick: {}", e);
                    e
                })?;

            if result.rows_affected() != 1 {
                error!("Failed to update token pick: {}", result.rows_affected());
                tx.rollback().await?;
                return Err(ApiError::InternalServerError(
                    "Failed to update token pick".to_string(),
                ));
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn save_token_pick(&self, pick: TokenPick) -> Result<TokenPick, sqlx::Error> {
        let query = r#"
			INSERT INTO social.token_picks (
				user_id,
				group_id,
				token_address,
				telegram_message_id,
				telegram_id,
				price_at_call,
				market_cap_at_call,
				supply_at_call,
				call_date,
				highest_market_cap,
				hit_date
			)
			VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
			RETURNING id
		"#;
        let result = sqlx::query_as::<_, From>(query)
            .bind(pick.user.as_ref().map(|u| u.id))
            .bind(pick.group.id)
            .bind(pick.token.address.clone())
            .bind(pick.telegram_message_id)
            .bind(pick.telegram_id)
            .bind(pick.price_at_call)
            .bind(pick.market_cap_at_call)
            .bind(pick.supply_at_call)
            .bind(pick.call_date)
            .bind(pick.highest_market_cap)
            .bind(pick.hit_date)
            .fetch_one(self.db.as_ref())
            .await?;
        if result.id == 1 {
            info!("Successfully saved token pick with id {}", pick.id);
        }

        let token_pick = self.get_token_pick_by_id(result.id).await?;

        Ok(token_pick.unwrap_or_default())
    }

    pub async fn update_highest_market_cap(
        &self,
        pick_id: i64,
        new_highest_market_cap: Decimal,
        new_hit_date: Option<DateTime<FixedOffset>>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
			UPDATE social.token_picks
			SET highest_market_cap = $1,
				hit_date = $2
			WHERE id = $3
			"#,
        )
        .bind(new_highest_market_cap)
        .bind(new_hit_date)
        .bind(pick_id)
        .execute(self.db.as_ref())
        .await?;

        Ok(())
    }

    pub async fn count_user_picks_in_period(
        &self,
        scope: UserPickLimitScope,
    ) -> Result<i64, ApiError> {
        let query = match scope {
            UserPickLimitScope::User(user_id, time_period) => sqlx::query_scalar(
                r#"
                    SELECT COUNT(*)
                    FROM social.token_picks
                    WHERE user_id = $1
                    AND call_date >= $2
                    "#,
            )
            .bind(user_id)
            .bind(time_period.get_start_datetime().fixed_offset()),
        };

        let count = query.fetch_one(self.db.as_ref()).await?;
        Ok(count)
    }

    pub async fn get_unprocessed_token_picks(
        &self,
        limit: i64,
    ) -> Result<Vec<TokenPick>, sqlx::Error> {
        let query = format!(
            r#"
            SELECT tp.*,
                   row_to_json(t) AS token,
                   row_to_json(u) AS user,
                   {GROUP_JSON_BUILDER} as group
            FROM social.token_picks tp
            JOIN social.tokens t ON tp.token_address = t.address
            JOIN public.user u ON tp.user_id = u.id
            JOIN social.groups g ON tp.group_id = g.id
            WHERE tp.highest_market_cap IS NULL
               OR tp.hit_date IS NULL
            ORDER BY tp.call_date DESC
            LIMIT $1
        "#,
        );

        sqlx::query_as::<_, TokenPick>(&query)
            .bind(limit)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn get_all_tokens_with_picks_group_by_group_id(
        &self,
    ) -> Result<HashMap<String, Vec<TokenPick>>, sqlx::Error> {
        let query = format!(
            r#"
            SELECT t.address,
                   json_agg(
                       json_build_object(
                           'id', tp.id,
                           'group_id', tp.group_id,
                           'token', row_to_json(t),
                           'user', row_to_json(u),
                           'group', {GROUP_JSON_BUILDER},
                           'telegram_message_id', tp.telegram_message_id,
                           'telegram_id', tp.telegram_id,
                           'price_at_call', tp.price_at_call,
                           'market_cap_at_call', tp.market_cap_at_call,
                           'supply_at_call', tp.supply_at_call,
                           'call_date', tp.call_date,
                           'highest_market_cap', tp.highest_market_cap,
                           'highest_multiplier', tp.highest_multiplier,
                           'hit_date', tp.hit_date
                       )
                   ) as picks
            FROM social.tokens t
            JOIN social.token_picks tp ON t.address = tp.token_address
            JOIN public.user u ON tp.user_id = u.id
            JOIN social.groups g ON tp.group_id = g.id
            GROUP BY t.address
        "#,
        );

        let groups = sqlx::query_as::<_, TokenPicksGroup>(&query)
            .fetch_all(self.db.as_ref())
            .await?;

        let result = groups
            .into_iter()
            .map(|group| (group.address, group.picks))
            .collect();

        Ok(result)
    }

    pub async fn bulk_update_token_picks(
        &self,
        picks: &[TokenPickResponse],
    ) -> Result<(), sqlx::Error> {
        if picks.is_empty() {
            return Ok(());
        }

        let filtered_picks: Vec<_> = picks
            .iter()
            .filter(|p| p.highest_mc_post_call.is_some())
            .collect();

        if filtered_picks.is_empty() {
            return Ok(());
        }

        // Build the VALUES part of the query dynamically
        let value_placeholders: Vec<String> = (0..filtered_picks.len())
            .map(|i| format!("(${},${},${})", i * 3 + 1, i * 3 + 2, i * 3 + 3))
            .collect();

        let query = format!(
            r#"
            UPDATE social.token_picks AS t
            SET highest_market_cap = v.highest_mc,
                hit_date = v.hit_date
            FROM (VALUES {}) AS v(highest_mc, hit_date, id)
            WHERE t.id = v.id
            "#,
            value_placeholders.join(",")
        );

        let mut query_builder = sqlx::query(&query);

        // Bind all values in order
        for pick in filtered_picks {
            query_builder = query_builder
                .bind(pick.highest_mc_post_call.unwrap())
                .bind(pick.hit_date)
                .bind(pick.id);
        }

        query_builder.execute(self.db.as_ref()).await?;

        Ok(())
    }

    pub async fn get_group_leaderboard(
        &self,
        group_id: i64,
        timeframe: &TimePeriod,
        limit: i64,
    ) -> Result<Vec<TokenPick>, sqlx::Error> {
        let query = format!(
            r#"
            SELECT tp.*,
                   row_to_json(t) AS token,
                   row_to_json(u) AS user,
                   row_to_json(g) AS group
            FROM social.token_picks tp
            JOIN social.tokens t ON tp.token_address = t.address
            JOIN public.user u ON tp.user_id = u.id
            JOIN social.groups g ON tp.group_id = g.id
            WHERE tp.group_id = $1
            AND tp.call_date >= $2
            {TOKEN_PICKS_FILTER_WITH_NULLS}
            {QUALIFIED_TOKEN_PICKS_FILTER}
            ORDER BY tp.highest_multiplier DESC
            LIMIT $3
        "#,
        );

        sqlx::query_as::<_, TokenPick>(&query)
            .bind(group_id)
            .bind(timeframe.get_start_datetime().fixed_offset())
            .bind(limit)
            .fetch_all(self.db.as_ref())
            .await
    }

    pub async fn count_busts(&self, user_id: &Uuid) -> Result<i64, sqlx::Error> {
        let query = r#"
        SELECT COUNT(*)
        FROM social.token_picks tp
		JOIN social.tokens t ON tp.token_address = t.address
        WHERE tp.user_id = $1
        AND (tp.highest_market_cap < tp.market_cap_at_call * 2
		OR t.volume_24h < 20000
		OR t.liquidity < 10000
		OR t.market_cap < 30000)
        "#;

        let count = sqlx::query_scalar(&query)
            .bind(user_id)
            .fetch_one(self.db.as_ref())
            .await?;

        Ok(count)
    }

    pub async fn get_token_pick_by_telegram_data(
        &self,
        telegram_message_id: i64,
        telegram_user_id: i64,
        telegram_chat_id: i64,
    ) -> Result<Option<TokenPickRow>, sqlx::Error> {
        sqlx::query_as::<_, TokenPickRow>(
            r#"
			SELECT * FROM social.token_picks
			WHERE telegram_message_id = $1
			AND telegram_id = $2
			AND group_id = $3
		"#,
        )
        .bind(telegram_message_id)
        .bind(telegram_user_id)
        .bind(telegram_chat_id)
        .fetch_optional(self.db.as_ref())
        .await
    }

    pub async fn delete_token_pick(&self, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(r#"DELETE FROM social.token_picks WHERE id = $1"#)
            .bind(id)
            .execute(self.db.as_ref())
            .await?;
        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow, Deserialize, Clone)]
pub struct TokenPickRow {
    pub id: i64,
    pub group_id: i64,
    pub token_address: String,
    pub telegram_message_id: Option<i64>,
    pub telegram_id: Option<i64>,
    pub price_at_call: Decimal,
    pub market_cap_at_call: Decimal,
    pub supply_at_call: Option<Decimal>,
    pub call_date: DateTime<FixedOffset>,
    pub highest_market_cap: Option<Decimal>,
    pub highest_multiplier: Option<Decimal>,
    pub hit_date: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, sqlx::FromRow)]
struct From {
    id: i64,
}

enum QueryValue {
    Timestamp(DateTime<FixedOffset>),
    Uuid(Uuid),
    Int64Array(Vec<i64>),
}
