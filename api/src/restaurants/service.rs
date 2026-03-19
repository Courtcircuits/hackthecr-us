use std::sync::Arc;

use htc::{models::{restaurants::{Restaurant, RestaurantModel as _, RestaurantModelError}, Entity}, regions::CrousRegion};
use sqlx::PgPool;

use crate::batches::service::{BatchesService, BatchesServiceImpl};

pub trait RestaurantsService {
    fn save_restaurants(&self, restaurants: Vec<Restaurant>) -> impl Future<Output = Result<(), RestaurantModelError>> + Send;
    fn get_restaurant_by_id(&self, id: String) -> impl Future<Output = Result<Restaurant, RestaurantModelError>> + Send;
    fn get_restaurants(&self, region: CrousRegion) -> impl Future<Output = Result<Vec<Restaurant>, RestaurantModelError>> + Send;
}

#[derive(Clone)]
pub struct RestaurantsServiceImpl<B>
where
    B: BatchesService
{
    pool: Arc<PgPool>,
    batch_service: Arc<B>
}

impl RestaurantsService for RestaurantsServiceImpl<BatchesServiceImpl> {
    async fn save_restaurants(&self, restaurants: Vec<Restaurant>) -> Result<(), RestaurantModelError> {
        for restaurant in restaurants {
            self.pool.create_restaurant(restaurant).await?;
        }
        Ok(())
    }

    async fn get_restaurant_by_id(&self, id: String) -> Result<Restaurant, RestaurantModelError> {
        self.pool.get_restaurant_by_id(id).await
    }

    async fn get_restaurants(&self, region: CrousRegion) -> Result<Vec<Restaurant>, RestaurantModelError> {
        let current_batch = self
            .batch_service
            .current_batch(Entity::Restaurants, region)
            .await
            .map_err(|_| RestaurantModelError::NotFound)?;
        println!("current_batch {}", current_batch);
        self.pool.get_all_restaurants_batch(current_batch).await
    }
}

impl<B> RestaurantsServiceImpl<B> 
where
    B: BatchesService
{
    pub fn new(pool: Arc<PgPool>, batch_service: Arc<B>) -> Self {
        Self { pool, batch_service }
    }
}
