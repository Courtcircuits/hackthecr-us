use scraper::Selector;
use thiserror::Error;

use crate::Scraper;

#[derive(Debug)]
pub struct RestaurantListScraper {
    url: String,
}

#[derive(Debug, Clone)]
pub struct RestaurantData {
    pub name: String,
    pub city: String,
    pub crous_url: String,
}

#[derive(Debug, Error)]
pub enum RestaurantListScraperError {
    #[error("Failed to perform HTTP request")]
    RequestFailed,
    #[error("Failed to parse HTML: {0}")]
    ParsingFailed(String),
}

impl RestaurantListScraper {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

impl Scraper<Vec<RestaurantData>> for RestaurantListScraper {
    type Failure = RestaurantListScraperError;

    async fn scrape(&self) -> Result<Vec<RestaurantData>, Self::Failure> {
        let document = crate::get(&self.url)
            .await
            .map_err(|_| RestaurantListScraperError::RequestFailed)?;

        let restaurant_list_selector =
            Selector::parse(".vc_restaurants > ul:nth-child(3) a").map_err(|_| {
                RestaurantListScraperError::ParsingFailed(
                    "Couldn't get .vc_restaurants > ul:nth-child(3) a".to_string(),
                )
            })?;

        let title_selector = Selector::parse(".restaurant_title").map_err(|_| {
            RestaurantListScraperError::ParsingFailed(
                "Couldn't get .restaurant_title".to_string(),
            )
        })?;
        let city_selector = Selector::parse(".restaurant_area").map_err(|_| {
            RestaurantListScraperError::ParsingFailed(
                "Couldn't get .restaurant_area".to_string(),
            )
        })?;

        let mut restaurants = Vec::new();

        for restaurant_list_element in document.select(&restaurant_list_selector) {
            let link_target = restaurant_list_element.value().attr("href").unwrap_or("");

            let title = restaurant_list_element
                .select(&title_selector)
                .next()
                .map(|e| e.inner_html())
                .unwrap_or_default();

            let city = restaurant_list_element
                .select(&city_selector)
                .next()
                .map(|e| e.inner_html())
                .unwrap_or_default();

            if !title.is_empty() && !city.is_empty() && !link_target.is_empty() {
                restaurants.push(RestaurantData {
                    name: title,
                    city,
                    crous_url: link_target.to_string(),
                });
            }
        }

        Ok(restaurants)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_restaurant_list_scraper() {
        let scraper = RestaurantListScraper::new("./src/stubs/ou-manger.html".to_string());
        let result = scraper.scrape().await;

        insta::assert_debug_snapshot!(result);

    }

    #[tokio::test]
    async fn test_restaurant_list_scraper_bfc() {
        let scraper = RestaurantListScraper::new("./src/stubs/ou-manger-bfc.html".to_string());
        let result = scraper.scrape().await;

        insta::assert_debug_snapshot!(result);
    }
}
