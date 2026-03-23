use std::str::FromStr;

use chrono::NaiveDateTime;
use serde::Deserialize;
use sqlx::{PgPool, PgTransaction, types::Uuid};

use crate::{models::Entity, regions::CrousRegion};

#[derive(Clone, Deserialize, serde::Serialize)]
pub struct ScrapeBatch {
    pub batch_id: Uuid,
    pub entity: Entity,
    pub author: Uuid,
    pub region: String,
    pub scraped_at: Option<NaiveDateTime>,
    pub checksum: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ScrapedBatchModelError {
    #[error("Database error : {0}")]
    DatabaseError(String),
    #[error("Transaction error : {0}")]
    TransactionError(String),
    #[error("Not an entity")]
    NotAnEntity,
    #[error("No drift with current batch")]
    NoDriftWithCurrentBatch,
}

pub trait ScrapedBatchModel {
    fn create_batch(
        &'_ self,
        batch: ScrapeBatch,
    ) -> impl Future<Output = Result<PgTransaction<'_>, ScrapedBatchModelError>> + Send;

    fn current_batch(
        &self,
        entity: &Entity,
        region: CrousRegion,
    ) -> impl Future<Output = Result<Option<ScrapeBatch>, ScrapedBatchModelError>> + Send;
}
impl ScrapedBatchModel for PgPool {
    async fn create_batch(
        &'_ self,
        batch: ScrapeBatch,
    ) -> Result<PgTransaction<'_>, ScrapedBatchModelError> {
        let mut tx = self
            .begin()
            .await
            .map_err(|e| ScrapedBatchModelError::TransactionError(e.to_string()))?;
        sqlx::query!(
            "INSERT INTO scrape_batch(batch_id, entity, author, region, checksum) VALUES ($1, $2, $3, $4, $5)",
            batch.batch_id,
            batch.entity.to_string(),
            batch.author,
            batch.region,
            batch.checksum
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| ScrapedBatchModelError::DatabaseError(e.to_string()))?;

        Ok(tx)
    }

    async fn current_batch(
        &self,
        entity: &Entity,
        region: CrousRegion,
    ) -> Result<Option<ScrapeBatch>, ScrapedBatchModelError> {
        let row = sqlx::query!(
            "SELECT batch_id, entity, author, region, checksum, scraped_at FROM scrape_batch WHERE entity = $1 AND region = $2 ORDER BY scraped_at LIMIT 1",
            entity.to_string(),
            region.to_string()
        )
        .fetch_optional(self)
        .await
        .map_err(|e| ScrapedBatchModelError::DatabaseError(e.to_string()))?;
        
        let Some(row) = row else { return Ok(None) };

        Ok(Some(ScrapeBatch {
            batch_id: row.batch_id,
            entity: Entity::from_str(&row.entity)
                .map_err(|_| ScrapedBatchModelError::NotAnEntity)?,
            author: row.author,
            region: row.region,
            scraped_at: row.scraped_at,
            checksum: row.checksum,
        }))
    }
}
