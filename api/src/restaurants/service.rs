use std::sync::Arc;

use htc_core::models::restaurants::{Restaurant, RestaurantModel as _, RestaurantModelError};
use sqlx::PgPool;

pub trait RestaurantsServices {
    fn save_restaurants(&self, restaurants: Vec<Restaurant>) -> impl Future<Output = Result<(), RestaurantModelError>> + Send;
    fn get_restaurant_by_name(&self, name: String) -> impl Future<Output = Result<Restaurant, RestaurantModelError>> + Send;
    fn get_restaurants(&self) -> impl Future<Output = Result<Vec<Restaurant>, RestaurantModelError>> + Send;
}

#[derive(Clone)]
pub struct RestaurantsServiceImpl {
    pool: Arc<PgPool>
}

impl RestaurantsServices for RestaurantsServiceImpl {
    async fn save_restaurants(&self, restaurants: Vec<Restaurant>) -> Result<(), RestaurantModelError> {
        for restaurant in restaurants {
            self.pool.create_restaurant(restaurant).await?;
        }
        Ok(())
    }

    async fn get_restaurant_by_name(&self, name: String) -> Result<Restaurant, RestaurantModelError> {
        self.pool.get_restaurant_by_name(name).await
    }

    async fn get_restaurants(&self) -> Result<Vec<Restaurant>, RestaurantModelError> {
        self.pool.get_all_restaurants().await
    }
}

impl RestaurantsServiceImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}
