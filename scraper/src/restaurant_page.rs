use scraper::{Html, Selector};
use thiserror::Error;

use crate::Scraper;

#[derive(Debug)]
pub struct RestaurantPageScraper {
    url: String,
}

#[derive(Debug)]
pub struct RestaurantPageData {
    pub menus: Vec<MenuData>,
    pub hours: String,
    pub coordinates: (f64, f64),
}

#[derive(Debug)]
pub struct MenuData {
    pub date: String,
    pub meals: Vec<MealData>,
}

#[derive(Debug)]
pub struct MealData {
    pub title: String,
    pub categories: Vec<FoodCategory>,
}

#[derive(Debug)]
pub struct FoodCategory {
    pub name: String,
    pub dishes: Vec<String>,
}

#[derive(Debug, Error)]
pub enum RestaurantPageScraperError {
    #[error("Failed to perform HTTP request")]
    RequestFailed,
    #[error("Failed to parse HTML: {0}")]
    ParsingFailed(String),
}

impl RestaurantPageScraper {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    fn parse(document: &Html) -> Result<RestaurantPageData, RestaurantPageScraperError> {
        let menu_selector = Selector::parse("section.menus div.menu").map_err(|_| {
            RestaurantPageScraperError::ParsingFailed("Couldn't parse menu selector".to_string())
        })?;
        let date_selector = Selector::parse("time.menu_date_title").map_err(|_| {
            RestaurantPageScraperError::ParsingFailed("Couldn't parse date selector".to_string())
        })?;
        let meal_selector = Selector::parse("div.meal").map_err(|_| {
            RestaurantPageScraperError::ParsingFailed("Couldn't parse meal selector".to_string())
        })?;
        let meal_title_selector = Selector::parse("div.meal_title").map_err(|_| {
            RestaurantPageScraperError::ParsingFailed(
                "Couldn't parse meal_title selector".to_string(),
            )
        })?;
        let category_selector = Selector::parse("ul.meal_foodies > li").map_err(|_| {
            RestaurantPageScraperError::ParsingFailed(
                "Couldn't parse category selector".to_string(),
            )
        })?;
        let dish_selector = Selector::parse("ul > li").map_err(|_| {
            RestaurantPageScraperError::ParsingFailed("Couldn't parse dish selector".to_string())
        })?;
        let info_selector = Selector::parse("section.infos div.info").map_err(|_| {
            RestaurantPageScraperError::ParsingFailed("Couldn't parse info selector".to_string())
        })?;
        let info_title_selector = Selector::parse("div.info_title").map_err(|_| {
            RestaurantPageScraperError::ParsingFailed(
                "Couldn't parse info_title selector".to_string(),
            )
        })?;
        let info_p_selector = Selector::parse("p").map_err(|_| {
            RestaurantPageScraperError::ParsingFailed("Couldn't parse info p selector".to_string())
        })?;
        let map_selector = Selector::parse("div#map").map_err(|_| {
            RestaurantPageScraperError::ParsingFailed("Couldn't parse map selector".to_string())
        })?;

        let mut menus = Vec::new();

        for menu_el in document.select(&menu_selector) {
            let date = menu_el
                .select(&date_selector)
                .next()
                .map(|e| e.inner_html())
                .unwrap_or_default();

            let mut meals = Vec::new();

            for meal_el in menu_el.select(&meal_selector) {
                let title = meal_el
                    .select(&meal_title_selector)
                    .next()
                    .map(|e| e.inner_html())
                    .unwrap_or_default();

                let mut categories = Vec::new();

                for category_el in meal_el.select(&category_selector) {
                    let name = category_el
                        .text()
                        .next()
                        .unwrap_or_default()
                        .trim()
                        .to_string();

                    let dishes = category_el
                        .select(&dish_selector)
                        .map(|d| d.inner_html().trim().to_string())
                        .filter(|d| !d.is_empty())
                        .collect();

                    if !name.is_empty() {
                        categories.push(FoodCategory { name, dishes });
                    }
                }

                if !title.is_empty() {
                    meals.push(MealData { title, categories });
                }
            }

            if !date.is_empty() {
                menus.push(MenuData { date, meals });
            }
        }

        let hours = document.select(&info_selector).find_map(|info_el| {
            let title = info_el
                .select(&info_title_selector)
                .next()
                .map(|e| e.text().collect::<String>())?;
            if title.contains("Horaires") {
                info_el
                    .select(&info_p_selector)
                    .next()
                    .map(|p| p.text().collect::<String>().trim().to_string())
            } else {
                None
            }
        });

        let coordinates = document.select(&map_selector).next().and_then(|map_el| {
            let lat = map_el.value().attr("data-lat")?.parse::<f64>().ok()?;
            let lon = map_el.value().attr("data-lon")?.parse::<f64>().ok()?;
            Some((lat, lon))
        });

        let Some(hours) = hours else {
            return Err(RestaurantPageScraperError::ParsingFailed(
                "Couldn't find hours".to_string(),
            ));
        };

        let Some(coordinates) = coordinates else {
            return Err(RestaurantPageScraperError::ParsingFailed(
                "Couldn't find coordinates".to_string(),
            ));
        };

        Ok(RestaurantPageData {
            menus,
            hours,
            coordinates,
        })
    }
}

impl Scraper<RestaurantPageData> for RestaurantPageScraper {
    type Failure = RestaurantPageScraperError;

    async fn scrape(&self) -> Result<RestaurantPageData, Self::Failure> {
        let document = crate::get(&self.url)
            .await
            .map_err(|_| RestaurantPageScraperError::RequestFailed)?;

        Self::parse(&document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_restaurant_page_scraper() {
        let html = include_str!("stubs/brasserie-triolet.html");
        let document = Html::parse_document(html);
        let result = RestaurantPageScraper::parse(&document);
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn test_restaurant_page_scraper_sevenans() {
        let html = include_str!("stubs/sevenans.html");
        let document = Html::parse_document(html);
        let result = RestaurantPageScraper::parse(&document);
        insta::assert_debug_snapshot!(result);
    }
}
