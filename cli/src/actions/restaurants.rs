use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use futures::future::join_all;
use htc::{
    client::HTCClient,
    models::restaurants::RestaurantSchema,
    regions::{CrousRegion, CrousUrl},
    sources::restaurants::RestaurantScrapedData,
};
use crawler::{
    Scraper, restaurant_list::RestaurantListScraper, restaurant_page::RestaurantPageScraper,
};
use rattles::presets::prelude as presets;
use tabled::{
    Table, Tabled,
    settings::{Alignment, Style, object::Columns},
};

use crate::actions::{Executable, ExecutionResult};

pub struct RestaurantsAction {
    pub target: CrousRegion,
    pub dry_run: bool,

    pub client: HTCClient,
}

impl Executable for RestaurantsAction {
    fn execute(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), ExecutionResult>> + Send + '_>>
    {
        Box::pin(async move {
            let restaurants = self.collect().await.map_err(|e| {
                ExecutionResult::Failure(format!("Failed to collect restaurant data: {:?}", e))
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
                let _ = self.client.put_restaurants(restaurants, self.target).await;
            }
            Ok(())
        })
    }
}

impl RestaurantsAction {
    pub fn new(target: CrousRegion, dry_run: bool, client: HTCClient) -> Self {
        Self {
            target,
            dry_run,
            client,
        }
    }

    pub async fn collect(&self) -> Result<Vec<RestaurantSchema>, ExecutionResult> {
        let url = CrousUrl(self.target.url().to_string()).to_list_url();

        let mut restaurants: Vec<RestaurantSchema> = Vec::new();
        let list_data = RestaurantListScraper::new(url.to_string())
            .scrape()
            .await
            .map_err(|e| {
                ExecutionResult::Failure(format!("Failed to scrape restaurant list: {}", e))
            })?;

        let total = list_data.len();
        let progress = Arc::new(AtomicUsize::new(0));
        let rattle = presets::waverows();

        let spinner_progress = Arc::clone(&progress);
        let spinner = tokio::spawn(async move {
            loop {
                let done = spinner_progress.load(Ordering::Relaxed);
                let frame = rattle.current_frame();
                print!("\r{} Scraping restaurants... {}/{}", frame, done, total);
                let _ = std::io::Write::flush(&mut std::io::stdout());
                tokio::time::sleep(std::time::Duration::from_millis(80)).await;
            }
        });

        let futures = list_data
            .into_iter()
            .map(|r| {
                let progress = Arc::clone(&progress);
                async move {
                    let result = Self::collect_restaurant(r).await;
                    progress.fetch_add(1, Ordering::Relaxed);
                    result
                }
            });
        let results = join_all(futures).await;

        spinner.abort();
        print!("\r{}\r", " ".repeat(60));
        let _ = std::io::Write::flush(&mut std::io::stdout());

        for result in results {
            restaurants.push(result?);
        }

        Ok(restaurants)
    }

    async fn collect_restaurant(
        restaurant_desc: crawler::restaurant_list::RestaurantData,
    ) -> Result<RestaurantSchema, ExecutionResult> {
        let url = &restaurant_desc.crous_url;
        let page_data = RestaurantPageScraper::new(url.to_string())
            .scrape()
            .await
            .map_err(|e| {
                ExecutionResult::Failure(format!(
                    "Failed to scrape restaurant page at {}: {}",
                    url, e
                ))
            })?;

        let scraped_data = RestaurantScrapedData {
            page: page_data,
            description: restaurant_desc,
        };


        Ok(scraped_data.into())
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
