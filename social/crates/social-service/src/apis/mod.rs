use std::sync::Arc;

use axum::middleware;
use axum::Router;
use middlewares::security::verify_api_key;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_scalar::{Scalar, Servable};

use crate::AppState;

pub mod api_models;
pub mod group_handlers;
pub mod middlewares;
pub mod profile_handlers;
pub mod token_handlers;
pub mod user_handlers;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "users", description = "User management API"),
        (name = "token-picks", description = "Token pick management API"),
        (name = "groups", description = "Group management API"),
        (name = "profiles", description = "Profile management API")
    ),
    modifiers(&SecurityAddon),
    components(
        schemas()
    ),
    security(
        ("api_key" = [])
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(
                        utoipa::openapi::security::ApiKeyValue::new("X-API-Key"),
                    ),
                ),
            );
        }
    }
}

pub fn setup_routes() -> Router<Arc<AppState>> {
    let api_doc = ApiDoc::openapi();
    let token_router = OpenApiRouter::new()
        .routes(routes!(
            token_handlers::list_token_picks,
            token_handlers::create_token_pick,
            token_handlers::delete_token_pick
        ))
        .routes(routes!(token_handlers::list_group_token_picks));

    let profile_router = OpenApiRouter::new()
        .routes(routes!(profile_handlers::get_profile))
        .routes(routes!(profile_handlers::get_profile_picks_and_stats))
        .routes(routes!(profile_handlers::leaderboard));

    let user_router = OpenApiRouter::new()
        .routes(routes!(user_handlers::follow_user))
        .routes(routes!(user_handlers::unfollow_user))
        .routes(routes!(user_handlers::get_followers))
        .routes(routes!(user_handlers::upload_avatar));

    let group_router = OpenApiRouter::new()
        .routes(routes!(group_handlers::list_groups))
        .routes(routes!(
            group_handlers::get_group,
            group_handlers::create_or_update_group
        ))
        .routes(routes!(
            group_handlers::add_user_to_group,
            group_handlers::remove_user_from_group
        ))
        .routes(routes!(group_handlers::get_group_members))
        .routes(routes!(group_handlers::get_group_picks))
        .routes(routes!(group_handlers::get_group_leaderboard))
        .routes(routes!(group_handlers::leaderboard));
    let user_router = OpenApiRouter::with_openapi(api_doc.clone()).nest("/users", user_router);

    let profile_router =
        OpenApiRouter::with_openapi(api_doc.clone()).nest("/profiles", profile_router);

    let token_router = OpenApiRouter::with_openapi(api_doc.clone()).nest("/tokens", token_router);

    let group_router = OpenApiRouter::with_openapi(api_doc.clone()).nest("/groups", group_router);

    let router = OpenApiRouter::new()
        .merge(user_router)
        .merge(profile_router)
        .merge(token_router)
        .merge(group_router);

    let (api_router, api_openapi) = OpenApiRouter::new()
        .nest("/api/v1", router)
        .split_for_parts();

    Router::new()
        .merge(api_router)
        .route_layer(middleware::from_fn(verify_api_key))
        .merge(Scalar::with_url("/docs", api_openapi))
}
