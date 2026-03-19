use axum::extract::Path;
use axum::{Json, extract::State, http::StatusCode};
use htc::id::build_id;
use htc::models::Entity;
use htc::regions::CrousRegion;
use tracing::error;
use htc::models::restaurants::{Restaurant, RestaurantSchema};
use htc::verifiable::SignedPayload;

use crate::{
    app::App,
    error::ApiError,
};

#[utoipa::path(
    put,
    path = "/{region}/restaurants",
    tag = "Restaurants",
    request_body = SignedPayload<Vec<RestaurantSchema>>,
    responses(
        (status = 201, description = "Restaurants created"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_restaurant<A>(
    Path(region): Path<String>,
    State(state): State<A>,
    Json(body): Json<SignedPayload<Vec<RestaurantSchema>>>,
) -> Result<StatusCode, ApiError>
where
    A: App + Send + Sync + Clone
{

    let region: CrousRegion = region.parse().map_err(|_| ApiError::NotFound(format!("Unknown region: {}", region)))?;
    let admin = state.get_admin(&body.author).await.map_err(|e| {
        error!("{}", e.to_string());
        ApiError::Unauthorized(e.to_string())
    })?;
    let user_key = admin.ssh_key;
    let (payload, _author) = body.verify(&user_key).map_err(|e| {
        error!("{}", e.to_string());
        ApiError::Unauthorized(e.to_string())
    })?;

    let batch = state.create_batch(Entity::Restaurants, admin.admin_id, region).await.map_err(|e| {
        error!("{}", e.to_string());
        ApiError::Unauthorized(e.to_string())
    })?;

    let restaurants: Vec<Restaurant> = payload
        .iter()
        .map(|schema| Restaurant {
            restaurant_id: build_id(&schema.name.clone()),
            name: schema.name.clone(),
            url: schema.url.clone(),
            city: schema.city.clone(),
            coordinates: schema.coordinates.clone(),
            opening_hours: schema.opening_hours.clone(),
            created_at: None,
            updated_at: None,
            batch_id: batch
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
