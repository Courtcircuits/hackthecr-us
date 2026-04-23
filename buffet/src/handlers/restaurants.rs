use htc::{buffet::ScrapingResult, models::restaurants::RestaurantSchema, verifiable::SignedPayload};
use tracing::info;
use uuid::Uuid;

use crate::{consumer::MessageHandler, store::ClickHouseStore};

pub struct RestaurantsHandler {
    pub store: ClickHouseStore,
}

impl MessageHandler<SignedPayload<ScrapingResult<Vec<RestaurantSchema>>>> for RestaurantsHandler {
    async fn handle(&self, message: SignedPayload<ScrapingResult<Vec<RestaurantSchema>>>) {
        let scraper_id = &message.author;
        let order = &message.payload.order;
        let restaurants = &message.payload.data;

        info!(
            scraper_id,
            region = %order.target,
            count = restaurants.len(),
            "Received restaurants batch"
        );

        self.store
            .handle_restaurants(
                Uuid::new_v4(),
                order.job_id,
                scraper_id,
                &order.target.to_string(),
                restaurants,
            )
            .await;
    }
}
