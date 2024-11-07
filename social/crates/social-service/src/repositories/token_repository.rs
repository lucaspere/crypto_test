use std::sync::Arc;

use chrono::{DateTime, FixedOffset};
use rust_decimal::Decimal;
use sqlx::PgPool;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    models::{
        token_picks::TokenPick,
        tokens::{Chain, Token},
    },
    utils::api_errors::ApiError,
};

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
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
    pub get_all: bool,
    pub group_ids: Option<Vec<i64>>,
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
            INSERT INTO social.tokens (address, name, symbol, chain)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (address, chain) DO UPDATE SET name = $2, symbol = $3
			RETURNING *
        "#;

        let token = sqlx::query_as::<_, Token>(query)
            .bind(token.address)
            .bind(token.name)
            .bind(token.symbol)
            .bind(token.chain)
            .fetch_one(self.db.as_ref())
            .await?;

        Ok(token)
    }

    pub async fn list_token_picks(
        &self,
        params: Option<&ListTokenPicksParams>,
    ) -> Result<(Vec<TokenPick>, i64), sqlx::Error> {
        let mut base_query = r#"
            SELECT tp.*,
                   row_to_json(t) AS token,
                   row_to_json(u) AS user
            FROM social.token_picks tp
            JOIN social.tokens t ON tp.token_address = t.address
            JOIN public.user u ON tp.user_id = u.id
            WHERE 1=1
        "#
        .to_string();

        let mut where_clauses = Vec::new();
        let mut bind_values = Vec::new();
        let mut bind_idx = 1;

        if let Some(params) = params {
            // Add picked_after condition if present
            if let Some(picked_after) = params.picked_after {
                where_clauses.push(format!("tp.call_date >= ${}", bind_idx));
                bind_values.push(QueryValue::Timestamp(picked_after));
                bind_idx += 1;
            }

            // Add user_id condition if present
            if let Some(user_id) = params.user_id {
                where_clauses.push(format!("tp.user_id = ${}", bind_idx));
                bind_values.push(QueryValue::Uuid(user_id));
                bind_idx += 1;
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
                base_query += &format!(" ORDER BY {} {}", order_by, direction);
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
            };
        }
        let total: i64 = count_builder.fetch_one(&mut *tx).await?;

        // Execute main query with bindings
        let mut query_builder = sqlx::query_as::<_, TokenPick>(&base_query);
        for value in &bind_values {
            query_builder = match value {
                QueryValue::Timestamp(ts) => query_builder.bind(ts),
                QueryValue::Uuid(uuid) => query_builder.bind(uuid),
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
        let mut base_query = r#"
            SELECT tp.*,
                   row_to_json(t) AS token,
                   row_to_json(u) AS user
            FROM social.token_picks tp
            JOIN social.tokens t ON tp.token_address = t.address
            JOIN public.user u ON tp.user_id = u.id
            JOIN social.group_users gu ON tp.group_id = gu.group_id
            WHERE gu.user_id = tp.user_id
        "#
        .to_string();

        let mut where_clause = String::new();
        if let Some(params) = params {
            if let Some(group_ids) = &params.group_ids {
                where_clause = format!(
                    " AND tp.group_id IN ({})",
                    group_ids
                        .iter()
                        .map(|id| id.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                );
            }
        }

        base_query += &where_clause;

        // Count query
        let count_query = format!("SELECT COUNT(*) FROM ({}) AS filtered", base_query);

        // Add ordering and pagination to main query
        if let Some(params) = params {
            if let Some(order_by) = &params.order_by {
                let direction = params.order_direction.as_deref().unwrap_or("ASC");
                base_query += &format!(" ORDER BY {} {}", order_by, direction);
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

        // Execute count query
        let mut count_builder = sqlx::query_scalar(&count_query);
        if let Some(params) = params {
            if let Some(user_id) = params.user_id {
                count_builder = count_builder.bind(user_id);
            }
        }
        let total: i64 = count_builder.fetch_one(&mut *tx).await?;

        // Execute main query
        let mut query_builder = sqlx::query_as::<_, TokenPick>(&base_query);
        if let Some(params) = params {
            if let Some(user_id) = params.user_id {
                query_builder = query_builder.bind(user_id);
            }
        }

        let picks = query_builder.fetch_all(&mut *tx).await?;
        tx.commit().await?;

        Ok((picks, total))
    }

    pub async fn get_token_pick_by_id(&self, id: i64) -> Result<Option<TokenPick>, sqlx::Error> {
        let query = r#"
            SELECT tp.*,
                   row_to_json(t) AS token,
                   row_to_json(u) AS user
            FROM social.token_picks tp
            JOIN social.tokens t ON tp.token_address = t.address
            JOIN public.user u ON tp.user_id = u.id
            WHERE tp.id = $1
        "#;

        sqlx::query_as::<_, TokenPick>(query)
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
            let highest_market_cap = pick.highest_market_cap.unwrap_or_default();
            let rounded_market_cap = highest_market_cap.round_dp(8);

            let result = sqlx::query(&query)
                .bind(rounded_market_cap)
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
			INSERT INTO social.token_picks (user_id, group_id, token_address, price_at_call, market_cap_at_call, supply_at_call, call_date, highest_market_cap, hit_date)
			VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
			RETURNING id
		"#;
        let result = sqlx::query_as::<_, From>(query)
            .bind(pick.user.id)
            .bind(pick.group_id)
            .bind(pick.token.address.clone())
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

    //     pub async fn list_token_picks_by_group(
    //         &self,
    //         params: Option<&ListTokenPicksParams>,
    //     ) -> Result<(Vec<TokenPick>, i64), sqlx::Error> {
    //         let mut base_query = r#"
    //             SELECT tp.*,
    //                    row_to_json(t) AS token,
    //                    row_to_json(u) AS user,
    //                    row_to_json(g) AS group
    //             FROM social.token_picks tp
    //             JOIN social.tokens t ON tp.token_address = t.address
    //             JOIN public.user u ON tp.user_id = u.id
    //             JOIN social.groups g ON tp.group_id = g.id
    //         "#
    //         .to_string();

    //         let mut where_clause = Vec::new();
    //         let mut bind_idx = 1;
    //         let mut bindings = Vec::new();

    //         if let Some(params) = params {
    //             if let Some(user_id) = params.user_id {
    //                 where_clause.push(format!("tp.user_id = ${}", bind_idx));
    //                 bind_idx += 1;
    //                 bindings.push(user_id);
    //             }
    //             if let Some(group_id) = params.group_id {
    //                 where_clause.push(format!("tp.group_id = ${}", bind_idx));
    //                 bindings.push(group_id);
    //             }
    //         }

    //         if !where_clause.is_empty() {
    //             base_query += " WHERE ";
    //             base_query += &where_clause.join(" AND ");
    //         }

    //         // Rest of the implementation similar to list_token_picks
    //         // ... (pagination, ordering, etc.)

    //         Ok((picks, total))
    //     }

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
        .bind(new_highest_market_cap.round_dp(8))
        .bind(new_hit_date)
        .bind(pick_id)
        .execute(self.db.as_ref())
        .await?;

        Ok(())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct From {
    id: i64,
}

// Helper enum for query parameter binding
enum QueryValue {
    Timestamp(DateTime<FixedOffset>),
    Uuid(Uuid),
}
