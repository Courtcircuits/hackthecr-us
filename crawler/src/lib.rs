use chrono::prelude::*;
use unicode_normalization::UnicodeNormalization;

pub mod restaurant_list;
pub mod restaurant_page;
pub mod school_api;

pub enum ScraperError {
    RequestFailed,
    ParsingFailed(String),
}

#[cfg(not(test))]
pub async fn get(url: &str) -> Result<scraper::Html, ScraperError> {
    let response = reqwest::get(url)
        .await
        .map_err(|_| ScraperError::RequestFailed)?;
    let body = response
        .text()
        .await
        .map_err(|_| ScraperError::RequestFailed)?;
    Ok(scraper::Html::parse_document(&body))
}

#[cfg(test)]
pub async fn get(_url: &str) -> Result<scraper::Html, ScraperError> {
    use std::fs;

    let html = fs::read_to_string(_url).map_err(|_| ScraperError::RequestFailed)?;
    Ok(scraper::Html::parse_document(&html))
}

pub trait Scraper<T> {
    type Failure;
    fn scrape(&self) -> impl Future<Output = Result<T, Self::Failure>>;
}

#[derive(Debug)]
pub enum DateParsingError {
    BadStructure,
    ChronosError(String)
}

pub fn parse_date(date: &str) -> Result<NaiveDate, DateParsingError> {
    let tokens: Vec<&str> = date.split(" ").collect();
    let days = vec!["lundi".to_string(), "mardi".to_string(), "mercredi".to_string(), "jeudi".to_string(), "vendredi".to_string(), "samedi".to_string(), "dimanche".to_string()];
    let months = vec!["janvier".to_string(), "fevrier".to_string(), "mars".to_string(), "avril".to_string(), "mai".to_string(), "juin".to_string(), "juillet".to_string(), "aout".to_string(), "septembre".to_string(), "octobre".to_string(), "novembre".to_string(), "decembre".to_string()];
    if tokens.len() != 6 {
        return Err(DateParsingError::BadStructure)
    }

    if tokens.get(0).unwrap().to_string() != "Menu".to_string() {
        return Err(DateParsingError::BadStructure)
    }

    if tokens.get(1).unwrap().to_string() != "du".to_string() {
        return Err(DateParsingError::BadStructure)
    }

    let day_name = tokens.get(2).unwrap().to_lowercase();

    if !days.contains(&day_name) {
        return Err(DateParsingError::BadStructure)
    }

    let day: u32 = tokens.get(3).unwrap().parse::<u32>().map_err(|_| DateParsingError::BadStructure)?;
    let month = tokens.get(4).unwrap().to_lowercase().nfd().filter(|c| !unicode_normalization::char::is_combining_mark(*c)).collect::<String>();
    let month_number = months.iter().position(|r| *r == month);
    let Some(month_number) = month_number else {
        return Err(DateParsingError::BadStructure);
    };
    let month_number: u32 = (month_number + 1).try_into().map_err(|_| DateParsingError::BadStructure)?;
    let year: i32 = tokens.get(5).unwrap().parse::<i32>().map_err(|_| DateParsingError::BadStructure)?;

    NaiveDate::from_ymd_opt(year, month_number, day).ok_or(DateParsingError::BadStructure)
}
