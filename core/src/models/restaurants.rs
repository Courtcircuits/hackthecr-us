use std::future::Future;

use chrono::NaiveDateTime;
use sqlx::PgPool;
use sqlx::types::Uuid;

#[derive(Clone)]
pub struct Restaurant {
    pub restaurant_id: Uuid,
    pub name: String,
    pub url: String,
    pub coordinates: Option<String>,
    pub opening_hours: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

pub enum RestaurantModelError {
    NotFound,
    DatabaseError(String),
}

pub trait RestaurantModel {
    fn create_restaurant(&self, restaurant: Restaurant) -> impl Future<Output = Result<(), RestaurantModelError>> + Send;
    fn get_restaurant_by_name(&self, name: String) -> impl Future<Output = Result<Restaurant, RestaurantModelError>> + Send;
    fn get_all_restaurants(&self) -> impl Future<Output = Result<Vec<Restaurant>, RestaurantModelError>> + Send;
}

impl RestaurantModel for PgPool {
    async fn create_restaurant(&self, restaurant: Restaurant) -> Result<(), RestaurantModelError> {
        sqlx::query!(
            "INSERT INTO restaurants (restaurant_id, name, url, coordinates, opening_hours) VALUES ($1, $2, $3, $4, $5)",
            restaurant.restaurant_id,
            restaurant.name,
            restaurant.url,
            restaurant.coordinates,
            restaurant.opening_hours
        )
        .execute(self)
        .await
        .map_err(|e| RestaurantModelError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_restaurant_by_name(&self, name: String) -> Result<Restaurant, RestaurantModelError> {
        let row = sqlx::query!(
            "SELECT restaurant_id, name, url, coordinates, opening_hours, created_at, updated_at FROM restaurants WHERE name = $1",
            name
        )
        .fetch_optional(self)
        .await
        .map_err(|e| RestaurantModelError::DatabaseError(e.to_string()))?
        .ok_or(RestaurantModelError::NotFound)?;

        Ok(Restaurant {
            restaurant_id: row.restaurant_id,
            name: row.name,
            url: row.url,
            coordinates: row.coordinates,
            opening_hours: row.opening_hours,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    async fn get_all_restaurants(&self) -> Result<Vec<Restaurant>, RestaurantModelError> {
        let rows = sqlx::query!(
            "SELECT restaurant_id, name, url, coordinates, opening_hours, created_at, updated_at FROM restaurants"
        )
        .fetch_all(self)
        .await
        .map_err(|e| RestaurantModelError::DatabaseError(e.to_string()))?;

        let restaurants = rows
            .into_iter()
            .map(|row| Restaurant {
                restaurant_id: row.restaurant_id,
                name: row.name,
                url: row.url,
                coordinates: row.coordinates,
                opening_hours: row.opening_hours,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(restaurants)
    }
}
