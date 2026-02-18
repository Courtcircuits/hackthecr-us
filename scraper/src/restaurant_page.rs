use scraper::{Html, Selector};

use crate::Scraper;

#[derive(Debug)]
pub struct RestaurantPageScraper {
    url: String,
}

#[derive(Debug)]
pub struct RestaurantPageData {
    pub menus: Vec<MenuData>,
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

#[derive(Debug)]
pub enum RestaurantPageScraperError {
    RequestFailed,
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

        Ok(RestaurantPageData { menus })
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
