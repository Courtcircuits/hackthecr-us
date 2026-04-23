use std::{path::PathBuf, sync::Arc};

use clap::Parser;
use htc::config::HTCConfig;
use r2d2::Pool;
use redis::Client;
use tokio::sync::watch;
use tokio::signal;
use ::tracing::{error, info};

use crate::{
    config::Config,
    consumer::Consumer,
    handlers::{meals::MealsHandler, restaurants::RestaurantsHandler},
    scheduler::Scheduler,
    store::ClickHouseStore,
    tracing::init_tracing_subscriber,
};

pub mod config;
pub mod tracing;
pub mod queue;
pub mod scheduler;
pub mod consumer;
pub mod handlers;
pub mod store;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = init_tracing_subscriber();
    dotenv::dotenv().ok();

    let config = Arc::new(Config::parse());

    let htc_config_path = config.config.clone().unwrap_or_else(|| {
        let home = std::env::var("HOME").expect("HOME env var not set");
        PathBuf::from(home).join(".config/htc.yml")
    });

    let Ok(htc_config) = HTCConfig::from(&htc_config_path) else {
        panic!("Failed to load config from {}", htc_config_path.display());
    };

    let cron = htc_config.schedule.expect("No schedule configured in htc.yml");
    let restaurants_schedule = cron.restaurants.expect("No restaurants schedule configured");
    let meals_schedule = cron.meals.expect("No meals schedule configured");

    let redis_client = Client::open(config.redis_url.clone()).expect("Failed to create Redis client");
    let redis_pool = Arc::new(Pool::builder().build(redis_client)?);
    let order_queue = Arc::new(queue::OrderQueue::new(redis_pool));

    let scheduler = Scheduler::new(restaurants_schedule, meals_schedule, order_queue);

    let broker = config.producer_endpoint.clone();
    let store = ClickHouseStore::new(&config.clickhouse_url);
    store.migrate().await?;

    let meals_consumer = Consumer::new(
        "meals".to_string(),
        "buffet".to_string(),
        broker.clone(),
        MealsHandler { store: store.clone() },
    );

    let restaurants_consumer = Consumer::new(
        "restaurants".to_string(),
        "buffet".to_string(),
        broker,
        RestaurantsHandler { store },
    );

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let mut scheduler_shutdown = shutdown_rx.clone();
    let scheduler_handle = tokio::spawn(async move {
        tokio::select! {
            result = scheduler.run() => {
                if let Err(e) = result {
                    error!("Scheduler exited with error: {}", e);
                }
            }
            _ = scheduler_shutdown.changed() => {
                info!("Scheduler received shutdown signal");
            }
        }
    });

    let meals_shutdown = shutdown_rx.clone();
    let meals_handle = tokio::spawn(async move {
        meals_consumer.run(meals_shutdown).await;
    });

    let restaurants_shutdown = shutdown_rx;
    let restaurants_handle = tokio::spawn(async move {
        restaurants_consumer.run(restaurants_shutdown).await;
    });

    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
        .expect("Failed to register SIGTERM handler");

    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received SIGINT, initiating graceful shutdown");
        }
        _ = sigterm.recv() => {
            info!("Received SIGTERM, initiating graceful shutdown");
        }
    }

    shutdown_tx.send(true).expect("Failed to broadcast shutdown signal");
    info!("Waiting for all tasks to finish...");
    let _ = tokio::join!(scheduler_handle, meals_handle, restaurants_handle);
    info!("Buffet service shut down gracefully");

    Ok(())
}
