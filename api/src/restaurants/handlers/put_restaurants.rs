use axum::{Json, extract::State, http::StatusCode};
use uuid::Uuid;
use htc_core::models::restaurants::Restaurant;

use crate::{
    app::App,
    error::ApiError,
    restaurants::RestaurantSchema,
};

#[utoipa::path(
    put,
    path = "/restaurants",
    tag = "Restaurants",
    request_body = Vec<RestaurantSchema>,
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
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    Ok(StatusCode::CREATED)
}
