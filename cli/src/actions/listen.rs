
use htc::{buffet::BuffetClient, client::HTCClient, models::Entity, orders::Order};

use crate::actions::{meals::MealsAction, restaurants::RestaurantsAction};

pub struct ListenAction {
    buffet_client: BuffetClient,
    htc_client: HTCClient,
}

#[derive(Debug, thiserror::Error)]
pub enum ListenActionError {
    #[error("Couldn't poll for new data : {0}")]
    PollFailed(String),
    #[error("Couldn't scrape restaurants : {0}")]
    ScrapeRestaurantsFailed(String),
    #[error("Couldn't scrape meals : {0}")]
    ScrapeMealsFailed(String),
}

impl ListenAction {
    pub fn new(buffet_client: BuffetClient, htc_client: HTCClient) -> Self {
        ListenAction {
            buffet_client,
            htc_client,
        }
    }

    pub async fn run(&self) -> Result<(), ListenActionError> {
        loop {
            if let Some(order) = self.poll().await? {
                match order.entity {
                    Entity::Restaurants => self.scrape_restaurants(order).await?,
                    Entity::Meals(_) => self.scrape_meals(order).await?,
                    Entity::Schools => {}
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    }

    async fn poll(&self) -> Result<Option<Order>, ListenActionError> {
        self.buffet_client
            .get_next_job()
            .await
            .map_err(|e| ListenActionError::PollFailed(e.to_string()))
    }

    async fn scrape_meals(&self, order: Order) -> Result<(), ListenActionError> {
        let region = order.target;
        let meals_action = MealsAction::new(region, false, self.htc_client.clone());
        let meals_by_restaurant = meals_action
            .collect()
            .await
            .map_err(|e| ListenActionError::ScrapeMealsFailed(e.to_string()))?;
        for meals in meals_by_restaurant {
            if !meals.is_empty() {
                self.buffet_client
                    .put_meals(meals, order.clone())
                    .await
                    .map_err(|e| ListenActionError::ScrapeMealsFailed(e.to_string()))?;
            }
        }
        Ok(())
    }

    async fn scrape_restaurants(&self, order: Order) -> Result<(), ListenActionError> {
        let region = order.target;
        let restaurants_action = RestaurantsAction::new(region, true, self.htc_client.clone());
        let restaurants = restaurants_action
            .collect()
            .await
            .map_err(|e| ListenActionError::ScrapeRestaurantsFailed(e.to_string()))?;
        self.buffet_client
            .put_restaurants(restaurants, order.clone())
            .await
            .map_err(|e| ListenActionError::ScrapeRestaurantsFailed(e.to_string()))?;
        Ok(())
    }
}
