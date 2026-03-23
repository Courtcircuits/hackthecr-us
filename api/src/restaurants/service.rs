use std::future::Future;
use std::sync::Arc;

use htc::{
    id::build_id,
    models::{
        Entity,
        admins::Admin,
        restaurants::{Restaurant, RestaurantModel as _, RestaurantModelError, RestaurantSchema},
        scrape_batch::ScrapedBatchModelError,
    },
    regions::CrousRegion,
};
use sqlx::PgPool;
use tracing::instrument;

use crate::batches::service::{BatchesService, BatchesServiceImpl};

pub trait RestaurantsService {
    fn save_restaurants(
        &self,
        restaurants: &[RestaurantSchema],
        admin: Admin,
        region: CrousRegion,
        checksum: String,
    ) -> impl Future<Output = Result<(), RestaurantModelError>> + Send;
    fn get_restaurant_by_id(
        &self,
        id: String,
    ) -> impl Future<Output = Result<Restaurant, RestaurantModelError>> + Send;
    fn get_restaurants(
        &self,
        region: CrousRegion,
    ) -> impl Future<Output = Result<Vec<Restaurant>, RestaurantModelError>> + Send;
}

#[derive(Clone)]
pub struct RestaurantsServiceImpl<B>
where
    B: BatchesService,
{
    pool: Arc<PgPool>,
    batch_service: Arc<B>,
}

impl RestaurantsService for RestaurantsServiceImpl<BatchesServiceImpl> {
    #[instrument(skip(self), fields(region=%region), err)]
    async fn save_restaurants(
        &self,
        restaurants: &[RestaurantSchema],
        admin: Admin,
        region: CrousRegion,
        checksum: String,
    ) -> Result<(), RestaurantModelError> {
        let (batch, mut tx) = self
            .batch_service
            .create_batch(Entity::Restaurants, admin.admin_id, region, checksum)
            .await
            .map_err(|e| -> RestaurantModelError {
                match e {
                    ScrapedBatchModelError::NoDriftWithCurrentBatch => {
                        RestaurantModelError::SyncSkipped
                    }
                    _ => RestaurantModelError::DatabaseError(e.to_string()),
                }
            })?;
        let restaurants: Vec<Restaurant> = restaurants
            .iter()
            .map(|schema| Restaurant {
                restaurant_id: build_id(&schema.name.clone()),
                name: schema.name.clone(),
                url: schema.url.clone(),
                city: schema.city.clone(),
                coordinates: schema.coordinates.clone(),
                opening_hours: schema.opening_hours.clone(),
                created_at: None,
                updated_at: None,
                batch_id: batch,
            })
            .collect();

        for restaurant in restaurants {
            self.pool.create_restaurant(restaurant, &mut tx).await?;
        }

        tx.commit()
            .await
            .map_err(|e| RestaurantModelError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_restaurant_by_id(&self, id: String) -> Result<Restaurant, RestaurantModelError> {
        self.pool.get_restaurant_by_id(id).await
    }

    async fn get_restaurants(
        &self,
        region: CrousRegion,
    ) -> Result<Vec<Restaurant>, RestaurantModelError> {
        let Some(current_batch) = self
            .batch_service
            .current_batch(&Entity::Restaurants, region)
            .await
            .map_err(|_| RestaurantModelError::NotFound)?
        else {
            return Err(RestaurantModelError::NotFound);
        };
        self.pool
            .get_all_restaurants_batch(current_batch.batch_id)
            .await
    }
}

impl<B> RestaurantsServiceImpl<B>
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
