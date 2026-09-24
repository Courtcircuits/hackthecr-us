use std::sync::Arc;

use ::tracing::error;
use clap::Parser;
use r2d2::Pool;

use crate::{app::App, config::Config, tracing::init_tracing_subscriber};

mod config;
mod http;
mod tracing;
mod app;
mod jobs;
mod router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = init_tracing_subscriber();
    dotenv::dotenv().ok();
    let config = Arc::new(Config::parse());

    let redis_client = redis::Client::open(config.redis_url.clone())?;
    let redis_pool = Arc::new(Pool::builder().build(redis_client)?);

    let app_state = App::new(redis_pool);
    let router = router::root(app_state, &config.cors_origins)
        .inspect_err(|e| error!("Failed to build router: {}", e))?;

    let http_server = crate::http::serve(router, config)
        .await
        .inspect_err(|e| error!("Failed to start HTTP server: {}", e))?;

    http_server.await?;
    Ok(())
}
