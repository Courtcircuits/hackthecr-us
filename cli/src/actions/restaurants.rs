use core::{models::restaurants::Restaurant, sources::restaurants::RestaurantScrapedData};

use scraper::{
    Scraper, restaurant_list::RestaurantListScraper, restaurant_page::RestaurantPageScraper,
};
use tabled::{Table, Tabled, settings::{Alignment, Style, object::Columns}};

use crate::crous;

pub struct RestaurantsAction {
    pub target: String,
    pub dry_run: bool,
}

pub enum RestaurantsActionResult {
    Success,
    Failure(String),
}

impl RestaurantsAction {
    pub fn new(target: String, dry_run: bool) -> Self {
        Self { target, dry_run }
    }

    pub async fn collect(&self) -> Result<Vec<Restaurant>, RestaurantsActionResult> {
        let urls = crous::get_urls();

        let mut restaurants: Vec<Restaurant> = Vec::new();
        println!("Collecting restaurant data from {}...", urls.montpellier.to_list_url());
        let list_data = RestaurantListScraper::new(urls.montpellier.to_list_url())
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

    pub async fn execute(&self) -> RestaurantsActionResult {
        match self.collect().await {
            Ok(restaurants) => {
                if self.dry_run {
                    println!("Collected {} restaurants (dry run):", restaurants.len());
                    let table_data = restaurants.iter().map(|restaurant| {
                        let restaurant: Restaurant = restaurant.clone();
                        DisplayableRestaurant {
                            name: restaurant.name,
                            url: restaurant.url,
                            city: restaurant.city.unwrap_or_else(|| "N/A".to_string()),
                            coordinates: restaurant.coordinates.unwrap_or_else(|| "N/A".to_string()),
                            opening_hours: restaurant.opening_hours.unwrap_or_else(|| "N/A".to_string()),
                        }
                    });
                    let mut table = Table::new(table_data);
                    table.with(Style::modern());
                    table.modify(Columns::first(), Alignment::right());
                    println!("{}", table);
                    RestaurantsActionResult::Success
                } else {
                    // Here you would normally save the restaurants to a database
                    // For this example, we'll just print them out
                    println!("Collected {} restaurants:", restaurants.len());
                    RestaurantsActionResult::Success
                }
            }
            Err(e) => e,
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
