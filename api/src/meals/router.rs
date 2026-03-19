use axum::{routing::{get, put}, Router};

use crate::{app::App, meals::handlers::{get_meals::get_meals, put_meals::put_meals}};

pub fn meals_router<A>(app: A) -> Router
where
    A: App + Send + Sync + Clone + 'static
{
    Router::new()
        .route("/{region}/meals", put(put_meals::<A>))
        .route("/{region}/meals/{name}", get(get_meals::<A>))
        .with_state(app)
}
