use std::sync::Arc;

use axum::Router;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

use crate::AppState;

pub mod profile_handlers;
pub mod user_handlers;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "users", description = "User management API")
    )
)]

pub struct ApiDoc;

pub fn setup_routes() -> Router<Arc<AppState>> {
    let api_doc = ApiDoc::openapi();

    let profile_router = OpenApiRouter::new()
        .routes(routes!(profile_handlers::get_profile_details))
        .routes(routes!(profile_handlers::get_user_stats))
        .routes(routes!(profile_handlers::get_user_picks));

    let user_router = OpenApiRouter::new()
        .routes(routes!(user_handlers::follow_user))
        .routes(routes!(user_handlers::unfollow_user));

    let user_router = OpenApiRouter::with_openapi(api_doc.clone()).nest("/users", user_router);

    let profile_router =
        OpenApiRouter::with_openapi(api_doc.clone()).nest("/profiles", profile_router);

    let router = OpenApiRouter::new()
        .merge(user_router)
        .merge(profile_router);

    let (api_router, api_openapi) = OpenApiRouter::new()
        .nest("/api/v1", router)
        .split_for_parts();

    Router::new()
        .merge(Scalar::with_url("/docs", api_openapi))
        .merge(api_router)
}

// fn setup_user_routes() {
//     let router = Router::new().route("/{id}/follow/{follower_id}", user_handl);
// }
