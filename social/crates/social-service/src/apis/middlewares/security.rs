use axum::{
    body::Body,
    http::{header::HeaderName, Request},
    middleware::Next,
    response::Response,
};
use once_cell::sync::Lazy;
use std::env;
use tracing::warn;

use crate::utils::errors::app_error::AppError;

pub static API_KEY_HEADER: Lazy<HeaderName> = Lazy::new(|| HeaderName::from_static("x-api-key"));
static API_KEY: Lazy<String> = Lazy::new(|| {
    env::var("API_KEY").unwrap_or_else(|_| {
        warn!("API_KEY not found in environment, using default key");
        "sk_live_51O6yZvBWRZBNXxT8k2mXpEGBj7D8XPDQPGQmsDZAEGBNXxT8".to_string()
    })
});

pub async fn verify_api_key(request: Request<Body>, next: Next) -> Result<Response, AppError> {
    let api_key = request
        .headers()
        .get(API_KEY_HEADER.as_str())
        .and_then(|header| header.to_str().ok());

    match api_key {
        Some(key) if key == API_KEY.as_str() => Ok(next.run(request).await),
        Some(_) => Err(AppError::Unauthorized("Invalid API key".to_string())),
        None => Err(AppError::Unauthorized("Missing API key".to_string())),
    }
}
