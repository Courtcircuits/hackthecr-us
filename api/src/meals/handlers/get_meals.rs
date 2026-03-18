use axum::{
    Json,
    extract::{Path, State},
};
use htc::models::meals::MealSchema;

use crate::{app::App, error::ApiError};

#[utoipa::path(
    get,
    path = "/meals/{name}",
    params(
        ("name" = String, Path, description = "Restaurant name")
    ),
    tag = "Meals",
    request_body = Vec<MealSchema>,
    responses(
        (status = 200, description ="List of meals for particular restaurant", body = [Vec<MealSchema>]),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_meals<A>(
    Path(name): Path<String>,
    State(state): State<A>,
) -> Result<Json<Vec<MealSchema>>, ApiError>
where
    A: App + Send + Sync + Clone,
{
    let meals = state
        .get_meals_by_restaurant_id(name)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let meals: Vec<MealSchema> = meals.into_iter().map(&MealSchema::from).collect();
    Ok(Json(meals))
}
