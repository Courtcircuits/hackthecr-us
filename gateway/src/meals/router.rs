use axum::{routing::put, Router};

use crate::{app::App, meals::put_meals};

pub fn meals_router(app: App) -> Router {
    Router::new()
        .route("/meals", put(put_meals::put_meals))
        .with_state(app)
}
