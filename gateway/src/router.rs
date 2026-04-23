use axum::Router;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::{
    app::App,
    http::{ApiError, default_cors_layer},
    jobs::{poll_job::__path_get_next_job, router::jobs_router},
    meals::{put_meals::__path_put_meals, router::meals_router},
    restaurants::{put_restaurants::__path_put_restaurants, router::restaurants_router},
};

#[derive(OpenApi)]
#[openapi(
    info(title = "Hack The Crous Gateway"),
    paths(get_next_job, put_meals, put_restaurants)
)]
pub struct ApiDoc;

pub fn root(app: App, cors_origins: &[String]) -> Result<Router, ApiError> {
    let openapi = ApiDoc::openapi();
    Ok(Router::new()
        .merge(Scalar::with_url("/docs", openapi))
        .merge(jobs_router(app.clone()))
        .merge(meals_router(app.clone()))
        .merge(restaurants_router(app))
        .layer(default_cors_layer(cors_origins)?)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        ))
}
