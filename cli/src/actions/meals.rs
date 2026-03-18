use tabled::{
    Table, Tabled,
    settings::{Alignment, Style, object::Columns},
};
use futures::future::join_all;
use htc::{
    models::{
        meals::Meal,
        restaurants::{RestaurantSchema},
    },
    sources::meals::RestaurantPageScrapedData,
};
use scraper::{
    Scraper,
    restaurant_page::{RestaurantPageScraper},
};
use thiserror::Error;

use crate::{client::HTCClient, crous::CrousRegion};

pub struct MealsAction {
    pub target: CrousRegion,
    pub dry_run: bool,

    pub client: HTCClient,
}

#[derive(Debug, Error)]
pub enum MealsActionResult {
    #[error("MealsAction failed : {0}")]
    Failure(String),
}

impl MealsAction {
    pub fn new(target: CrousRegion, dry_run: bool, client: HTCClient) -> Self {
        Self {
            target,
            dry_run,
            client,
        }
    }

    pub async fn collect(&self) -> Result<Vec<Vec<Meal>>, MealsActionResult> {
        let restaurants_url: Vec<RestaurantSchema> = self
            .client
            .get_restaurants()
            .await
            .map_err(|e| MealsActionResult::Failure(e.to_string()))?;

        let meals_scrape_futures = restaurants_url
            .into_iter()
            .map(Self::collect_restaurant);

        let results = join_all(meals_scrape_futures).await;
        let mut meals = Vec::new();
        for result in results {
            meals.push(result?);
        }
        Ok(meals)
    }

    pub async fn collect_restaurant(
        restaurant: RestaurantSchema,
    ) -> Result<Vec<Meal>, MealsActionResult> {
        let page_data = RestaurantPageScraper::new(restaurant.url.to_string())
            .scrape()
            .await
            .map_err(|e| {
                MealsActionResult::Failure(format!("Failed to scraper restaurant page : {}", e))
            })?;
        let scraped_data = RestaurantPageScrapedData {
            restaurant,
            page: page_data,
        };
        let meals: Vec<Meal> = scraped_data.try_into().map_err(|_| {
            MealsActionResult::Failure("Failed to convert restaurant page".to_string())
        })?;

        Ok(meals)
    }

    pub async fn execute(&self) -> Result<(), MealsActionResult> {
        let meals = self.collect().await.map_err(|e| {
            MealsActionResult::Failure(format!("Failed to collect restaurant page data : {:?}", e))
        })?;

        if self.dry_run {
            let table_data: Vec<&Meal>= meals.iter().flatten().collect();
            let table_data= table_data.iter().map(|meal| {
                DisplayableMeal {
                    meal_type: meal.meal_type.clone(),
                    foodies: meal.foodies.clone().unwrap_or("".to_string()),
                    date: meal.date.clone().unwrap_or("".to_string()),
                    restaurant_id: meal.restaurant_id.to_string()

                }
            });

            let mut table = Table::new(table_data);
            table.with(Style::modern());
            table.modify(Columns::first(), Alignment::right());
            println!("{}", table);
        }
        Ok(())
    }
}

#[derive(Tabled)]
pub struct DisplayableMeal {
    pub meal_type: String,
    pub foodies: String,
    pub date: String,
    pub restaurant_id: String
}
