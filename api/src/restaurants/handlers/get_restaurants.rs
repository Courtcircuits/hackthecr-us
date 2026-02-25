use axum::{Json, extract::State};
use htc_core::models::restaurants::RestaurantSchema;

use crate::{
    app::App,
    error::ApiError,
};

#[utoipa::path(
    get,
    path = "/restaurants",
    tag = "Restaurants",
    responses(
        (status = 200, description = "List of restaurants", body = [Vec<RestaurantSchema>]),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_restaurants<A>(
    State(state): State<A>,
) -> Result<Json<Vec<RestaurantSchema>>, ApiError> 
where 
    A: App + Send + Sync + Clone
{
    let restaurants = state
        .get_restaurants()
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    let restaurants: Vec<RestaurantSchema> = restaurants
        .into_iter()
        .map(&RestaurantSchema::from)
        .collect();
    Ok(Json(restaurants))
}
