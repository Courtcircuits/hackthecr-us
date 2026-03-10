use std::sync::Arc;

use htc::models::restaurants::{Restaurant, RestaurantModelError};

use crate::{
    admins::service::{AdminError, AdminService, AdminServiceImpl},
    config::Config,
    restaurants::service::{RestaurantsServiceImpl, RestaurantsServices},
};

pub trait App {
    fn config(&self) -> &Config;
    fn get_restaurants(&self) -> impl Future<Output = Result<Vec<Restaurant>, RestaurantModelError>> + Send;
    fn get_restaurant_by_name(&self, name: String) -> impl Future<Output = Result<Restaurant, RestaurantModelError>> + Send;
    fn save_restaurants(&self, restaurants: Vec<Restaurant>) -> impl Future<Output = Result<(), RestaurantModelError>> + Send;
    fn get_public_key(&self, name: &str) -> impl Future<Output = Result<String, AdminError>> + Send;
}

pub type DefaultApp = AppImpl<RestaurantsServiceImpl, AdminServiceImpl>;

#[derive(Clone)]
pub struct AppImpl<R, A>
where
    R: RestaurantsServices + Send + Sync,
    A: AdminService + Send + Sync,
{
    restaurants_service: R,
    admin_service: A,
    config: Arc<Config>,
}

impl<R, A> App for AppImpl<R, A>
where
    R: RestaurantsServices + Send + Sync,
    A: AdminService + Send + Sync,
{
    async fn get_restaurants(&self) -> Result<Vec<Restaurant>, RestaurantModelError> {
        self.restaurants_service.get_restaurants().await
    }

    async fn get_restaurant_by_name(&self, name: String) -> Result<Restaurant, RestaurantModelError> {
        self.restaurants_service.get_restaurant_by_name(name).await
    }

    async fn save_restaurants(&self, restaurants: Vec<Restaurant>) -> Result<(), RestaurantModelError> {
        self.restaurants_service.save_restaurants(restaurants).await
    }

    fn config(&self) -> &Config {
        &self.config
    }

    async fn get_public_key(&self, name: &str) -> Result<String, AdminError> {
        self.admin_service.get_public_key(name).await
    }
}

impl<R, A> AppImpl<R, A>
where
    R: RestaurantsServices + Send + Sync,
    A: AdminService + Send + Sync,
{
    pub fn new(restaurants_service: R, admin_service: A, config: Arc<Config>) -> Self {
        Self {
            restaurants_service,
            admin_service,
            config,
        }
    }
}
