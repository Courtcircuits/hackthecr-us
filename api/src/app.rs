use std::sync::Arc;

use htc::{
    models::{
        Entity,
        admins::Admin,
        meals::{Meal, MealModelError, MealSchema},
        restaurants::{Restaurant, RestaurantModelError, RestaurantSchema},
        scrape_batch::ScrapedBatchModelError,
    },
    regions::CrousRegion,
};
use sqlx::PgTransaction;
use uuid::Uuid;

use crate::{
    admins::service::{AdminError, AdminService, AdminServiceImpl},
    batches::service::{BatchesService, BatchesServiceImpl},
    config::Config,
    meals::service::{MealsService, MealsServiceImpl},
    restaurants::service::{RestaurantsService, RestaurantsServiceImpl},
};

pub trait App {
    fn config(&self) -> &Config;
    fn get_restaurants(
        &self,
        region: CrousRegion,
    ) -> impl Future<Output = Result<Vec<Restaurant>, RestaurantModelError>> + Send;
    fn get_restaurant_by_id(
        &self,
        name: String,
    ) -> impl Future<Output = Result<Restaurant, RestaurantModelError>> + Send;
    fn save_restaurants(
        &self,
        restaurants: &[RestaurantSchema],
        admin: Admin,
        crous_region: CrousRegion,
        checksum: String,
    ) -> impl Future<Output = Result<(), RestaurantModelError>> + Send;
    fn save_meals(
        &self,
        meals: &[MealSchema],
        admin: Admin,
        crous_region: CrousRegion,
        checksum: String,
    ) -> impl Future<Output = Result<(), MealModelError>> + Send;
    fn get_admin(&self, name: &str) -> impl Future<Output = Result<Admin, AdminError>> + Send;
    fn get_meals_by_restaurant_id(
        &self,
        name: String,
        region: CrousRegion,
    ) -> impl Future<Output = Result<Vec<Meal>, MealModelError>> + Send;
    fn create_batch(
        &'_ self,
        entity: Entity,
        author_id: Uuid,
        region: CrousRegion,
        checksum: String,
    ) -> impl Future<Output = Result<(Uuid, PgTransaction<'_>), ScrapedBatchModelError>> + Send;
}

pub type DefaultApp = AppImpl<
    RestaurantsServiceImpl<BatchesServiceImpl>,
    MealsServiceImpl<BatchesServiceImpl>,
    AdminServiceImpl,
    BatchesServiceImpl,
>;

#[derive(Clone)]
pub struct AppImpl<R, M, A, S>
where
    R: RestaurantsService + Send + Sync,
    M: MealsService + Send + Sync,
    A: AdminService + Send + Sync,
    S: BatchesService + Send + Sync,
{
    restaurants_service: R,
    meals_service: M,
    admin_service: A,
    batch_service: Arc<S>,
    config: Arc<Config>,
}

impl<R, M, A, S> App for AppImpl<R, M, A, S>
where
    R: RestaurantsService + Send + Sync,
    M: MealsService + Send + Sync,
    A: AdminService + Send + Sync,
    S: BatchesService + Send + Sync,
{
    async fn get_restaurants(
        &self,
        region: CrousRegion,
    ) -> Result<Vec<Restaurant>, RestaurantModelError> {
        self.restaurants_service.get_restaurants(region).await
    }

    async fn get_restaurant_by_id(&self, name: String) -> Result<Restaurant, RestaurantModelError> {
        self.restaurants_service.get_restaurant_by_id(name).await
    }

    async fn save_restaurants(
        &self,
        restaurants: &[RestaurantSchema],
        admin: Admin,
        crous_region: CrousRegion,
        checksum: String,
    ) -> Result<(), RestaurantModelError> {
        self.restaurants_service
            .save_restaurants(restaurants, admin, crous_region, checksum)
            .await
    }

    async fn save_meals(
        &self,
        meals: &[MealSchema],
        admin: Admin,
        crous_region: CrousRegion,
        checksum: String,
    ) -> Result<(), MealModelError> {
        self.meals_service
            .save_meals(meals, admin, crous_region, checksum)
            .await
    }

    fn config(&self) -> &Config {
        &self.config
    }

    async fn get_admin(&self, name: &str) -> Result<Admin, AdminError> {
        self.admin_service.get_admin(name).await
    }

    async fn get_meals_by_restaurant_id(
        &self,
        name: String,
        region: CrousRegion,
    ) -> Result<Vec<Meal>, MealModelError> {
        self.meals_service
            .get_meals_by_restaurant_id(name, region)
            .await
    }

    async fn create_batch(
        &'_ self,
        entity: Entity,
        author_id: Uuid,
        region: CrousRegion,
        checksum: String,
    ) -> Result<(Uuid, PgTransaction<'_>), ScrapedBatchModelError> {
        self.batch_service
            .create_batch(entity, author_id, region, checksum)
            .await
    }
}

impl<R, M, A, S> AppImpl<R, M, A, S>
where
    R: RestaurantsService + Send + Sync,
    M: MealsService + Send + Sync,
    A: AdminService + Send + Sync,
    S: BatchesService + Send + Sync,
{
    pub fn new(
        restaurants_service: R,
        meals_service: M,
        admin_service: A,
        batch_service: Arc<S>,
        config: Arc<Config>,
    ) -> Self {
        Self {
            restaurants_service,
            meals_service,
            admin_service,
            batch_service,
            config,
        }
    }
}
