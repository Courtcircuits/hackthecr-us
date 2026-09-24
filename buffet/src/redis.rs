use std::sync::Arc;

use htc::orders::Order;
use r2d2::Pool;


#[derive(Clone)]
pub struct OrderQueue {
    pub redis_pool: Arc<Pool<redis::Client>>
}

impl OrderQueue {
    pub fn new(redis_pool: Arc<Pool<redis::Client>>) -> Self {
        Self { redis_pool }
    }

    pub async fn enqueue(&self, order: Order) -> Result<(), String> {
        let pool = self.redis_pool.clone();
        let order_json = serde_json::to_string(&order).map_err(|e| format!("Failed to serialize order: {}", e))?;

        tokio::task::spawn_blocking(move || {
            let mut conn = pool.get().map_err(|e| format!("Failed to get Redis connection: {}", e))?;
            redis::cmd("RPUSH")
                .arg("job_queue")
                .arg(order_json)
                .query::<()>(&mut *conn)
                .map_err(|e| format!("Failed to enqueue job in Redis: {}", e))?;
            Ok::<_, String>(())
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
    }
}
