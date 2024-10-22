use axum::{
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use repositories::user_repository::UserRepository;
use services::user_service::UserService;
use sqlx::postgres::PgListener;
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};
// mod cli;
mod apis;
mod models;
mod repositories;
mod services;

#[derive(OpenApi)]
#[openapi(
    components(schemas(models::users::UserResponse)),
    tags(
        (name = "users", description = "User management API")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "DEV".to_string());
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = sqlx::postgres::PgPool::connect(&db_url).await?;

    if environment == "DEV" {
        println!("Running in DEV environment");
    } else {
        println!("Running in PROD environment");
    }
    // sqlx::migrate!().run(&db).await?;
    println!("Connected to the database successfully!");
    let db = Arc::new(db);
    let user_repository = UserRepository::new(db.clone());
    let user_service = UserService::new(Arc::new(user_repository));
    let api_doc = ApiDoc::openapi();
    let user_router: OpenApiRouter<UserService> = OpenApiRouter::new()
        .routes(routes!(apis::user_handlers::get_user))
        .routes(routes!(apis::user_handlers::follow_user))
        .routes(routes!(apis::user_handlers::unfollow_user));

    let (router, user_openapi) = OpenApiRouter::with_openapi(api_doc)
        .nest("/users", user_router)
        .split_for_parts();
    let app = router
        .merge(Scalar::with_url("/scalar", user_openapi))
        .with_state(user_service);

    let mut pg_listener = PgListener::connect_with(&db).await?;
    pg_listener.listen("new_comment").await?;
    let (tx, mut rx) = tokio::sync::broadcast::channel::<String>(16);
    let sender = tx.clone();
    tokio::spawn(async move {
        while let Ok(notification) = pg_listener.recv().await {
            let payload = notification.payload();
            println!("Notification 1: {:?}", payload);
            if let Err(e) = sender.send(payload.to_string()) {
                println!("Error sending notification: {:?}", e);
            }
        }
    });

    let mut first_rx = tx.subscribe();
    tokio::spawn(async move {
        while let Ok(notification) = first_rx.recv().await {
            println!("Notification 2: {:?}", notification);
        }
    });

    tokio::spawn(async move {
        while let Ok(notification) = rx.recv().await {
            println!("Notification 3: {:?}", notification);
        }
    });

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    // cli::run_cli(&db).await?;

    Ok(())
}
