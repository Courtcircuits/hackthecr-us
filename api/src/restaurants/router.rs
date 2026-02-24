use axum::{Router, routing::{get, put}};

use crate::{app::App, restaurants::handlers::get_restaurants::get_restaurants};

pub fn restaurants_router<A>(app: A) -> Router 
where A: App + Send + Sync + Clone
{
    Router::new()
        .route("/restaurants", get(get_restaurants))
        .with_state(app)
}
