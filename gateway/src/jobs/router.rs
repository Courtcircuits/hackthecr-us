use axum::{routing::post, Router};

use crate::{app::App, jobs::poll_job};


pub fn jobs_router(app: App) -> Router
{
    Router::new()
        .route("/jobs/next", post(poll_job::get_next_job))
        .with_state(app)
}
