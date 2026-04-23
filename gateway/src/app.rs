use std::sync::Arc;
use std::time::Duration;

use htc::{
    buffet::ScrapingResult,
    models::{meals::MealSchema, restaurants::RestaurantSchema},
    orders::Order,
    verifiable::SignedPayload,
};
use r2d2::Pool;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use tracing::error;

use crate::http::ApiError;

#[derive(Clone)]
pub struct App {
    pub redis_pool: Arc<Pool<redis::Client>>,
    pub producer: FutureProducer,
}

impl App {
    pub fn new(redis_pool: Arc<Pool<redis::Client>>, broker: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", broker)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Failed to create Kafka producer");
        Self { redis_pool, producer }
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

    pub async fn produce_meals(
        &self,
        payload: SignedPayload<ScrapingResult<Vec<MealSchema>>>,
    ) -> Result<(), ApiError> {
        let payload_json = serde_json::to_string(&payload).map_err(|e| {
            ApiError::InternalServerError(format!("Failed to serialize meals payload: {}", e))
        })?;

        self.producer
            .send(
                FutureRecord::to("meals").payload(&payload_json).key(""),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(e, _)| {
                error!("Failed to produce meals message: {}", e);
                ApiError::InternalServerError(format!("Failed to forward meals to Kafka: {}", e))
            })?;

        Ok(())
    }

    pub async fn produce_restaurants(
        &self,
        payload: SignedPayload<ScrapingResult<Vec<RestaurantSchema>>>,
    ) -> Result<(), ApiError> {
        let payload_json = serde_json::to_string(&payload).map_err(|e| {
            ApiError::InternalServerError(format!("Failed to serialize restaurants payload: {}", e))
        })?;

        self.producer
            .send(
                FutureRecord::to("restaurants").payload(&payload_json).key(""),
                Duration::from_secs(5),
            )
            .await
            .map_err(|(e, _)| {
                error!("Failed to produce restaurants message: {}", e);
                ApiError::InternalServerError(format!(
                    "Failed to forward restaurants to Kafka: {}",
                    e
                ))
            })?;

        Ok(())
    }
}
