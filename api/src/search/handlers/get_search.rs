use axum::{extract::{Path, Query, State}, Json};
use htc::{models::restaurants::RestaurantSchema, regions::CrousRegion};
use serde::Deserialize;

use crate::{app::App, error::ApiError};

#[derive(Deserialize)]
pub struct SearchParameters {
    q: String
}

#[utoipa::path(
    get,
    path = "/{region}/search",
    tag = "Search",
    responses(
        (status = 200, description = " List of associated restaurants", body = [Vec<RestaurantSchema>]),
        (status = 500, description = "Internal server error")
        )
    )]
pub async fn get_search<A>(
    Path(region): Path<String>,
    Query(parameters): Query<SearchParameters>,
    State(state): State<A>
) -> Result<Json<Vec<RestaurantSchema>>, ApiError> 
where A: App + Send + Sync + Clone,
{
    let region: CrousRegion = region
        .parse()
        .map_err(|_| ApiError::NotFound(format!("Unknown region: {}", region)))?;

    let restaurants = state.search_restaurant(parameters.q, region).await.map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let restaurants: Vec<RestaurantSchema> = restaurants
        .into_iter()
        .map(RestaurantSchema::from)
        .collect();
    Ok(Json(restaurants))
}
