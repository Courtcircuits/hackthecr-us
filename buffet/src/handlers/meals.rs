use htc::{buffet::ScrapingResult, models::meals::MealSchema, verifiable::SignedPayload};
use tracing::info;
use uuid::Uuid;

use crate::{consumer::MessageHandler, store::ClickHouseStore};

pub struct MealsHandler {
    pub store: ClickHouseStore,
}

impl MessageHandler<SignedPayload<ScrapingResult<Vec<MealSchema>>>> for MealsHandler {
    async fn handle(&self, message: SignedPayload<ScrapingResult<Vec<MealSchema>>>) {
        let scraper_id = &message.author;
        let order = &message.payload.order;
        let meals = &message.payload.data;

        info!(
            scraper_id,
            region = %order.target,
            count = meals.len(),
            "Received meals batch"
        );

        self.store
            .handle_meals(
                Uuid::new_v4(),
                order.job_id,
                scraper_id,
                &order.target.to_string(),
                meals,
            )
            .await;
    }
}
