use ::tracing::error;
use clap::Parser;

use crate::{config::Config, tracing::init_tracing_subscriber};

mod config;
mod http;
mod tracing;
mod app;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = init_tracing_subscriber();
    dotenv::dotenv().ok();
    let config = Config::parse();

    let http_server = crate::http::serve(app, config)
        .await
        .inspect_err(|e| error!("Failed to start HTTP server: {}", e))?;
    Ok(())
}
