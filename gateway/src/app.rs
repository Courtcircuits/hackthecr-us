use std::sync::Arc;
use htc::orders::Order;
use tracing::error;

use r2d2::Pool;

use crate::http::ApiError;

#[derive(Clone)]
pub struct App {
    pub redis_pool: Arc<Pool<redis::Client>>,
}


impl App {
    pub fn new(redis_pool: Arc<Pool<redis::Client>>) -> Self {
        Self { redis_pool }
    }

    pub async fn poll_job(&self) -> Result<Option<Order>, ApiError> {

        let pool = self.redis_pool.clone();

        let job_str: Option<String> = tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| {
                error!("Failed to get Redis connection: {}", e);
                format!("Failed to get Redis connection: {}", e)
            })?;

            // checking queue length before blocking pop to avoid unnecessary blocking when queue
            // is empty
            let result: Option<isize> = redis::cmd("LLEN")
                .arg("job_queue")
                .query(&mut *conn)
                .map_err(|e| {
                    error!("Failed to check job queue length in Redis: {}", e);
                    format!("Failed to check job queue length in Redis: {}", e)
                })?;

            if result == Some(0) {
                return Ok(None);
            }


            let result: Option<(String, String)> = redis::cmd("BLPOP")
                .arg("job_queue")
                .arg(0)
                .query(&mut *conn)
                .map_err(|e| {
                    error!("Failed to poll job from Redis: {}", e);
                    format!("Failed to poll job from Redis: {}", e)
                })?;

            Ok::<_, String>(result.map(|(_, value)| value))
        })
        .await
        .map_err(|e| ApiError::InternalServerError(format!("Task join error: {}", e)))?
        .map_err(ApiError::NotFound)?;

        if let Some(s) = job_str {
            let order: Order = serde_json::from_str(&s).map_err(|e| {
                error!("Failed to parse job JSON: {}", e);
                ApiError::InternalServerError(format!("Failed to parse job JSON: {}", e))
            })?;
            Ok(Some(order))
        } else {
            Ok(None)
        }
    }
}
