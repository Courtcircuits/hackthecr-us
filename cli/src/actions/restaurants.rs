use thiserror::Error;
use htc_core::{
    models::restaurants::{Restaurant},
    sources::restaurants::RestaurantScrapedData,
};

use scraper::{
    Scraper, restaurant_list::RestaurantListScraper, restaurant_page::RestaurantPageScraper,
};
use tabled::{
    Table, Tabled,
    settings::{Alignment, Style, object::Columns},
};

use crate::crous::{self, CrousRegion, CrousUrl};

pub struct RestaurantsAction {
    pub target: CrousRegion,
    pub dry_run: bool,
}

#[derive(Debug, Error)]
pub enum RestaurantsActionResult {
    #[error("RestaurantsAction failed: {0}")]
    Failure(String),
}

impl RestaurantsAction {
    pub fn new(target: CrousRegion, dry_run: bool) -> Self {
        Self {
            target,
            dry_run,
        }
    }

    pub async fn collect(&self) -> Result<Vec<Restaurant>, RestaurantsActionResult> {
        let url = CrousUrl(self.target.url().to_string()).to_list_url();

        let mut restaurants: Vec<Restaurant> = Vec::new();
        println!(
            "Collecting restaurant data from {}...",
            url
        );
        let list_data = RestaurantListScraper::new(url.to_string())
            .scrape()
            .await
            .map_err(|e| {
                RestaurantsActionResult::Failure(format!("Failed to scrape restaurant list: {}", e))
            })?;

        for restaurant_desc in &list_data {
            let url = &restaurant_desc.crous_url;
            let page_data = RestaurantPageScraper::new(url.to_string())
                .scrape()
                .await
                .map_err(|e| {
                    RestaurantsActionResult::Failure(format!(
                        "Failed to scrape restaurant page at {}: {}",
                        url, e
                    ))
                })?;

            let scraped_data = RestaurantScrapedData {
                page: page_data,
                description: restaurant_desc.clone(),
            };

            let restaurant: Restaurant = scraped_data.into();
            restaurants.push(restaurant);
        }

        Ok(restaurants)
    }

    pub async fn execute(&self) -> Result<(), RestaurantsActionResult> {
        let restaurants = self.collect().await.map_err(|e| {
            RestaurantsActionResult::Failure(format!("Failed to collect restaurant data: {:?}", e))
        })?;

        if self.dry_run {
            println!("Collected {} restaurants (dry run):", restaurants.len());
            let table_data = restaurants.iter().map(|restaurant| {
                let restaurant: Restaurant = restaurant.clone();
                DisplayableRestaurant {
                    name: restaurant.name,
                    url: restaurant.url,
                    city: restaurant.city.unwrap_or_else(|| "N/A".to_string()),
                    coordinates: restaurant.coordinates.unwrap_or_else(|| "N/A".to_string()),
                    opening_hours: restaurant
                        .opening_hours
                        .unwrap_or_else(|| "N/A".to_string()),
                }
            });
            let mut table = Table::new(table_data);
            table.with(Style::modern());
            table.modify(Columns::first(), Alignment::right());
            println!("{}", table);
            Ok(())
        } else {
            // Here you would normally save the restaurants to a database
            // For this example, we'll just print them out
            for restaurant in &restaurants {
                let restaurant: Restaurant = restaurant.clone();
            }
            Ok(())
        }
    }
}

#[derive(Tabled)]
pub struct DisplayableRestaurant {
    pub name: String,
    pub url: String,
    pub city: String,
    pub coordinates: String,
    pub opening_hours: String,
}
