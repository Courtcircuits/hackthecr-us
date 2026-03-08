use scraper::{restaurant_list::RestaurantData, restaurant_page::RestaurantPageData};
use sqlx::types::uuid;

use crate::models::restaurants::Restaurant;

pub struct RestaurantScrapedData {
    pub page: RestaurantPageData,
    pub description: RestaurantData,
}

impl Into<Restaurant> for RestaurantScrapedData {
    fn into(self) -> Restaurant {
        let (latitude, longitude) = self.page.coordinates;
        Restaurant {
            restaurant_id: uuid::uuid!("00000000-0000-0000-0000-000000000000"),
            name: self.description.name,
            url: self.description.crous_url,
            city: Some(self.description.city),
            coordinates: Some(format!("{},{}", latitude, longitude)),
            opening_hours: Some(self.page.hours),
            created_at: None,
            updated_at: None
        }
    }
}

#[cfg(test)]
mod tests {
    use scraper::restaurant_page::{FoodCategory, MealData, MenuData};

    use super::*;

    #[test]
    fn test_restaurant_conversion() {
        let restaurant_page_data = RestaurantPageData {
            menus: vec![
                MenuData {
                    date: "2024-06-01".to_string(),
                    meals: vec![
                        MealData {
                            title: "Lunch".to_string(),
                            categories: vec![
                                FoodCategory {
                                    name: "Main Course".to_string(),
                                    dishes: vec!["Spaghetti".to_string(), "Pizza".to_string()],
                                },
                                FoodCategory {
                                    name: "Dessert".to_string(),
                                    dishes: vec!["Ice Cream".to_string()],
                                },
                            ],
                        },
                    ],
                },
            ],
            hours: "12:00 - 14:00".to_string(),
            coordinates: (48.5734, 7.7521),
        };

        let restaurant_data = RestaurantData {
            name: "Test Restaurant".to_string(),
            city: "Strasbourg".to_string(),
            crous_url: "https://example.com/restaurant".to_string(),
        };

        let scraped = RestaurantScrapedData {
            page: restaurant_page_data,
            description: restaurant_data,
        };

        let restaurant: Restaurant = scraped.into();
        assert_eq!(restaurant.name, "Test Restaurant");
        assert_eq!(restaurant.url, "https://example.com/restaurant");
        assert_eq!(restaurant.city, Some("Strasbourg".to_string()));
        assert_eq!(restaurant.coordinates, Some("48.5734,7.7521".to_string()));
        assert_eq!(restaurant.opening_hours, Some("12:00 - 14:00".to_string()));
    }
}
