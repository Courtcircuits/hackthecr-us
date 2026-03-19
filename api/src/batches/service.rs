use std::sync::Arc;

use htc::{models::{
    scrape_batch::{ScrapeBatch, ScrapedBatchModel, ScrapedBatchModelError}, Entity
}, regions::CrousRegion};
use sqlx::PgPool;
use uuid::Uuid;


#[derive(Clone)]
pub struct BatchesServiceImpl {
    pool: Arc<PgPool>,
}

pub trait BatchesService {
    fn create_batch(
        &self,
        entity: Entity,
        author: Uuid,
        region: CrousRegion
    ) -> impl Future<Output = Result<Uuid, ScrapedBatchModelError>> + Send;

    fn current_batch(&self, entity: Entity, region: CrousRegion) -> impl Future<Output = Result<Uuid, ScrapedBatchModelError>> + Send;
}

impl BatchesService for BatchesServiceImpl {
    async fn create_batch(
        &self,
        entity: Entity,
        author_id: Uuid,
        region: CrousRegion
    ) -> Result<Uuid, ScrapedBatchModelError> {
        let batch_uuid = uuid::Uuid::new_v4();
        self.pool.create_batch(ScrapeBatch {
            batch_id: batch_uuid,
            entity,
            author: author_id,
            scraped_at: None,
            region: region.to_string()
        }).await?;
        Ok(batch_uuid)
    }

    async fn current_batch(&self, entity: Entity, region: CrousRegion) -> Result<Uuid, ScrapedBatchModelError> {
        self.pool.current_batch(entity, region).await
    }
}

impl BatchesServiceImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}
