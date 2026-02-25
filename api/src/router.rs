use axum::Router;
use tower_http::trace::TraceLayer;

use crate::{app::App, error::ApiError, http::default_cors_layer, restaurants::router::restaurants_router};

pub async fn root<A>(app: A) -> Result<Router, ApiError> 
where A: App + Send + Sync + Clone + 'static
{
    let origins = app.clone().config().origins.clone();
    Ok(Router::new()
        .merge(restaurants_router(app))
        .layer(default_cors_layer(&origins)?)
        .layer(TraceLayer::new_for_http()))
}
