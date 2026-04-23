use futures::future::join_all;
use std::{pin::Pin, sync::Arc};

use htc::{
    config::EntityScheduleConfig,
    models::Entity,
    orders::Order,
    regions::CrousRegion,
    scheduler::{Executable, ExecutionResult, SchedulableAction},
};

use crate::queue::OrderQueue;

pub struct Scheduler {
    pub restaurants_schedule: EntityScheduleConfig,
    pub meals_schedule: EntityScheduleConfig,
    pub order_queue: Arc<OrderQueue>,
}

impl Scheduler {
    pub fn new(
        restaurants_schedule: EntityScheduleConfig,
        meals_schedule: EntityScheduleConfig,
        order_queue: Arc<OrderQueue>,
    ) -> Self {
        Self {
            restaurants_schedule,
            meals_schedule,
            order_queue,
        }
    }

    pub async fn run(&self) -> Result<(), ExecutionResult> {
        let mut handles = Vec::new();

        for target in &self.restaurants_schedule.target {
            let action = EnqueueJob {
                target: *target,
                order_queue: self.order_queue.clone(),
                entity: Entity::Restaurants,
            };
            let schedulable =
                SchedulableAction::new(action, self.restaurants_schedule.schedule.clone());
            let handle = tokio::spawn(async move { schedulable.schedule().await.unwrap() });
            handles.push(handle);
        }

        for target in &self.meals_schedule.target {
            let action = EnqueueJob {
                target: *target,
                order_queue: self.order_queue.clone(),
                entity: Entity::Meals("*".to_string()),
            };
            let schedulable = SchedulableAction::new(action, self.meals_schedule.schedule.clone());
            let handle = tokio::spawn(async move { schedulable.schedule().await.unwrap() });
            handles.push(handle);
        }

        let _ = join_all(handles).await;
        Ok(())
    }
}

pub struct EnqueueJob {
    pub target: CrousRegion,
    pub entity: Entity,
    pub order_queue: Arc<OrderQueue>,
}

impl Executable for EnqueueJob {
    fn execute(
        &self,
    ) -> Pin<Box<dyn std::future::Future<Output = Result<(), ExecutionResult>> + Send + '_>> {
        Box::pin(async move {
            let order = Order::new(self.target, self.entity.clone());
            self.order_queue
                .enqueue(order)
                .await
                .map_err(ExecutionResult::Failure)?;
            Ok(())
        })
    }
}
