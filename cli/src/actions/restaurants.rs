use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use futures::future::join_all;
use htc::{models::restaurants::RestaurantSchema, sources::restaurants::RestaurantScrapedData};
use thiserror::Error;

use scraper::{
    Scraper, restaurant_list::RestaurantListScraper, restaurant_page::RestaurantPageScraper,
};
use tabled::{
    Table, Tabled,
    settings::{Alignment, Style, object::Columns},
};
use zenity::progress::{Frames, ProgressBar};

use crate::{
    client::HTCClient,
    crous::{CrousRegion, CrousUrl},
};

pub struct RestaurantsAction {
    pub target: CrousRegion,
    pub dry_run: bool,

    pub client: HTCClient,
}

#[derive(Debug, Error)]
pub enum RestaurantsActionResult {
    #[error("RestaurantsAction failed: {0}")]
    Failure(String),
}

impl RestaurantsAction {
    pub fn new(target: CrousRegion, dry_run: bool, client: HTCClient) -> Self {
        Self {
            target,
            dry_run,
            client,
        }
    }

    pub async fn collect(&self) -> Result<Vec<RestaurantSchema>, RestaurantsActionResult> {
        let url = CrousUrl(self.target.url().to_string()).to_list_url();

        let mut restaurants: Vec<RestaurantSchema> = Vec::new();
        let list_data = RestaurantListScraper::new(url.to_string())
            .scrape()
            .await
            .map_err(|e| {
                RestaurantsActionResult::Failure(format!("Failed to scrape restaurant list: {}", e))
            })?;

        let progress = Arc::new(ProgressBar::new(Frames::rect().set_goal(list_data.len())));
        let uid = progress.get_last();
        progress.run_all();

        let counter = Arc::new(AtomicUsize::new(0));
        let futures = list_data
            .into_iter()
            .map(|data| Self::collect_restaurant(data, progress.clone(), uid, counter.clone()));
        let results = join_all(futures).await;
        for result in results {
            restaurants.push(result?);
        }

        Ok(restaurants)
    }

    async fn collect_restaurant(
        restaurant_desc: scraper::restaurant_list::RestaurantData,
        progress_bar: Arc<ProgressBar>,
        uid: usize,
        counter: Arc<AtomicUsize>,
    ) -> Result<RestaurantSchema, RestaurantsActionResult> {
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
            description: restaurant_desc,
        };

        let completed = counter.fetch_add(1, Ordering::Relaxed) + 1;
        progress_bar.set(&uid, &completed);

        Ok(scraped_data.into())
    }

    pub async fn execute(&self) -> Result<(), RestaurantsActionResult> {
        let restaurants = self.collect().await.map_err(|e| {
            RestaurantsActionResult::Failure(format!("Failed to collect restaurant data: {:?}", e))
        })?;

        if self.dry_run {
            let table_data = restaurants.iter().map(|restaurant| {
                let restaurant: RestaurantSchema = restaurant.clone();
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
        } else {
            // Here you would normally save the restaurants to a database
            // For this example, we'll just print them out
            let _ = self.client.put_restaurants(restaurants).await;
        }
        Ok(())
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
