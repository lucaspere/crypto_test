#![allow(dead_code)]

use apis::setup_routes;
use axum::{
    http::{request::Parts, HeaderValue},
    Router,
};
use container::ServiceContainer;
use events::{
    handlers::{token_pick::TokenPickHandler, EventHandler},
    listeners::PostgresEventListener,
    types::Channel,
};
use services::{
    group_service::GroupService, profile_service::ProfileService, s3_service::S3Service,
    token_service::TokenService, user_service::UserService,
};
use settings::Settings;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::{collections::HashMap, sync::Arc};
use tower_http::cors::{AllowHeaders, AllowOrigin, Any, CorsLayer};
use utils::errors::app_error::AppError;

pub mod apis;
pub mod container;
pub mod events;
pub mod external_services;
pub mod jobs;
pub mod models;
pub mod repositories;
pub mod services;
pub mod settings;
pub mod utils;

pub struct AppState {
    pub user_service: Arc<UserService>,
    pub profile_service: Arc<ProfileService>,
    pub token_service: Arc<TokenService>,
    pub group_service: Arc<GroupService>,
    pub s3_service: Arc<S3Service>,
}

pub async fn setup_database(database_url: &str) -> Result<Arc<PgPool>, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .connect(&database_url)
        .await?;
    Ok(Arc::new(pool))
}

pub async fn setup_router(
    settings: &settings::Settings,
) -> Result<(Router, Arc<ServiceContainer>), Box<dyn std::error::Error>> {
    let db = setup_database(&settings.database_url).await?;
    let container = setup_services(db, settings).await?;
    let router = create_routes();
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(AllowHeaders::list(vec![
            "content-type".parse().unwrap(),
            "x-telegram-auth".parse().unwrap(),
            "authorization".parse().unwrap(),
        ]))
        .allow_origin(AllowOrigin::predicate(
            |origin: &HeaderValue, _request_parts: &Parts| {
                origin.as_bytes().ends_with(b".bullpen.fi")
            },
        ))
        .allow_credentials(false);

    let router = router.layer(cors);
    Ok((
        router.with_state(Arc::new(AppState {
            user_service: Arc::clone(&container.user_service),
            profile_service: Arc::clone(&container.profile_service),
            token_service: Arc::clone(&container.token_service),
            group_service: Arc::clone(&container.group_service),
            s3_service: Arc::clone(&container.s3_service),
        })),
        Arc::new(container),
    ))
}

fn create_routes() -> Router<Arc<AppState>> {
    setup_routes()
}

pub async fn setup_services(
    db: Arc<PgPool>,
    settings: &settings::Settings,
) -> Result<ServiceContainer, Box<dyn std::error::Error>> {
    let container = ServiceContainer::new(settings, db).await?;
    Ok(container)
}

pub fn init_tracing(settings: &settings::Settings) {
    let env = settings.environment.clone().unwrap_or("DEV".to_string());
    let level = match env.as_str() {
        "prod" => tracing::Level::INFO,
        _ => tracing::Level::DEBUG,
    };

    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_ansi(env != "prod")
        .init();
}

pub async fn start_event_listeners(
    settings: Arc<Settings>,
    services: Arc<ServiceContainer>,
) -> Result<(), AppError> {
    let mut handlers = HashMap::new();
    handlers.insert(
        Channel::TokenPick,
        Box::new(TokenPickHandler::new(services)) as Box<dyn EventHandler>,
    );

    let mut listener = PostgresEventListener::new(settings, handlers).await?;
    listener.start().await?;

    Ok(())
}
