use futures::future::join_all;
use htc::{
    client::HTCClient,
    models::{meals::MealSchema, restaurants::RestaurantSchema},
    regions::CrousRegion,
    sources::meals::RestaurantPageScrapedData,
};
use crawler::{Scraper, restaurant_page::RestaurantPageScraper};
use tabled::{
    Table, Tabled,
    settings::{Alignment, Style, object::Columns},
};
use thiserror::Error;

use crate::actions::{Executable, ExecutionResult};

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

    pub async fn collect(&self) -> Result<Vec<Vec<MealSchema>>, MealsActionResult> {
        let restaurants_url: Vec<RestaurantSchema> = self
            .client
            .get_restaurants(self.target)
            .await
            .map_err(|e| MealsActionResult::Failure(e.to_string()))?;

        let meals_scrape_futures = restaurants_url.into_iter().map(Self::collect_restaurant);

        let results = join_all(meals_scrape_futures).await;
        let mut meals = Vec::new();
        for result in results {
            meals.push(result?);
        }
        Ok(meals)
    }

    pub async fn collect_restaurant(
        restaurant: RestaurantSchema,
    ) -> Result<Vec<MealSchema>, MealsActionResult> {
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
        let meals: Vec<MealSchema> = scraped_data.into();
        Ok(meals)
    }

    async fn execute_inner(&self) -> Result<(), MealsActionResult> {
        let meals = self.collect().await.map_err(|e| {
            MealsActionResult::Failure(format!("Failed to collect restaurant page data : {:?}", e))
        })?;

        if self.dry_run {
            let table_data: Vec<&MealSchema> = meals.iter().flatten().collect();
            let table_data = table_data.iter().map(|meal| DisplayableMeal {
                meal_type: meal.meal_type.clone(),
                foodies: meal.foodies.clone().unwrap_or("".to_string()),
                date: meal.date.clone().unwrap_or("".to_string()),
                restaurant_id: meal.restaurant_id.to_string(),
            });

            let mut table = Table::new(table_data);
            table.with(Style::modern());
            table.modify(Columns::first(), Alignment::right());
            println!("{}", table);
        } else {
            for meals_by_restaurant in meals {
                if !meals_by_restaurant.is_empty() {
                    let _ = self
                        .client
                        .put_meals(meals_by_restaurant, self.target)
                        .await;
                }
            }
        }
        Ok(())
    }
}

impl Executable for MealsAction {
    fn execute(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ExecutionResult>> + Send + '_>>
    {
        Box::pin(async move {
            self.execute_inner()
                .await
                .map_err(|e| ExecutionResult::Failure(e.to_string()))
        })
    }
}

#[derive(Tabled)]
pub struct DisplayableMeal {
    pub meal_type: String,
    pub foodies: String,
    pub date: String,
    pub restaurant_id: String,
}
