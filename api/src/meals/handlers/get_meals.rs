use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, State},
};
use htc::models::meals::MealSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{app::App, error::ApiError};

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct MenuSchema {
    date: String,
    meals: Vec<MenuSectionSchema>,
}

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct MenuSectionSchema {
    meal_type: String,
    foods: Vec<String>,
}

impl From<Vec<MealSchema>> for MenuSchema {
    fn from(value: Vec<MealSchema>) -> Self {
        let date = value
            .first()
            .and_then(|m| m.date.clone())
            .unwrap_or_default();

        let mut sections: HashMap<String, Vec<String>> = HashMap::new();
        for meal in value {
            let food = meal.foodies.unwrap_or_default();
            sections
                .entry(meal.meal_type)
                .or_default()
                .push(food);
        }

        let meals = sections
            .into_iter()
            .map(|(meal_type, foods)| MenuSectionSchema { meal_type, foods })
            .collect();

        MenuSchema { date, meals }
    }
}

#[utoipa::path(
    get,
    path = "/meals/{name}",
    params(
        ("name" = String, Path, description = "Restaurant name")
    ),
    tag = "Meals",
    responses(
        (status = 200, description = "List of meals for particular restaurant", body = [Vec<MenuSchema>]),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_meals<A>(
    Path(name): Path<String>,
    State(state): State<A>,
) -> Result<Json<Vec<MenuSchema>>, ApiError>
where
    A: App + Send + Sync + Clone,
{
    let meals = state
        .get_meals_by_restaurant_id(name)
        .await
        .map_err(|e| ApiError::InternalServerError(e.to_string()))?;

    let meal_schemas: Vec<MealSchema> = meals.into_iter().map(&MealSchema::from).collect();

    let mut by_date: HashMap<String, Vec<MealSchema>> = HashMap::new();
    for meal in meal_schemas {
        let date = meal.date.clone().unwrap_or_default();
        by_date.entry(date).or_default().push(meal);
    }

    let menus: Vec<MenuSchema> = by_date.into_values().map(MenuSchema::from).collect();
    Ok(Json(menus))
}
