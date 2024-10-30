use std::sync::Arc;

use sqlx::PgPool;
use tracing::error;
use uuid::Uuid;

use crate::{
    models::{
        token_picks::{ProfilePicksAndStatsQuery, TokenPick},
        tokens::Token,
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

impl TokenRepository {
    pub async fn find_by_address(&self, address: String) -> Result<Option<Token>, sqlx::Error> {
        sqlx::query_as::<_, Token>("SELECT * FROM tokens WHERE address = $1")
            .bind(address)
            .fetch_optional(self.db.as_ref())
            .await
    }

    pub async fn find_token_picks_by_user_id(
        &self,
        user_id: Uuid,
        params: &ProfilePicksAndStatsQuery,
    ) -> Result<Vec<TokenPick>, sqlx::Error> {
        let mut query = r#"
            SELECT tp.*,
                   row_to_json(t) AS token
            FROM social.token_picks tp
            JOIN social.tokens t ON tp.token_address = t.address
            WHERE tp.user_id = $1
        "#
        .to_string();

        if let Some(multiplier) = params.multiplier {
            query += &format!(
                " AND COALESCE(tp.highest_market_cap / NULLIF(tp.market_cap_at_call, 0), 0) >= {}",
                multiplier
            );
        }

        sqlx::query_as::<_, TokenPick>(query.as_str())
            .bind(user_id)
            .fetch_all(self.db.as_ref())
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
                .await?;

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
}
