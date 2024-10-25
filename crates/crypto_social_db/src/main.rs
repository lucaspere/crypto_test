use dotenv::dotenv;
use repositories::user_repository::UserRepository;
use services::{
    notification_service::NotificationService, profile_service::ProfileService,
    user_service::UserService,
};
use sqlx::postgres::PgListener;
use std::env;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::broadcast};
use tokio_tungstenite::accept_async;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};
use workers::notification_workers::NotificationWorker;
// mod cli;
mod apis;
mod models;
mod repositories;
mod services;
mod websocket;
mod workers;

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
    let redis_url = env::var("REDIS_URL").expect("REDIS_URL must be set");
    // sqlx::migrate!().run(&db).await?;
    println!("Connected to the database successfully!");
    let db = Arc::new(db);
    let user_repository = UserRepository::new(db.clone());
    let notification_service = Arc::new(NotificationService::new(&redis_url)?);
    let notification_worker = NotificationWorker::new(notification_service.clone());
    let user_service = UserService::new(Arc::new(user_repository));
    let profile_service = ProfileService::new();
    let api_doc = ApiDoc::openapi();
    let profile_router: OpenApiRouter<ProfileService> = OpenApiRouter::new()
        .routes(routes!(apis::profile_handlers::get_profile_details))
        .routes(routes!(apis::profile_handlers::get_user_stats))
        .routes(routes!(apis::profile_handlers::get_user_picks));

    let user_router: OpenApiRouter<UserService> = OpenApiRouter::new()
        .routes(routes!(apis::user_handlers::follow_user))
        .routes(routes!(apis::user_handlers::unfollow_user))
        .routes(routes!(apis::user_handlers::get_notification_preferences))
        .routes(routes!(apis::user_handlers::set_notification_preferences));

    let (router, user_openapi) = OpenApiRouter::with_openapi(api_doc)
        .nest("/users", user_router)
        .split_for_parts();
    let (_router, profile_openapi) = OpenApiRouter::with_openapi(user_openapi)
        .nest("/profile", profile_router)
        .split_for_parts();
    let app = router
        .merge(Scalar::with_url("/scalar", profile_openapi))
        .with_state(user_service)
        .with_state(notification_service);

    let (notification_tx, _) = broadcast::channel::<String>(100);
    let notification_tx = Arc::new(notification_tx);

    let websocket_addr = "127.0.0.1:8000";
    let websocket_listener = TcpListener::bind(websocket_addr).await?;
    println!("WebSocket server listening on: {}", websocket_addr);

    let notification_tx_clone = notification_tx.clone();
    tokio::spawn(async move {
        while let Ok((stream, _)) = websocket_listener.accept().await {
            let notification_tx = notification_tx_clone.clone();
            tokio::spawn(async move {
                let websocket = accept_async(stream)
                    .await
                    .expect("Failed to accept websocket");
                websocket::handle_websocket_connection(websocket, notification_tx).await;
            });
        }
    });

    let mut pg_listener = PgListener::connect_with(&db).await?;
    pg_listener.listen("user_follow").await?;
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

    // tokio::spawn(async move {
    //     notification_worker.run().await;
    // });
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    // cli::run_cli(&db).await?;

    Ok(())
}
