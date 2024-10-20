use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    seed::seed_data(&db).await?;

    Ok(())
}

mod seed {
    use chrono::Utc;
    use fake::{Fake, Faker};
    use rust_decimal::prelude::FromPrimitive;
    use sqlx::PgPool;
    use uuid::Uuid;
    #[derive(sqlx::FromRow)]
    struct UserCount {
        count: i64,
    }

    pub async fn seed_data(db: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
        // Check if data already exists
        let user_count = sqlx::query_as::<_, UserCount>("SELECT COUNT(*) FROM users")
            .fetch_one(db)
            .await?;
        if user_count.count > 0 {
            println!("Data already exists, skipping seed");
            return Ok(());
        }

        let user_ids = seed_users(db, 10).await?;

        // Seed tokens
        let token_addresses = seed_tokens(db, 5).await?;

        // Seed user groups
        let group_ids = seed_user_groups(db, 5).await?;
        // Seed token calls
        seed_token_calls(db, &user_ids, &group_ids, &token_addresses, 20).await?;

        // Seed user follows
        seed_user_follows(db, &user_ids, 15).await?;

        // Seed group users
        seed_group_users(db, &user_ids, 5).await?;

        // Seed user comments
        seed_user_comments(db, &user_ids, &token_addresses, 40).await?;

        println!("Seed data inserted successfully");
        Ok(())
    }

    async fn seed_users(
        db: &PgPool,
        count: usize,
    ) -> Result<Vec<Uuid>, Box<dyn std::error::Error>> {
        let mut user_ids = Vec::new();

        for _ in 0..count {
            let id = Uuid::new_v4();
            let username: String = Faker.fake();
            let telegram_id: String = Faker.fake();
            let now = Utc::now();

            sqlx::query(
            "INSERT INTO users (id, username, telegram_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(id)
            .bind(username)
            .bind(telegram_id)
            .bind(now)
            .bind(now)
            .execute(db)
            .await?;

            user_ids.push(id);
        }

        Ok(user_ids)
    }

    async fn seed_tokens(
        db: &PgPool,
        count: usize,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut token_addresses = Vec::new();

        for _ in 0..count {
            let address: String = Faker.fake();
            let name: String = Faker.fake();
            let symbol: String = Faker.fake();
            let now = Utc::now();

            sqlx::query(
            "INSERT INTO social.tokens (address, name, symbol, created_at, updated_at) VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(&address)
            .bind(name)
            .bind(symbol)
            .bind(now)
            .bind(now)
        .execute(db)
        .await?;

            token_addresses.push(address);
        }

        Ok(token_addresses)
    }

    async fn seed_token_calls(
        db: &PgPool,
        user_ids: &[Uuid],
        group_ids: &[i64],
        token_addresses: &[String],
        count: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for _ in 0..count {
            let token_address =
                token_addresses[Faker.fake::<usize>() % token_addresses.len()].clone();
            let user_id = user_ids[Faker.fake::<usize>() % user_ids.len()];
            let call_type = if Faker.fake::<bool>() { "buy" } else { "sell" };
            let price_at_call =
                rust_decimal::Decimal::from_f64(Faker.fake::<f64>() * 1000.0).unwrap();
            let target_price =
                rust_decimal::Decimal::from_f64(Faker.fake::<f64>() * 1000.0).unwrap();
            let now = Utc::now();
            sqlx::query(
            "INSERT INTO social.token_calls (token_address, user_id, group_id, call_type, price_at_call, target_price, call_date) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            )
            .bind(token_address)
            .bind(user_id)
            .bind(group_ids[Faker.fake::<usize>() % group_ids.len()])
            .bind(call_type)
            .bind(price_at_call)
            .bind(target_price)
            .bind(now)
            .execute(db)
            .await?;
        }

        Ok(())
    }

    async fn seed_user_follows(
        db: &PgPool,
        user_ids: &[Uuid],
        count: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for _ in 0..count {
            let follower_id = user_ids[Faker.fake::<usize>() % user_ids.len()];
            let mut followed_id = user_ids[Faker.fake::<usize>() % user_ids.len()];

            // Ensure follower and followed are different users
            while follower_id == followed_id {
                followed_id = user_ids[Faker.fake::<usize>() % user_ids.len()];
            }

            let now = Utc::now();

            sqlx::query(
                "INSERT INTO social.user_follows (follower_id, followed_id, created_at) VALUES ($1, $2, $3)",
            )
            .bind(follower_id)
            .bind(followed_id)
            .bind(now)
            .execute(db)
            .await?;
        }

        Ok(())
    }

    async fn seed_user_groups(
        db: &PgPool,
        count: usize,
    ) -> Result<Vec<i64>, Box<dyn std::error::Error>> {
        let mut group_ids = Vec::new();

        for id in 0..count {
            let name: String = Faker.fake();
            let now = Utc::now();
            let id = id as i64;
            sqlx::query("INSERT INTO social.groups (id, name, created_at) VALUES ($1, $2, $3)")
                .bind(id)
                .bind(name)
                .bind(now)
                .execute(db)
                .await?;

            group_ids.push(id);
        }

        Ok(group_ids)
    }

    async fn seed_group_users(
        db: &PgPool,
        user_ids: &[Uuid],
        count: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for _ in 0..count {
            let group_id = Faker.fake::<i64>();
            let user_id = user_ids[Faker.fake::<usize>() % user_ids.len()];
            let now = Utc::now();

            sqlx::query(
                "INSERT INTO social.group_users (group_id, user_id, created_at) VALUES ($1, $2, $3)",
            )
            .bind(group_id)
            .bind(user_id)
            .bind(now)
            .execute(db)
            .await?;
        }

        Ok(())
    }

    async fn seed_user_comments(
        db: &PgPool,
        user_ids: &[Uuid],
        token_addresses: &[String],
        count: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for _ in 0..count {
            let user_id = user_ids[Faker.fake::<usize>() % user_ids.len()];
            let token_address =
                token_addresses[Faker.fake::<usize>() % token_addresses.len()].clone();
            let content: String = Faker.fake();
            let now = Utc::now();

            sqlx::query(
                "INSERT INTO social.comments (user_id, token_address, content, created_at) VALUES ($1, $2, $3, $4)",
            )
            .bind(user_id)
            .bind(token_address)
            .bind(content)
            .bind(now)
            .execute(db)
            .await?;
        }

        Ok(())
    }
}
