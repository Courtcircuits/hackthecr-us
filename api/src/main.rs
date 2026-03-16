use std::sync::Arc;
use tracing::{error, info};

use clap::Parser as _;

use crate::{
    admins::service::{AdminService, AdminServiceImpl},
    app::AppImpl,
    config::Config,
    restaurants::service::RestaurantsServiceImpl,
    router::root,
};

pub mod app;
pub mod config;
pub mod error;
pub mod http;
pub mod restaurants;
pub mod router;
pub mod admins;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let config = Config::parse();
    let config = Arc::new(config);

    let pool = sqlx::PgPool::connect(&config.database_url).await?;
    let pool = Arc::new(pool);

    let restaurants_service = RestaurantsServiceImpl::new(pool.clone());
    let admin_service = AdminServiceImpl::new(pool.clone());
    let key = config.admin_public_key.clone();

    if !key.is_empty() {
        info!("Default key found");
        let _ = admin_service.create_default_admin_key(&key).await.map_err(|e| {
            tracing::error!("{}", e.to_string());
        });
        info!("Default admin key inserted");
    }else {
        info!("No default key found");
    }



    let app = AppImpl::new(restaurants_service, admin_service, config.clone());
    let root = root(app).await.map_err(|e| {
        error!("Failed to create router: {}", e);
        e
    })?;

    let _ = crate::http::serve(root, config)
        .await
        .inspect_err(|e| error!("{}", e));

    Ok(())
}
