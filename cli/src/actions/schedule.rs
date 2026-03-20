use std::sync::Arc;

use chrono::Utc;
use cron_parser::parse;
use futures::future::join_all;
use htc::regions::CrousRegion;

use crate::{
    actions::{Executable, ExecutionResult},
    client::HTCClient,
    config::{ConfigError, CronConfig},
};

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
                let region: CrousRegion = target
                    .parse()
                    .map_err(|_| ConfigError::UnknownRegion(target))?;
                println!("Scheduling restaurant crawl job for {}", region);
                let action = RestaurantsAction::new(region, false, client.clone());
                restaurants.push(Arc::new(SchedulableAction::new(
                    action,
                    restaurants_config.schedule.clone(),
                )));
            }
        }

        if let Some(meals_config) = cron.meals {
            for target in meals_config.target {
                let region: CrousRegion = target
                    .parse()
                    .map_err(|_| ConfigError::UnknownRegion(target))?;
                println!("Scheduling meals crawl job for {}", region);
                let action = MealsAction::new(region, false, client.clone());
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

pub struct SchedulableAction<A>
where
    A: Executable,
{
    executable: A,
    schedule: String,
}

impl<A> SchedulableAction<A>
where
    A: Executable,
{
    pub fn new(executable: A, schedule: String) -> Self {
        Self {
            executable,
            schedule,
        }
    }
    pub async fn schedule(&self) -> Result<(), ExecutionResult> {
        loop {
            let now = Utc::now();
            let next = parse(&self.schedule, &now)
                .map_err(|e| ExecutionResult::Failure(format!("Invalid cron expression: {e}")))?;
            let delay = next - now;
            println!("Next schedule in {}", delay);
            if let Ok(std_duration) = delay.to_std() {
                tokio::select! {
                    _ = tokio::time::sleep(std_duration) => {}
                    _ = tokio::signal::ctrl_c() => { return Ok(()); }
                }
            }
            self.executable.execute().await?;
        }
    }
}
