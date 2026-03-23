use std::{convert::Infallible, sync::Arc};

use axum::{
    Router,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{
        IntoResponse, Response,
        sse::{Event, Sse},
    },
    routing::get,
};
use htc::models::scrape_batch::ScrapeBatch;
use tokio::sync::broadcast;
use tokio_stream::{StreamExt, wrappers::BroadcastStream};

pub struct SseState {
    sender: broadcast::Sender<ScrapeBatch>,
    token: String,
}

impl SseState {
    pub fn new(token: String) -> (Self, broadcast::Sender<ScrapeBatch>) {
        let (sender, _) = broadcast::channel(100);
        (
            Self {
                sender: sender.clone(),
                token,
            },
            sender,
        )
    }
}

async fn sse_handler(State(state): State<Arc<SseState>>, headers: HeaderMap) -> Response {
    let authorized = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|t| t == state.token)
        .unwrap_or(false);

    if !authorized {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    let rx = state.sender.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(batch) => serde_json::to_string(&batch)
            .ok()
            .map(|json| Ok::<Event, Infallible>(Event::default().data(json))),
        Err(_) => None,
    });

    Sse::new(stream).into_response()
}

pub fn sse_router(state: Arc<SseState>) -> Router {
    Router::new()
        .route("/events", get(sse_handler))
        .with_state(state)
}
