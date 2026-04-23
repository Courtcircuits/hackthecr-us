use axum::{extract::State, http::StatusCode, Json};
use htc::{
    buffet::ScrapingResult,
    models::restaurants::RestaurantSchema,
    verifiable::SignedPayload,
};
use tracing::info;

use crate::{app::App, http::ApiError};

#[utoipa::path(
    put,
    path = "/restaurants",
    tag = "Restaurants",
    request_body = SignedPayload<ScrapingResult<Vec<RestaurantSchema>>>,
    responses(
        (status = 201, description = "Restaurants stored"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_restaurants(
    State(state): State<App>,
    Json(body): Json<SignedPayload<ScrapingResult<Vec<RestaurantSchema>>>>,
) -> Result<StatusCode, ApiError> {
    let order = &body.payload.order;
    info!(
        region = %order.target,
        date = %order.date,
        "Forwarding restaurants scraping result to Kafka"
    );
    state.produce_restaurants(body).await?;
    info!("Restaurants message forwarded successfully");
    Ok(StatusCode::CREATED)
}
