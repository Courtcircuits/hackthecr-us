use axum::extract::Path;
use axum::{Json, extract::State, http::StatusCode};
use htc::models::restaurants::RestaurantSchema;
use htc::regions::CrousRegion;
use htc::verifiable::SignedPayload;
use tracing::error;

use crate::{app::App, error::ApiError};

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
    A: App + Send + Sync + Clone,
{
    let region: CrousRegion = region
        .parse()
        .map_err(|_| ApiError::NotFound(format!("Unknown region: {}", region)))?;
    let admin = state.get_admin(&body.author).await.map_err(|e| {
        error!("{}", e.to_string());
        ApiError::Unauthorized(e.to_string())
    })?;
    let (payload, digest) = body.verify(admin.ssh_key.as_str()).map_err(|e| {
        error!("{}", e.to_string());
        ApiError::Unauthorized(e.to_string())
    })?;

    state
        .save_restaurants(payload, admin, region, digest)
        .await
        .map_err(|e| {
            error!("{}", e.to_string());
            match e {
                htc::models::restaurants::RestaurantModelError::NotFound => {
                    ApiError::NotFound("No restaurant found".to_string())
                }
                htc::models::restaurants::RestaurantModelError::DatabaseError(ie) => {
                    ApiError::InternalServerError(ie)
                }
                htc::models::restaurants::RestaurantModelError::SyncSkipped => ApiError::Conflict,
            }
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
pub async fn get_healthcheck_handler() -> Result<String, ApiError> {
    Ok("Test".to_string())
}
