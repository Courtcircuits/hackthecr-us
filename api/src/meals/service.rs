use std::sync::Arc;

use htc::{
    models::{
        Entity,
        admins::Admin,
        keywords::{self, Category, Keyword, KeywordModel},
        meals::{Meal, MealModel as _, MealModelError, MealSchema},
        scrape_batch::ScrapedBatchModelError,
    },
    regions::CrousRegion,
};
use sqlx::PgPool;
use tracing::instrument;
use uuid::Uuid;

use crate::{batches::service::{BatchesService, BatchesServiceImpl}, search::clean_word};

pub trait MealsService {
    fn save_meals(
        &self,
        meals: &[MealSchema],
        admin: Admin,
        region: CrousRegion,
        checksum: String,
    ) -> impl Future<Output = Result<(), MealModelError>> + Send;
    fn get_meals_by_restaurant_id(
        &self,
        name: String,
        region: CrousRegion,
    ) -> impl Future<Output = Result<Vec<Meal>, MealModelError>> + Send;
}

#[derive(Clone)]
pub struct MealsServiceImpl<B>
where
    B: BatchesService,
{
    pool: Arc<PgPool>,
    batch_service: Arc<B>,
}

impl MealsService for MealsServiceImpl<BatchesServiceImpl> {
    #[instrument(skip(self), fields(region=%region), err)]
    async fn save_meals(
        &self,
        meals: &[MealSchema],
        admin: Admin,
        region: CrousRegion,
        checksum: String,
    ) -> Result<(), MealModelError> {
        let Some(first_meal) = meals.first() else {
            return Err(MealModelError::EmptyBody);
        };

        let restaurant_id = first_meal.restaurant_id.clone();
        let (batch, mut tx) = self
            .batch_service
            .create_batch(
                Entity::Meals(restaurant_id),
                admin.admin_id,
                region,
                checksum,
            )
            .await
            .map_err(|e| match e {
                ScrapedBatchModelError::NoDriftWithCurrentBatch => MealModelError::SyncSkipped,
                _ => MealModelError::DatabaseError(e.to_string()),
            })?;

        let meals: Vec<Meal> = meals
            .iter()
            .map(|schema| Meal {
                meal_id: Uuid::new_v4(),
                meal_type: schema.meal_type.clone(),
                foodies: schema.foodies.clone(),
                date: schema.date.clone(),
                restaurant_id: schema.restaurant_id.clone(),
                batch_id: batch,
            })
            .collect();

        for meal in meals {
            let keywords = self.create_keywords(&meal, region);
            self.pool.create_meal(meal, &mut tx).await?;
            for keyword in keywords {
                self.pool
                    .create_keyword(keyword, &mut tx)
                    .await
                    .map_err(MealModelError::DatabaseError)?;
            }
        }

        tx.commit()
            .await
            .map_err(|e| MealModelError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn get_meals_by_restaurant_id(
        &self,
        name: String,
        region: CrousRegion,
    ) -> Result<Vec<Meal>, MealModelError> {
        let Some(current_batch) = self
            .batch_service
            .current_batch(&Entity::Meals(name.to_string()), region)
            .await
            .map_err(|_| MealModelError::NotFound)?
        else {
            return Err(MealModelError::NotFound);
        };
        self.pool
            .get_meals_by_restaurant_id_batch(name, current_batch.batch_id)
            .await
    }
}

impl<B> MealsServiceImpl<B>
where
    B: BatchesService,
{
    pub fn new(pool: Arc<PgPool>, batch_service: Arc<B>) -> Self {
        Self {
            pool,
            batch_service,
        }
    }

    pub fn create_keywords(&self, meal: &Meal, region: CrousRegion) -> Vec<Keyword> {
        let mut keywords = Vec::new();
        if let Some(foody) = &meal.foodies {
            keywords.push(Keyword {
                keyword_id: Uuid::new_v4(),
                keyword: clean_word(foody.clone()),
                restaurant_id: meal.restaurant_id.clone(),
                category: Category::Meal,
                region,
            });
            let words = foody.split(" ");
            for word in words {
                if word.len() > 2 {
                    // getting rid of words like "de", "a", "le" ...
                    keywords.push(Keyword {
                        keyword_id: Uuid::new_v4(),
                        keyword: clean_word(word.to_string()),
                        restaurant_id: meal.restaurant_id.clone(),
                        category: Category::Meal,
                        region,
                    });
                }
            }
        }

        keywords
    }
}
