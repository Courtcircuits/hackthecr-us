use std::future::Future;

use sqlx::PgPool;
use sqlx::types::Uuid;

pub enum Category {
    Meal,
    Restaurant,
    Food,
}

impl From<&str> for Category {
    fn from(value: &str) -> Self {
        match value {
            "meal" => Category::Meal,
            "food" => Category::Food,
            "restaurant" => Category::Restaurant,
            _ => panic!("invalid category: {}", value),
        }
    }
}

impl From<Category> for &str {
    fn from(value: Category) -> Self {
        match value {
            Category::Meal => "meal",
            Category::Food => "food",
            Category::Restaurant => "restaurant",
        }
    }
}

pub struct Keyword {
    pub keyword_id: Uuid,
    pub keyword: String,
    pub restaurant_id: Uuid,
    pub category: Category,
}

pub trait KeywordModel {
    fn create_keyword(&self, keyword: Keyword) -> impl Future<Output = Result<(), String>> + Send;
    fn get_keywords_by_restaurant_id(&self, restaurant_id: Uuid) -> impl Future<Output = Result<Vec<Keyword>, String>> + Send;
    fn query_restaurant(&self, query: String) -> impl Future<Output = Result<Vec<Uuid>, String>> + Send;
}

impl KeywordModel for PgPool {
    async fn create_keyword(&self, keyword: Keyword) -> Result<(), String> {
        let category: &str = keyword.category.into();
        sqlx::query!(
            "INSERT INTO keywords (keyword_id, keyword, restaurant_id, category) VALUES ($1, $2, $3, $4)",
            keyword.keyword_id,
            keyword.keyword,
            keyword.restaurant_id,
            category
        )
        .execute(self)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn get_keywords_by_restaurant_id(&self, restaurant_id: Uuid) -> Result<Vec<Keyword>, String> {
        let rows = sqlx::query!(
            "SELECT keyword_id, keyword, restaurant_id, category FROM keywords WHERE restaurant_id = $1",
            restaurant_id
        )
        .fetch_all(self)
        .await
        .map_err(|e| e.to_string())?;

        let keywords = rows
            .into_iter()
            .map(|row| Keyword {
                keyword_id: row.keyword_id,
                keyword: row.keyword,
                restaurant_id: row.restaurant_id,
                category: Category::from(row.category.as_str()),
            })
            .collect();

        Ok(keywords)
    }

    async fn query_restaurant(&self, query: String) -> Result<Vec<Uuid>, String> {
        let pattern = format!("%{}%", query);
        let rows = sqlx::query!(
            "SELECT DISTINCT restaurant_id FROM keywords WHERE keyword ILIKE $1",
            pattern
        )
        .fetch_all(self)
        .await
        .map_err(|e| e.to_string())?;

        let ids = rows.into_iter().map(|row| row.restaurant_id).collect();

        Ok(ids)
    }
}
