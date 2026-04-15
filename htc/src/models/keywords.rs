use std::future::Future;
use tracing::instrument;

use sqlx::types::Uuid;
use sqlx::{PgPool, PgTransaction};

use crate::regions::CrousRegion;

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Keyword {
    pub keyword_id: Uuid,
    pub keyword: String,
    pub restaurant_id: String,
    pub category: Category,
    pub region: CrousRegion,
}

pub trait KeywordModel {
    fn create_keyword(
        &self,
        keyword: Keyword,
        tx: &mut PgTransaction<'_>,
    ) -> impl Future<Output = Result<(), String>> + Send;
    fn query_restaurant(
        &self,
        query: String,
        region: CrousRegion,
    ) -> impl Future<Output = Result<Vec<String>, String>> + Send;
}

impl KeywordModel for PgPool {
    #[instrument(skip(self), err)]
    async fn create_keyword(
        &self,
        keyword: Keyword,
        tx: &mut PgTransaction<'_>,
    ) -> Result<(), String> {
        let category: &str = keyword.category.into();
        sqlx::query!(
            "INSERT INTO keywords (keyword_id, keyword, restaurant_id, category, region) VALUES ($1, $2, $3, $4, $5)",
            keyword.keyword_id,
            keyword.keyword,
            keyword.restaurant_id,
            category,
            keyword.region.to_string()
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn query_restaurant(
        &self,
        query: String,
        region: CrousRegion,
    ) -> Result<Vec<String>, String> {
        let pattern = format!("%{}%", query);
        let rows = sqlx::query!(
            "SELECT restaurant_id, SUM(word_similarity($1, keyword)) AS score FROM keywords WHERE word_similarity($1, keyword) > 0.5 AND region = $2 GROUP BY restaurant_id ORDER BY score DESC",
            pattern,
            region.to_string()
        )
        .fetch_all(self)
        .await
        .map_err(|e| e.to_string())?;

        let ids = rows.into_iter().map(|row| row.restaurant_id).collect::<Vec<_>>();

        Ok(ids)
    }
}
