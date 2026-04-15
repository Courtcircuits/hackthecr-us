use axum::{Router, routing::get};

use crate::{app::App, search::handlers::get_search::get_search};

pub fn search_router<A>(app: A) -> Router
where
    A: App + Send + Sync + Clone + 'static,
{
    Router::new()
        .route("/{region}/search", get(get_search::<A>))
        .with_state(app)
}
