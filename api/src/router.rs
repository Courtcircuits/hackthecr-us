use std::sync::Arc;

use axum::Router;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::{
    app::App,
    error::ApiError,
    http::default_cors_layer,
    meals::{
        handlers::get_meals::__path_get_meals, handlers::put_meals::__path_put_meals,
        router::meals_router,
    },
    restaurants::{
        handlers::{
            get_restaurants::__path_get_restaurants, put_restaurants::__path_put_restaurant,
        },
        router::restaurants_router,
    },
    sse::{SseState, sse_router},
};

#[derive(OpenApi)]
#[openapi(
    info(title = "Hack The Crous API"),
    paths(put_restaurant, get_restaurants, put_meals, get_meals)
)]
pub struct ApiDoc;

pub async fn root<A>(app: A, sse_state: Arc<SseState>) -> Result<Router, ApiError>
where
    A: App + Send + Sync + Clone + 'static,
{
    let origins = app.clone().config().origins.clone();
    let openapi = ApiDoc::openapi();
    Ok(Router::new()
        .merge(Scalar::with_url("/docs", openapi))
        .merge(restaurants_router(app.clone()))
        .merge(meals_router(app))
        .merge(sse_router(sse_state))
        .layer(default_cors_layer(&origins)?)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        ))
}
