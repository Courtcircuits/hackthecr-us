use axum::{Router, routing::{get, put}};

use crate::{app::App, restaurants::handlers::{get_restaurants::get_restaurants, put_restaurants::put_restaurant}};

pub fn restaurants_router<A>(app: A) -> Router
where
    A: App + Send + Sync + Clone + 'static
{
    Router::new()
        .route("/restaurants", get(get_restaurants::<A>))
        .route("/restaurants", put(put_restaurant::<A>))
        .with_state(app)
}
