use std::sync::Arc;

use futures::future::join_all;
use htc::scheduler::{ExecutionResult, SchedulableAction};
use htc::client::HTCClient;

use htc::config::{ConfigError, CronConfig};

use super::{meals::MealsAction, restaurants::RestaurantsAction};

pub struct ScheduleAction {
    restaurants: Vec<Arc<SchedulableAction<RestaurantsAction>>>,
    meals: Vec<Arc<SchedulableAction<MealsAction>>>,
}

impl ScheduleAction {
    pub fn try_from_config(
        cron: CronConfig,
        client: HTCClient,
    ) -> Result<Self, ConfigError<'static>> {
        let mut restaurants = Vec::new();
        let mut meals = Vec::new();
        if let Some(restaurants_config) = cron.restaurants {
            for target in restaurants_config.target {
                println!("Scheduling restaurant crawl job for {}", target);
                let action = RestaurantsAction::new(target, false, client.clone());
                restaurants.push(Arc::new(SchedulableAction::new(
                    action,
                    restaurants_config.schedule.clone(),
                )));
            }
        }

        if let Some(meals_config) = cron.meals {
            for target in meals_config.target {
                println!("Scheduling meals crawl job for {}", target);
                let action = MealsAction::new(target, false, client.clone());
                meals.push(Arc::new(SchedulableAction::new(
                    action,
                    meals_config.schedule.clone(),
                )));
            }
        }

        Ok(Self { restaurants, meals })
    }

    pub async fn schedule(&self) -> Result<(), ExecutionResult> {
        let mut handles = Vec::new();

        for restaurant in &self.restaurants {
            let restaurant = restaurant.clone();
            let handle = tokio::spawn(async move { restaurant.schedule().await.unwrap() });
            handles.push(handle)
        }

        for meal in &self.meals {
            let meal = meal.clone();
            let handle = tokio::spawn(async move { meal.schedule().await.unwrap() });
            handles.push(handle)
        }

        let _ = join_all(handles).await;
        Ok(())
    }
}
