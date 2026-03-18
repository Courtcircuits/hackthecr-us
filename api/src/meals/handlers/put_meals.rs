use axum::{extract::State, Json};
use htc::{models::{meals::{Meal, MealSchema}, Entity}, verifiable::SignedPayload};
use reqwest::StatusCode;
use tracing::error;
use uuid::Uuid;

use crate::{app::App, error::ApiError};

#[utoipa::path(
    put,
    path = "/meals",
    tag = "Meals",
    request_body = SignedPayload<Vec<MealSchema>>,
    responses(
        (status = 201, description = "Meals created"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_meals<A>(
    State(state): State<A>,
    Json(body): Json<SignedPayload<Vec<MealSchema>>>,
) -> Result<StatusCode, ApiError>
where
    A: App + Send + Sync + Clone
{
    let admin = state.get_admin(&body.author).await.map_err(|e| {
        error!("{}", e.to_string());
        ApiError::Unauthorized(e.to_string())
    })?;
    let user_key = admin.ssh_key;
    let (payload, _author) = body.verify(&user_key).map_err(|e| {
        error!("{}", e.to_string());
        ApiError::Unauthorized(e.to_string())
    })?;

    let Some(first_meal) = payload.get(0) else {
      return Err(ApiError::BadRequest("Empty request".to_string()));
    };

    let restaurant_id = first_meal.restaurant_id.clone();

    let batch = state.create_batch(Entity::Meals(restaurant_id), admin.admin_id).await.map_err(|e| {
        error!("{}", e.to_string());
        ApiError::Unauthorized(e.to_string())
    })?;

    let meals: Vec<Meal> = payload
        .iter()
        .map(|schema| {
            Meal {
                meal_id: Uuid::new_v4(),
                meal_type: schema.meal_type.clone(),
                foodies: schema.foodies.clone(),
                date: schema.date.clone(),
                restaurant_id: schema.restaurant_id.clone(),
                batch_id: batch
            }
        })
        .collect();

    state.save_meals(meals).await.map_err(|e| {
        error!("{}", e.to_string());
        ApiError::InternalServerError(e.to_string())
    })?;

    Ok(StatusCode::CREATED)
}
