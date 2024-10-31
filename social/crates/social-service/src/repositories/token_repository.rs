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
    ) -> Result<Vec<TokenPick>, sqlx::Error> {
        let mut query = r#"
            SELECT tp.*,
                   row_to_json(t) AS token,
                   row_to_json(u) AS user
            FROM social.token_picks tp
            JOIN social.tokens t ON tp.token_address = t.address
            JOIN public.user u ON tp.user_id = u.id
        "#
        .to_string();

        let mut query_builder = sqlx::query_as::<_, TokenPick>(&query);

        if let Some(params) = params {
            if let Some(user_id) = params.user_id {
                query += " WHERE tp.user_id = $1";
                query_builder = sqlx::query_as(&query).bind(user_id);
            }
        }

        query_builder.fetch_all(self.db.as_ref()).await
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
                    println!("{:?}", rounded_market_cap);
                    println!("{:#?}", pick);
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
