use std::future::Future;

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::Uuid;
use utoipa::ToSchema;

#[derive(Debug, Clone, ToSchema, Serialize, Deserialize)]
pub struct MealSchema {
    pub meal_type: String,
    pub foodies: Option<String>,
    pub date: Option<String>,
    pub restaurant_id: String,
}

#[derive(Clone)]
pub struct Meal {
    pub meal_id: Uuid,
    pub meal_type: String,
    pub foodies: Option<String>,
    pub date: Option<String>,
    pub batch_id: Uuid,
    pub restaurant_id: String,
}

impl From<Meal> for MealSchema {
    fn from(meal: Meal) -> Self {
        MealSchema {
            meal_type: meal.meal_type,
            foodies: meal.foodies,
            date: meal.date,
            restaurant_id: meal.restaurant_id.to_string(),
        }
    }
}

impl From<&Meal> for MealSchema {
    fn from(meal: &Meal) -> Self {
        MealSchema {
            meal_type: meal.meal_type.clone(),
            foodies: meal.foodies.clone(),
            date: meal.date.clone(),
            restaurant_id: meal.restaurant_id.to_string(),
        }
    }
}

#[derive(Debug)]
pub enum MealModelError {
    NotFound,
    DatabaseError(String),
}

impl std::fmt::Display for MealModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MealModelError::NotFound => write!(f, "Meal not found"),
            MealModelError::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

pub trait MealModel {
    fn create_meal(&self, meal: Meal) -> impl Future<Output = Result<(), MealModelError>> + Send;
    fn get_meals_by_restaurant_id_batch(&self, restaurant_name: String, batch_id: Uuid) -> impl Future<Output = Result<Vec<Meal>, MealModelError>> + Send;
}

impl MealModel for PgPool {
    async fn create_meal(&self, meal: Meal) -> Result<(), MealModelError> {
        sqlx::query!(
            "INSERT INTO meals (meal_id, meal_type, foodies, date, restaurant_id, batch_id) VALUES ($1, $2, $3, $4, $5, $6)",
            meal.meal_id,
            meal.meal_type,
            meal.foodies,
            meal.date,
            meal.restaurant_id,
            meal.batch_id
        )
        .execute(self)
        .await
        .map_err(|e| MealModelError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_meals_by_restaurant_id_batch(&self, restaurant_name: String, batch_id: Uuid) -> Result<Vec<Meal>, MealModelError> where Self: Sync {
        let rows = sqlx::query!(
            "SELECT m.meal_id, m.meal_type, m.foodies, m.date, m.restaurant_id, m.batch_id FROM meals m WHERE m.restaurant_id = $1 AND m.batch_id = $2",
            restaurant_name,
            batch_id
        )
        .fetch_all(self)
        .await
        .map_err(|e| MealModelError::DatabaseError(e.to_string()))?;

        let meals = rows
            .into_iter()
            .map(|row| Meal {
                meal_id: row.meal_id,
                meal_type: row.meal_type,
                foodies: row.foodies,
                batch_id: row.batch_id,
                date: row.date,
                restaurant_id: row.restaurant_id,
            })
            .collect();

        Ok(meals)
    }
}
