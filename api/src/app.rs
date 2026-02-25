use std::sync::Arc;

use htc_core::models::restaurants::{Restaurant, RestaurantModelError};

use crate::{config::Config, restaurants::service::{RestaurantsServiceImpl, RestaurantsServices}};

pub trait App{
    fn config(&self) -> &Config;
    fn get_restaurants(&self) -> impl Future<Output = Result<Vec<Restaurant>, RestaurantModelError>> + Send;
    fn get_restaurant_by_name(&self, name: String) -> impl Future<Output = Result<Restaurant, RestaurantModelError>> + Send;
    fn save_restaurants(&self, restaurants: Vec<Restaurant>) -> impl Future<Output = Result<(), RestaurantModelError>> + Send;
}

pub type DefaultApp = AppImpl<RestaurantsServiceImpl>;

#[derive(Clone)]
pub struct AppImpl<R> 
    where R: RestaurantsServices + Send + Sync
{
    restaurants_service: R,
    config: Arc<Config>,
}

impl<R> App for AppImpl<R> 
where 
    R: RestaurantsServices + Send + Sync
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
}

impl <R> AppImpl<R> 
where 
    R: RestaurantsServices + Send + Sync
{
    pub fn new(restaurants_service: R, config: Arc<Config>) -> Self {
        Self {
            restaurants_service,
            config
        }
    }
}
