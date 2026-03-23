use ::tracing::{error, info};
use std::sync::Arc;

use clap::Parser as _;

use crate::{
    admins::service::{AdminService, AdminServiceImpl},
    app::AppImpl,
    batches::service::BatchesServiceImpl,
    config::Config,
    events::{EventListener, scraping_channel::ScrapingChannel},
    meals::service::MealsServiceImpl,
    restaurants::service::RestaurantsServiceImpl,
    router::root,
    sse::SseState,
    tracing::init_tracing_subscriber,
};

pub mod admins;
pub mod app;
pub mod batches;
pub mod config;
pub mod error;
pub mod events;
pub mod http;
pub mod meals;
pub mod restaurants;
pub mod router;
pub mod sse;
pub mod tracing;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = init_tracing_subscriber();
    dotenv::dotenv().ok();

    let config = Config::parse();
    let config = Arc::new(config);

    let pool = sqlx::PgPool::connect(&config.database_url).await?;
    let pool = Arc::new(pool);

    let batch_service = Arc::new(BatchesServiceImpl::new(pool.clone()));
    let restaurants_service = RestaurantsServiceImpl::new(pool.clone(), batch_service.clone());
    let meals_service = MealsServiceImpl::new(pool.clone(), batch_service.clone());
    let admin_service = AdminServiceImpl::new(pool.clone());
    let key = config.admin_public_key.clone();

    if !key.is_empty() {
        info!("Default key found");
        let _ = admin_service
            .create_default_admin_key(&key)
            .await
            .map_err(|e| {
                error!("{}", e.to_string());
            });
        info!("Default admin key inserted");
    } else {
        info!("No default key found");
    }

    let (sse_state, sse_sender) = SseState::new(config.sse_token.clone());
    let sse_state = Arc::new(sse_state);

    let event_handler = Arc::new(ScrapingChannel { sender: sse_sender });
    let event_listener = EventListener::new(event_handler, pool.clone());

    let app = AppImpl::new(
        restaurants_service,
        meals_service,
        admin_service,
        batch_service,
        config.clone(),
    );
    let root = root(app, sse_state).await.map_err(|e| {
        error!("Failed to create router: {}", e);
        e
    })?;

    let http_server = crate::http::serve(root, config)
        .await
        .inspect_err(|e| error!("{}", e))?;

    let listener_handle = event_listener
        .listen("scraping_channel".to_string())
        .await
        .map_err(|e| {
            error!("Failed to create listener: {}", e);
            e
        })?;

    let (http_result, listener_result) = tokio::join!(http_server, listener_handle);
    if let Err(e) = http_result {
        error!("HTTP server error: {}", e);
    }
    if let Err(e) = listener_result {
        error!("Event listener error: {}", e);
    }

    Ok(())
}
