use std::sync::Arc;

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

#[derive(Debug, sqlx::FromRow)]
pub struct ListTokenPicksParams {
    pub user_id: Option<Uuid>,
    pub page: u32,
    pub limit: u32,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
    pub get_all: bool,
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
        "#
        .to_string();

        let mut where_clause = String::new();
        if let Some(params) = params {
            if let Some(_user_id) = params.user_id {
                where_clause = " WHERE tp.user_id = $1".to_string();
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
}

#[derive(Debug, sqlx::FromRow)]
struct From {
    id: i64,
}
