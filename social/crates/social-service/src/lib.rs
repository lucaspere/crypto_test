#![allow(dead_code)]

use apis::setup_routes;
use axum::Router;
use repositories::{token_repository::TokenRepository, user_repository::UserRepository};
use services::{profile_service::ProfileService, user_service::UserService};
use sqlx::postgres::PgPool;
use std::sync::Arc;
use tracing::debug;
pub mod apis;
pub mod models;
pub mod repositories;
pub mod services;
pub mod settings;
pub mod utils;

pub struct AppState {
    pub user_service: UserService,
    pub profile_service: ProfileService,
}

pub async fn setup_database(database_url: &str) -> Result<Arc<PgPool>, sqlx::Error> {
    let pool = PgPool::connect(database_url).await?;
    Ok(Arc::new(pool))
}

pub async fn setup_router(
    settings: &settings::Settings,
) -> Result<Router, Box<dyn std::error::Error>> {
    let db = setup_database(&settings.database_url).await?;
    debug!("Connected to the database successfully!");
    let (user_service, profile_service) = setup_services(db).await?;
    let router = setup_routes();

    Ok(router.with_state(Arc::new(AppState {
        user_service,
        profile_service,
    })))
}

pub async fn setup_services(
    db: Arc<PgPool>,
) -> Result<(UserService, ProfileService), Box<dyn std::error::Error>> {
    let user_repository = Arc::new(UserRepository::new(db.clone()));
    let token_repository = Arc::new(TokenRepository::new(db.clone()));

    let user_service = UserService::new(user_repository.clone());
    let profile_service = ProfileService::new(user_repository, token_repository);

    Ok((user_service, profile_service))
}

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
}
