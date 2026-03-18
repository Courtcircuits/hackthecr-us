use chrono::NaiveDateTime;
use sqlx::{PgPool, types::Uuid};

use crate::models::Entity;

#[derive(Clone)]
pub struct ScrapeBatch {
    pub batch_id: Uuid,
    pub entity: Entity,
    pub author: Uuid,
    pub scraped_at: Option<NaiveDateTime>,
}

#[derive(thiserror::Error, Debug)]
pub enum ScrapedBatchModelError {
    #[error("No scrapper batch available")]
    NotFound,
    #[error("Database error : {0}")]
    DatabaseError(String),
}

pub trait ScrapedBatchModel {
    fn create_batch(
        &self,
        batch: ScrapeBatch,
    ) -> impl Future<Output = Result<(), ScrapedBatchModelError>> + Send;

    fn current_batch(
        &self,
        entity: Entity
    ) -> impl Future<Output = Result<Uuid, ScrapedBatchModelError>> + Send;
}
impl ScrapedBatchModel for PgPool {
    async fn create_batch(&self, batch: ScrapeBatch) -> Result<(), ScrapedBatchModelError> {
        sqlx::query!(
            "INSERT INTO scrape_batch(batch_id, entity, author) VALUES ($1, $2, $3)",
            batch.batch_id,
            batch.entity.to_string(),
            batch.author,
        )
        .execute(self)
        .await
        .map_err(|e| ScrapedBatchModelError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn current_batch(
            &self,
            entity: Entity,
        ) -> Result<Uuid, ScrapedBatchModelError> {
        let row = sqlx::query!(
            "SELECT batch_id FROM scrape_batch WHERE entity = $1 ORDER BY scraped_at LIMIT 1",
            entity.to_string()
        )
        .fetch_optional(self)
        .await
        .map_err(|e| ScrapedBatchModelError::DatabaseError(e.to_string()))?
        .ok_or(ScrapedBatchModelError::NotFound)?;
        
        Ok(row.batch_id)
    }
}
