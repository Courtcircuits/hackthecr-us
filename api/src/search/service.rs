use std::sync::Arc;

use htc::{
    models::{keywords::KeywordModel, restaurants::Restaurant},
    regions::CrousRegion,
};
use sqlx::PgPool;
use thiserror::Error;
use tracing::instrument;

use crate::{
    batches::service::BatchesServiceImpl,
    restaurants::service::{RestaurantsService, RestaurantsServiceImpl}, search::clean_word,
};

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("Database error : {0}")]
    DatabaseError(String),
}

#[derive(Clone)]
pub struct SearchServiceImpl {
    pool: Arc<PgPool>,
    restaurant_service: Arc<RestaurantsServiceImpl<BatchesServiceImpl>>,
}

pub trait SearchService {
    fn search_restaurant(
        &self,
        query: String,
        region: CrousRegion,
    ) -> impl Future<Output = Result<Vec<Restaurant>, SearchError>> + Send;
}

impl SearchService for SearchServiceImpl {
    #[instrument(skip(self), fields(region=%region), err)]
    async fn search_restaurant(
        &self,
        query: String,
        region: CrousRegion,
    ) -> Result<Vec<Restaurant>, SearchError> {
        let ids = self
            .pool
            .query_restaurant(clean_word(query), region)
            .await
            .map_err(SearchError::DatabaseError)?;
        self.restaurant_service
            .get_restaurants_by_ids(ids)
            .await
            .map_err(|e| SearchError::DatabaseError(e.to_string()))
    }
}

impl SearchServiceImpl {
    pub fn new(
        pool: Arc<PgPool>,
        restaurant_service: Arc<RestaurantsServiceImpl<BatchesServiceImpl>>,
    ) -> Self {
        Self {
            pool,
            restaurant_service,
        }
    }
}
