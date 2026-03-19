use axum::{
    Json,
    extract::{Path, State},
};
use htc::{models::restaurants::RestaurantSchema, regions::CrousRegion};

use crate::{app::App, error::ApiError};

#[utoipa::path(
    get,
    path = "/{region}/restaurants",
    tag = "Restaurants",
    responses(
        (status = 200, description = "List of restaurants", body = [Vec<RestaurantSchema>]),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_restaurants<A>(
    Path(region): Path<String>,
    State(state): State<A>,
) -> Result<Json<Vec<RestaurantSchema>>, ApiError>
where
    A: App + Send + Sync + Clone,
{
    let region: CrousRegion = region
        .parse()
        .map_err(|_| ApiError::NotFound(format!("Unknown region: {}", region)))?;
    let restaurants = state
        .get_restaurants(region)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    let restaurants: Vec<RestaurantSchema> = restaurants
        .into_iter()
        .map(&RestaurantSchema::from)
        .collect();
    Ok(Json(restaurants))
}
