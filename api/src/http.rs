use std::sync::Arc;

use axum::{
    Router,
    http::{HeaderValue, Method, header},
};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::{config::Config, error::ApiError};

pub async fn serve(app: Router, config: Arc<Config>) -> Result<(), ApiError> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Failed to bind to port {}: {}", config.port, e)))?;

    info!("Starting server on 0.0.0.0:{}", config.port);
    axum::serve(listener, app)
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Server error: {}", e)))?;
    Ok(())
}

pub fn default_cors_layer(origins: &[String]) -> Result<CorsLayer, ApiError> {
    if origins.contains(&"*".to_string()) {
        return Ok(CorsLayer::permissive());
    }
    let origins = origins
        .iter()
        .map(|origin| {
            origin
                .parse::<HeaderValue>()
                .map_err(|e| ApiError::InternalServerError(format!("Invalid origin '{}': {}", origin, e)))
        })
        .collect::<Result<Vec<HeaderValue>, ApiError>>()?;

    Ok(CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
            header::ORIGIN,
        ])
        .allow_origin(origins))
}
