use axum::{Json, extract::State};
use htc::orders::Order;

use crate::{app::App, http::ApiError};

#[utoipa::path(
    post,
    path = "/jobs/next",
    tag = "Jobs",
    responses(
        (status = 200, description = "Next job to process", body = Order),
        (status = 404, description = "No jobs in the queue"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_next_job(State(state): State<App>) -> Result<Json<Order>, ApiError> {
    let order = state.poll_job().await?;
    if let Some(order) = order {
        Ok(Json(order))
    } else {
        Err(ApiError::NotFound("No jobs in the queue".to_string()))
    }
}
