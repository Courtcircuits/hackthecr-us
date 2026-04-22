use std::sync::Arc;

pub struct App {
    pub redis_pool: Arc<r2d2::Pool>,
}

pub enum AppError {
    RedisError(String),
}

impl App {
    pub fn new(redis_pool: Arc<r2d2::Pool>) -> Self {
        Self { redis_pool }
    }

    pub async fn poll_job(&self) -> Result<Option<String>, AppError> {
        let mut conn = self.redis_pool.get().map_err(|e| {
            AppError::RedisError({
                error!("Failed to get Redis connection: {}", e);
                format!("Failed to get Redis connection: {}", e)
            })
        })?;

        let job: Option<String> = redis::cmd("BLPOP")
            .arg("job_queue")
            .arg(0)
            .query_async(&mut *conn)
            .await
            .map_err(|e| {
                AppError::RedisError({
                    error!("Failed to poll job from Redis: {}", e);
                    format!("Failed to poll job from Redis: {}", e)
                })
            })?;
    }
}
