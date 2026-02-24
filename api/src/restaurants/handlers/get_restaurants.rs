use axum::{extract::State, http::Response};

use crate::{app::App, error::ApiError, restaurants::RestaurantSchema};

#[utoipa::path(
    get,
    path = "/restaurants",
    tag = "Restaurants",
    responses(
        (status = 200, description = "List of restaurants", body = [Vec<RestaurantSchema>]),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_restaurants<A>
(
    State(state): State<A>,
    ) -> Result<Response<Vec<RestaurantSchema>>, ApiError> 
where A: App + Send + Sync + Clone
{
    let restaurants = state.get_restaurants().await.map_err(|e| ApiError::InternalServerError(e.to_string()))?;
    Ok(Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .body(restaurants.into_iter().map(RestaurantSchema::from).collect())
        .unwrap())

}
