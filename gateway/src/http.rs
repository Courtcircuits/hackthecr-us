use std::sync::Arc;

use axum::{
    Router,
    http::{HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use tokio::{net::TcpListener, task::JoinHandle};
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::config::Config;

#[derive(thiserror::Error, Debug)]
pub enum ApiError {
    #[error("Internal server error: {0}")]
    InternalServerError(String),
    #[error("Not found: {0}")]
    NotFound(String),
}


impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
        };
        (status, body).into_response()
    }
}

pub async fn serve(app: Router, config: Arc<Config>) -> Result<JoinHandle<()>, ApiError> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port))
        .await
        .map_err(|e| {
            ApiError::InternalServerError(format!("Failed to bind to port {}: {}", config.port, e))
        })?;

    info!("Starting server on 0.0.0.0:{}", config.port);

    Ok(tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    }))
}

pub fn default_cors_layer(origins: &[String]) -> Result<CorsLayer, ApiError> {
    if origins.contains(&"*".to_string()) {
        return Ok(CorsLayer::permissive());
    }
    let origins = origins
        .iter()
        .map(|origin| {
            origin.parse::<HeaderValue>().map_err(|e| {
                ApiError::InternalServerError(format!("Invalid origin '{}': {}", origin, e))
            })
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
