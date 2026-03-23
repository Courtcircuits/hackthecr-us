use std::sync::Arc;

use htc::{
    models::{
        Entity,
        scrape_batch::{ScrapeBatch, ScrapedBatchModel, ScrapedBatchModelError},
    },
    regions::CrousRegion,
};
use sqlx::{PgPool, PgTransaction};
use tracing::instrument;
use uuid::Uuid;

#[derive(Clone)]
pub struct BatchesServiceImpl {
    pool: Arc<PgPool>,
}

pub trait BatchesService {
    fn create_batch(
        &'_ self,
        entity: Entity,
        author: Uuid,
        region: CrousRegion,
        checksum: String,
    ) -> impl Future<Output = Result<(Uuid, PgTransaction<'_>), ScrapedBatchModelError>> + Send;

    fn current_batch(
        &self,
        entity: &Entity,
        region: CrousRegion,
    ) -> impl Future<Output = Result<Option<ScrapeBatch>, ScrapedBatchModelError>> + Send;
}

impl BatchesService for BatchesServiceImpl {
    #[instrument(skip(self), fields(region=%region), err)]
    async fn create_batch(
        &'_ self,
        entity: Entity,
        author_id: Uuid,
        region: CrousRegion,
        checksum: String,
    ) -> Result<(Uuid, PgTransaction<'_>), ScrapedBatchModelError> {
        let current_batch = self.current_batch(&entity, region).await?;

        if let Some(current_batch) = current_batch
            && checksum == current_batch.checksum
        {
            return Err(ScrapedBatchModelError::NoDriftWithCurrentBatch);
        }

        let batch_uuid = uuid::Uuid::new_v4();
        let tx = self
            .pool
            .create_batch(ScrapeBatch {
                batch_id: batch_uuid,
                entity,
                author: author_id,
                scraped_at: None,
                region: region.to_string(),
                checksum,
            })
            .await?;
        Ok((batch_uuid, tx))
    }

    async fn current_batch(
        &self,
        entity: &Entity,
        region: CrousRegion,
    ) -> Result<Option<ScrapeBatch>, ScrapedBatchModelError> {
        self.pool.current_batch(entity, region).await
    }
}

impl BatchesServiceImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}
