use axum::{extract::State, http::StatusCode, Json};
use htc::{
    buffet::ScrapingResult,
    models::meals::MealSchema,
    verifiable::SignedPayload,
};

use tracing::info;

use crate::{app::App, http::ApiError};

#[utoipa::path(
    put,
    path = "/meals",
    tag = "Meals",
    request_body = SignedPayload<ScrapingResult<Vec<MealSchema>>>,
    responses(
        (status = 201, description = "Meals stored"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_meals(
    State(state): State<App>,
    Json(body): Json<SignedPayload<ScrapingResult<Vec<MealSchema>>>>,
) -> Result<StatusCode, ApiError> {
    let order = &body.payload.order;
    info!(
        region = %order.target,
        date = %order.date,
        "Forwarding meals scraping result to Kafka"
    );
    state.produce_meals(body).await?;
    info!("Meals message forwarded successfully");
    Ok(StatusCode::CREATED)
}
