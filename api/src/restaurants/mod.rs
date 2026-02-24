use htc_core::models::restaurants::Restaurant;
use utoipa::ToSchema;

pub mod handlers;
pub mod service;
pub mod router;


#[derive(Debug, Clone, ToSchema)]
pub struct RestaurantSchema {
    pub name: String,
    pub url: String,
    pub city: Option<String>,
    pub coordinates: Option<String>,
    pub opening_hours: Option<String>,
}

impl From<Restaurant> for RestaurantSchema {
    fn from(restaurant: Restaurant) -> Self {
        RestaurantSchema {
            name: restaurant.name,
            url: restaurant.url,
            city: restaurant.city,
            coordinates: restaurant.coordinates,
            opening_hours: restaurant.opening_hours,
        }
    }
}
