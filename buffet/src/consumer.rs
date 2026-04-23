use std::future::Future;

use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer as RdConsumer, StreamConsumer};
use rdkafka::message::Message;
use serde::de::DeserializeOwned;
use tokio::sync::watch;
use tracing::{info, warn};

pub trait MessageHandler<T> {
    fn handle(&self, message: T) -> impl Future<Output = ()>;
}

pub struct Consumer<T, H>
where
    T: DeserializeOwned,
    H: MessageHandler<T> + Send + Sync,
{
    pub topic: String,
    pub group_id: String,
    pub broker: String,
    pub _marker: std::marker::PhantomData<T>,
    pub handler: H,
}

impl<T, H> Consumer<T, H>
where
    T: DeserializeOwned + Send,
    H: MessageHandler<T> + Send + Sync,
{
    pub fn new(topic: String, group_id: String, broker: String, handler: H) -> Self {
        Self {
            topic,
            group_id,
            broker,
            _marker: std::marker::PhantomData,
            handler,
        }
    }

    async fn ensure_topic(&self) {
        let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
            .set("bootstrap.servers", &self.broker)
            .create()
            .expect("AdminClient creation failed");

        let new_topic = NewTopic::new(&self.topic, 1, TopicReplication::Fixed(1));
        let results = admin
            .create_topics(&[new_topic], &AdminOptions::new())
            .await
            .expect("Topic creation request failed");

        for result in results {
            match result {
                Ok(name) => info!("Topic '{}' created", name),
                Err((name, rdkafka::error::RDKafkaErrorCode::TopicAlreadyExists)) => {
                    info!("Topic '{}' already exists", name)
                }
                Err((name, e)) => warn!("Failed to create topic '{}': {:?}", name, e),
            }
        }
    }

    pub async fn run(&self, mut shutdown: watch::Receiver<bool>) {
        self.ensure_topic().await;

        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", &self.group_id)
            .set("bootstrap.servers", &self.broker)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .create()
            .expect("Consumer creation failed");

        consumer
            .subscribe(&[self.topic.as_str()])
            .expect("Can't subscribe to specified topic");

        info!(topic = %self.topic, "Consumer started");

        loop {
            tokio::select! {
                result = consumer.recv() => {
                    match result {
                        Err(e) => warn!("Kafka error: {}", e),
                        Ok(m) => {
                            let payload = match m.payload_view::<str>() {
                                None => {
                                    warn!("Empty payload, skipping");
                                    consumer.commit_message(&m, CommitMode::Async).unwrap();
                                    continue;
                                }
                                Some(Ok(s)) => s,
                                Some(Err(e)) => {
                                    warn!("Error deserializing message payload: {:?}", e);
                                    consumer.commit_message(&m, CommitMode::Async).unwrap();
                                    continue;
                                }
                            };

                            info!(
                                topic = m.topic(),
                                partition = m.partition(),
                                offset = m.offset(),
                                "Message received"
                            );

                            match serde_json::from_str::<T>(payload) {
                                Ok(message) => self.handler.handle(message).await,
                                Err(e) => warn!("Failed to deserialize message as {}: {:?}", std::any::type_name::<T>(), e),
                            }

                            consumer.commit_message(&m, CommitMode::Async).unwrap();
                        }
                    }
                }
                _ = shutdown.changed() => {
                    info!(topic = %self.topic, "Shutdown signal received, stopping consumer");
                    break;
                }
            }
        }
    }
}
