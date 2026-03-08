use std::future::Future;

use chrono::NaiveDateTime;
use sqlx::PgPool;
use sqlx::types::Uuid;

pub struct Meal {
    pub meal_id: Uuid,
    pub meal_type: String,
    pub foodies: Option<String>,
    pub date: Option<String>,
    pub scraped_at: Option<NaiveDateTime>,
    pub restaurant_id: Uuid,
}

pub enum MealModelError {
    NotFound,
    DatabaseError(String),
}

pub trait MealModel {
    fn create_meal(&self, meal: Meal) -> impl Future<Output = Result<(), MealModelError>> + Send;
    fn get_meals_by_restaurant_id(&self, restaurant_name: String) -> impl Future<Output = Result<Vec<Meal>, MealModelError>> + Send;
}

impl MealModel for PgPool {
    async fn create_meal(&self, meal: Meal) -> Result<(), MealModelError> {
        sqlx::query!(
            "INSERT INTO meals (meal_id, meal_type, foodies, date, restaurant_id) VALUES ($1, $2, $3, $4, $5)",
            meal.meal_id,
            meal.meal_type,
            meal.foodies,
            meal.date,
            meal.restaurant_id
        )
        .execute(self)
        .await
        .map_err(|e| MealModelError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_meals_by_restaurant_id(&self, restaurant_name: String) -> Result<Vec<Meal>, MealModelError> {
        let rows = sqlx::query!(
            "SELECT m.meal_id, m.meal_type, m.foodies, m.date, m.scraped_at, m.restaurant_id FROM meals m INNER JOIN restaurants r ON m.restaurant_id = r.restaurant_id WHERE r.name = $1",
            restaurant_name
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
                scraped_at: row.scraped_at,
                date: row.date,
                restaurant_id: row.restaurant_id,
            })
            .collect();

        Ok(meals)
    }
}
