pub mod restaurant_page;
pub mod restaurant_list;
pub mod school_api;

pub enum ScraperError {
    RequestFailed,
    ParsingFailed(String),
}


#[cfg(not(test))]
pub async fn get(url: &str) -> Result<scraper::Html, ScraperError> {
    let response = reqwest::get(url).await.map_err(|_| ScraperError::RequestFailed)?;
    let body = response.text().await.map_err(|_| ScraperError::RequestFailed)?;
    Ok(scraper::Html::parse_document(&body))
}

#[cfg(test)]
pub async fn get(_url: &str) -> Result<scraper::Html, ScraperError> {
    let html = include_str!("stubs/ou-manger.html");
    Ok(scraper::Html::parse_document(html))
}

pub trait Scraper<T> {
    type Failure;
    fn scrape(&self) -> impl Future<Output = Result<T, Self::Failure>>;
}
