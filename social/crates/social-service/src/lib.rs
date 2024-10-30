#![allow(dead_code)]

use apis::setup_routes;
use axum::Router;
use external_services::{birdeye::BirdeyeService, rust_monorepo::RustMonorepoService};
use repositories::{token_repository::TokenRepository, user_repository::UserRepository};
use services::{
    profile_service::ProfileService, token_service::TokenService, user_service::UserService,
};
use sqlx::postgres::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

pub mod apis;
pub mod external_services;
pub mod models;
pub mod repositories;
pub mod services;
pub mod settings;
pub mod utils;

pub struct AppState {
    pub user_service: UserService,
    pub profile_service: ProfileService,
    pub token_service: TokenService,
}

pub async fn setup_database(database_url: &str) -> Result<Arc<PgPool>, sqlx::Error> {
    let pool = PgPool::connect(database_url).await?;
    Ok(Arc::new(pool))
}

pub async fn setup_router(
    settings: &settings::Settings,
) -> Result<Router, Box<dyn std::error::Error>> {
    let db = setup_database(&settings.database_url).await?;
    let (user_service, profile_service, token_service) = setup_services(db, settings).await?;
    let router = setup_routes();

    Ok(router
        .layer(CorsLayer::permissive())
        .with_state(Arc::new(AppState {
            user_service,
            profile_service,
            token_service,
        })))
}

pub async fn setup_services(
    db: Arc<PgPool>,
    settings: &settings::Settings,
) -> Result<(UserService, ProfileService, TokenService), Box<dyn std::error::Error>> {
    let user_repository = Arc::new(UserRepository::new(db.clone()));
    let token_repository = Arc::new(TokenRepository::new(db.clone()));

    let user_service = UserService::new(user_repository.clone());
    let birdeye_service = Arc::new(BirdeyeService::new(settings.birdeye_api_key.clone()));
    let rust_monorepo = Arc::new(RustMonorepoService::new(settings.rust_monorepo_url.clone()));
    let profile_service = ProfileService::new(
        user_repository,
        token_repository.clone(),
        rust_monorepo.clone(),
        birdeye_service,
    );
    let token_service = TokenService::new(
        token_repository,
        rust_monorepo,
        Arc::new(user_service.clone()),
    );

    Ok((user_service, profile_service, token_service))
}

pub fn init_tracing(settings: &settings::Settings) {
    let env = settings.environment.clone().unwrap_or("DEV".to_string());
    let level = match env.as_str() {
        "PROD" => tracing::Level::INFO,
        _ => tracing::Level::DEBUG,
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_ansi(env != "PROD")
        .init();
}
