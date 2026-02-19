use scraper::restaurant_page::RestaurantPageData;
use sqlx::types::uuid;

use crate::models::{meals::Meal, restaurants::Restaurant};

pub struct RestaurantPageScrapedData {
    pub restaurant: Restaurant,
    pub page: RestaurantPageData
}

impl Into<Vec<Meal>> for RestaurantPageScrapedData  {
    fn into(self) -> Vec<Meal> {
        let mut meals = Vec::new();
        for menu in self.page.menus {
            for meal_data in menu.meals {
                for category in meal_data.categories {
                    for dish in category.dishes {
                        meals.push(Meal {
                            meal_id: uuid::uuid!("00000000-0000-0000-0000-000000000000"),
                            restaurant_id: self.restaurant.restaurant_id,
                            foodies: Some(dish),
                            meal_type: category.name.clone(),
                            date: Some(menu.date.clone()),
                            scraped_at: None
                        });
                    }
                }
            }
        }
        meals

    }
}

#[cfg(test)]
mod tests {
    use scraper::restaurant_page::{FoodCategory, MealData, MenuData, RestaurantPageData};
    use sqlx::types::uuid;

    use crate::models::restaurants::Restaurant;

    use super::*;

    fn make_restaurant() -> Restaurant {
        Restaurant {
            restaurant_id: uuid::uuid!("00000000-0000-0000-0000-000000000001"),
            name: "Test Restaurant".to_string(),
            url: "https://example.com".to_string(),
            coordinates: None,
            opening_hours: None,
            created_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn test_empty_page_gives_no_meals() {
        let scraped = RestaurantPageScrapedData {
            restaurant: make_restaurant(),
            page: RestaurantPageData {
                menus: vec![],
                hours: "12:00 - 14:00".to_string(),
                coordinates: (48.5734, 7.7521),
            },
        };
        let meals: Vec<Meal> = scraped.into();
        assert!(meals.is_empty());
    }

    #[test]
    fn test_single_dish_becomes_one_meal() {
        let restaurant = make_restaurant();
        let restaurant_id = restaurant.restaurant_id;
        let scraped = RestaurantPageScrapedData {
            restaurant,
            page: RestaurantPageData {
                menus: vec![MenuData {
                    date: "2024-06-01".to_string(),
                    meals: vec![MealData {
                        title: "Lunch".to_string(),
                        categories: vec![FoodCategory {
                            name: "Main Course".to_string(),
                            dishes: vec!["Spaghetti".to_string()],
                        }],
                    }],
                }],
                hours: "12:00 - 14:00".to_string(),
                coordinates: (48.5734, 7.7521),
            },
        };
        let meals: Vec<Meal> = scraped.into();
        assert_eq!(meals.len(), 1);
        assert_eq!(meals[0].restaurant_id, restaurant_id);
        assert_eq!(meals[0].foodies, Some("Spaghetti".to_string()));
        assert_eq!(meals[0].meal_type, "Main Course");
        assert_eq!(meals[0].date, Some("2024-06-01".to_string()));
    }

    #[test]
    fn test_multiple_dishes_become_multiple_meals() {
        let scraped = RestaurantPageScrapedData {
            restaurant: make_restaurant(),
            page: RestaurantPageData {
                menus: vec![MenuData {
                    date: "2024-06-01".to_string(),
                    meals: vec![MealData {
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
                    }],
                }],
                hours: "12:00 - 14:00".to_string(),
                coordinates: (48.5734, 7.7521),
            },
        };
        let meals: Vec<Meal> = scraped.into();
        assert_eq!(meals.len(), 3);
        assert_eq!(meals[0].foodies, Some("Spaghetti".to_string()));
        assert_eq!(meals[0].meal_type, "Main Course");
        assert_eq!(meals[1].foodies, Some("Pizza".to_string()));
        assert_eq!(meals[1].meal_type, "Main Course");
        assert_eq!(meals[2].foodies, Some("Ice Cream".to_string()));
        assert_eq!(meals[2].meal_type, "Dessert");
    }

    #[test]
    fn test_multiple_menus_produce_correct_dates() {
        let scraped = RestaurantPageScrapedData {
            restaurant: make_restaurant(),
            page: RestaurantPageData {
                menus: vec![
                    MenuData {
                        date: "2024-06-01".to_string(),
                        meals: vec![MealData {
                            title: "Lunch".to_string(),
                            categories: vec![FoodCategory {
                                name: "Main Course".to_string(),
                                dishes: vec!["Spaghetti".to_string()],
                            }],
                        }],
                    },
                    MenuData {
                        date: "2024-06-02".to_string(),
                        meals: vec![MealData {
                            title: "Dinner".to_string(),
                            categories: vec![FoodCategory {
                                name: "Main Course".to_string(),
                                dishes: vec!["Pizza".to_string()],
                            }],
                        }],
                    },
                ],
                hours: "12:00 - 14:00".to_string(),
                coordinates: (48.5734, 7.7521),
            },
        };
        let meals: Vec<Meal> = scraped.into();
        assert_eq!(meals.len(), 2);
        assert_eq!(meals[0].date, Some("2024-06-01".to_string()));
        assert_eq!(meals[1].date, Some("2024-06-02".to_string()));
    }
}
