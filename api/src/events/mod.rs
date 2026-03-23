use std::sync::Arc;

pub mod scraping_channel;

use serde::{Deserialize, de::DeserializeOwned};
use sqlx::{PgPool, postgres::PgListener};
use tracing::{error, instrument};

pub trait EventHandler {
    type Input: DeserializeOwned + Send;
    type Rejection;
    fn handle(
        &self,
        input: Self::Input,
    ) -> impl Future<Output = Result<(), Self::Rejection>> + Send;
}

pub struct EventListener<H>
where
    H: EventHandler,
{
    handler: Arc<H>,
    pool: Arc<PgPool>,
}

#[derive(thiserror::Error, Debug)]
pub enum EventError {
    #[error("Couldn't listen : {0}")]
    CouldntListen(String),
    #[error("Coudln't parse: {0}")]
    CouldntParse(String),
}

impl<H> EventListener<H>
where
    H: EventHandler + Send + Sync + 'static,
{
    pub fn new(handler: Arc<H>, pool: Arc<PgPool>) -> Self {
        Self { handler, pool }
    }
    #[instrument(skip(self), fields(topic=topic), err)]
    pub async fn listen(
        &self,
        topic: String,
    ) -> Result<tokio::task::JoinHandle<Result<(), EventError>>, EventError> {
        let mut listener = PgListener::connect_with(&self.pool)
            .await
            .map_err(|e| EventError::CouldntListen(e.to_string()))?;
        listener
            .listen(&topic)
            .await
            .map_err(|e| EventError::CouldntListen(e.to_string()))?;
        let handler = self.handler.clone();
        Ok(tokio::spawn(async move {
            loop {
                let Ok(notification) = listener.recv().await else {
                    error!("got an error on notification");
                    continue;
                };
                let payload: Notification<H::Input> = serde_json::from_str(notification.payload())
                    .map_err(|e| {
                        error!(
                            "Couldn't parse {:#?} because of {}",
                            notification.payload(),
                            e.to_string()
                        );
                        EventError::CouldntParse(e.to_string())
                    })?;
                let handler = handler.clone();
                let data = payload.data;
                tokio::spawn(async move {
                    let _ = handler.handle(data).await;
                });
            }
        }))
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Notification<T> {
    operation: String,
    table: String,
    data: T,
    entity: String,
}
