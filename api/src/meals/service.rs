use std::sync::Arc;

use htc::{models::{
    meals::{Meal, MealModel as _, MealModelError}, Entity
}, regions::CrousRegion};
use sqlx::PgPool;

use crate::batches::service::{BatchesService, BatchesServiceImpl};

pub trait MealsService {
    fn save_meals(
        &self,
        meals: Vec<Meal>,
    ) -> impl Future<Output = Result<(), MealModelError>> + Send;
    fn get_meals_by_restaurant_id(
        &self,
        name: String,
        region: CrousRegion
    ) -> impl Future<Output = Result<Vec<Meal>, MealModelError>> + Send;
}

#[derive(Clone)]
pub struct MealsServiceImpl<B>
where
    B: BatchesService,
{
    pool: Arc<PgPool>,
    batch_service: Arc<B>,
}

impl MealsService for MealsServiceImpl<BatchesServiceImpl> {
    async fn save_meals(&self, meals: Vec<Meal>) -> Result<(), MealModelError> {
        for meal in meals {
            self.pool.create_meal(meal).await?;
        }
        Ok(())
    }

    async fn get_meals_by_restaurant_id(&self, name: String, region: CrousRegion) -> Result<Vec<Meal>, MealModelError> {
        let current_batch = self
            .batch_service
            .current_batch(Entity::Meals(name.to_string()), region)
            .await
            .map_err(|_| MealModelError::NotFound)?;
        self.pool.get_meals_by_restaurant_id_batch(name, current_batch).await
    }
}

impl<B> MealsServiceImpl<B>
where
    B: BatchesService,
{
    pub fn new(pool: Arc<PgPool>, batch_service: Arc<B>) -> Self {
        Self {
            pool,
            batch_service,
        }
    }
}
