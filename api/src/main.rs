use std::sync::Arc;

use clap::Parser as _;

use crate::{
    app::AppImpl, config::Config, restaurants::service::RestaurantsServiceImpl, router::root,
};

pub mod app;
pub mod config;
pub mod error;
pub mod http;
pub mod restaurants;
pub mod router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let config = Config::parse();
    let config = Arc::new(config);

    let pool = sqlx::PgPool::connect(&config.database_url).await?;
    let pool = Arc::new(pool);

    let restaurants_service = RestaurantsServiceImpl::new(pool.clone());

    let app = AppImpl::new(restaurants_service, config.clone());
    let root = root(app).await.map_err(|e| {
        eprintln!("Failed to create router: {}", e);
        e
    })?;

    let _ = crate::http::serve(root, config)
        .await
        .inspect_err(|e| eprintln!("{}", e));

    Ok(())
}
