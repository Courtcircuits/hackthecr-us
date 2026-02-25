use axum::{Json, extract::State, http::StatusCode};
use tracing::error;
use uuid::Uuid;
use htc_core::models::restaurants::{Restaurant, RestaurantSchema};

use crate::{
    app::App,
    error::ApiError,
};

#[utoipa::path(
    put,
    path = "/restaurants",
    tag = "Restaurants",
    request_body = RestaurantSchema,
    responses(
        (status = 201, description = "Restaurants created"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_restaurant<A>(
    State(state): State<A>,
    Json(body): Json<Vec<RestaurantSchema>>,
) -> Result<StatusCode, ApiError>
where
    A: App + Send + Sync + Clone
{
    let restaurants: Vec<Restaurant> = body
        .into_iter()
        .map(|schema| Restaurant {
            restaurant_id: Uuid::new_v4(),
            name: schema.name,
            url: schema.url,
            city: schema.city,
            coordinates: schema.coordinates,
            opening_hours: schema.opening_hours,
            created_at: None,
            updated_at: None,
        })
        .collect();
    state
        .save_restaurants(restaurants)
        .await
        .map_err(|e| {
            error!("{}", e.to_string());
            ApiError::InternalServerError(e.to_string())
        })?;
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "healthcheck",
    responses(
        (status = 200, description = "Health check successful", body = String),
        (status = 500, description = "Internal server error", body = ApiError)
    )
)]
pub async fn get_healthcheck_handler(
) -> Result<String, ApiError> {
    Ok("Test".to_string())
}
