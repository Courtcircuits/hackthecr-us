use axum::{routing::put, Router};

use crate::{app::App, restaurants::put_restaurants};

pub fn restaurants_router(app: App) -> Router {
    Router::new()
        .route("/restaurants", put(put_restaurants::put_restaurants))
        .with_state(app)
}
