use axum::{
    Json,
    extract::{Path, State},
};
use htc::{models::meals::MealSchema, regions::CrousRegion, verifiable::SignedPayload};
use reqwest::StatusCode;
use tracing::error;

use crate::{app::App, error::ApiError};

#[utoipa::path(
    put,
    path = "/{region}/meals",
    tag = "Meals",
    request_body = SignedPayload<Vec<MealSchema>>,
    responses(
        (status = 201, description = "Meals created"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_meals<A>(
    Path(region): Path<String>,
    State(state): State<A>,
    Json(body): Json<SignedPayload<Vec<MealSchema>>>,
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
    let user_key = admin.ssh_key.clone();
    let (payload, digest) = body.verify(&user_key).map_err(|e| {
        error!("{}", e.to_string());
        ApiError::Unauthorized(e.to_string())
    })?;

    state
        .save_meals(payload, admin, region, digest)
        .await
        .map_err(|e| {
            error!("{}", e.to_string());
            ApiError::InternalServerError(e.to_string())
        })?;

    Ok(StatusCode::CREATED)
}
