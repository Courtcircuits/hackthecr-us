use clickhouse::{Client, Row};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, warn};

#[derive(Clone)]
pub struct ClickHouseStore {
    client: Client,
}

const SCHEMA: &str = include_str!("../../clickhouse/schema.sql");

fn schema_statements() -> Vec<String> {
    let stripped = SCHEMA
        .lines()
        .filter(|l| !l.trim_start().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");

    stripped
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

#[derive(Row, Serialize)]
struct BatchRow {
    batch_id:   Uuid,
    job_id:     Uuid,
    scraper_id: String,
    region:     String,
    entity:     String,
}

#[derive(Row, Serialize)]
struct RestaurantRow {
    batch_id:      Uuid,
    job_id:        Uuid,
    scraper_id:    String,
    region:        String,
    restaurant_id: String,
    name:          String,
    url:           String,
    city:          Option<String>,
    coordinates:   Option<String>,
    opening_hours: Option<String>,
}

#[derive(Row, Serialize)]
struct MealRow {
    batch_id:      Uuid,
    job_id:        Uuid,
    scraper_id:    String,
    region:        String,
    restaurant_id: String,
    meal_type:     String,
    foodies:       Option<String>,
    date:          Option<String>,
}

#[derive(Row, Deserialize)]
struct Count {
    count: u64,
}

impl ClickHouseStore {
    pub fn new(url: &str) -> Self {
        Self {
            client: Client::default().with_url(url),
        }
    }

    pub async fn migrate(&self) -> Result<(), clickhouse::error::Error> {
        for stmt in schema_statements() {
            self.client.query(&stmt).execute().await?;
        }
        info!("ClickHouse migrations applied");
        Ok(())
    }

    async fn insert_batch(
        &self,
        batch_id: Uuid,
        job_id: Uuid,
        scraper_id: &str,
        region: &str,
        entity: &str,
    ) -> Result<(), clickhouse::error::Error> {
        let mut insert = self.client.insert("scraping_batches")?;
        insert.write(&BatchRow {
            batch_id,
            job_id,
            scraper_id: scraper_id.to_string(),
            region:     region.to_string(),
            entity:     entity.to_string(),
        }).await?;
        insert.end().await
    }

    async fn insert_restaurants(
        &self,
        batch_id: Uuid,
        job_id: Uuid,
        scraper_id: &str,
        region: &str,
        restaurants: &[htc::models::restaurants::RestaurantSchema],
    ) -> Result<(), clickhouse::error::Error> {
        if restaurants.is_empty() {
            return Ok(());
        }
        let mut insert = self.client.insert("scraped_restaurants")?;
        for r in restaurants {
            insert.write(&RestaurantRow {
                batch_id,
                job_id,
                scraper_id:    scraper_id.to_string(),
                region:        region.to_string(),
                restaurant_id: r.id.clone(),
                name:          r.name.clone(),
                url:           r.url.clone(),
                city:          r.city.clone(),
                coordinates:   r.coordinates.clone(),
                opening_hours: r.opening_hours.clone(),
            }).await?;
        }
        insert.end().await
    }

    async fn insert_meals(
        &self,
        batch_id: Uuid,
        job_id: Uuid,
        scraper_id: &str,
        region: &str,
        meals: &[htc::models::meals::MealSchema],
    ) -> Result<(), clickhouse::error::Error> {
        if meals.is_empty() {
            return Ok(());
        }
        let mut insert = self.client.insert("scraped_meals")?;
        for m in meals {
            insert.write(&MealRow {
                batch_id,
                job_id,
                scraper_id:    scraper_id.to_string(),
                region:        region.to_string(),
                restaurant_id: m.restaurant_id.clone(),
                meal_type:     m.meal_type.clone(),
                foodies:       m.foodies.clone(),
                date:          m.date.clone(),
            }).await?;
        }
        insert.end().await
    }

    async fn batch_count(&self, job_id: Uuid) -> Result<u64, clickhouse::error::Error> {
        let row = self.client
            .query("SELECT count() AS count FROM scraping_batches WHERE job_id = ?")
            .bind(job_id.to_string())
            .fetch_one::<Count>()
            .await?;
        Ok(row.count)
    }

    async fn compute_jaccard_restaurants(&self, job_id: Uuid) -> Result<(), clickhouse::error::Error> {
        let sql = format!(
            "INSERT INTO batch_jaccard_scores \
             SELECT a.job_id, 'restaurants' AS entity, \
               if(a.scraper_id < b.scraper_id, a.scraper_id, b.scraper_id) AS scraper_a, \
               if(a.scraper_id < b.scraper_id, b.scraper_id, a.scraper_id) AS scraper_b, \
               toFloat32(length(arrayIntersect(a.ids, b.ids))) \
                 / length(arrayDistinct(arrayConcat(a.ids, b.ids))) AS jaccard, \
               now() AS computed_at \
             FROM \
               (SELECT job_id, scraper_id, groupArray(restaurant_id) AS ids \
                FROM scraped_restaurants WHERE job_id = '{job_id}' \
                GROUP BY job_id, scraper_id) a \
             JOIN \
               (SELECT job_id, scraper_id, groupArray(restaurant_id) AS ids \
                FROM scraped_restaurants WHERE job_id = '{job_id}' \
                GROUP BY job_id, scraper_id) b USING (job_id) \
             WHERE a.scraper_id < b.scraper_id"
        );
        self.client.query(&sql).execute().await
    }

    async fn compute_jaccard_meals(&self, job_id: Uuid) -> Result<(), clickhouse::error::Error> {
        let sql = format!(
            "INSERT INTO batch_jaccard_scores \
             SELECT a.job_id, 'meals' AS entity, \
               if(a.scraper_id < b.scraper_id, a.scraper_id, b.scraper_id) AS scraper_a, \
               if(a.scraper_id < b.scraper_id, b.scraper_id, a.scraper_id) AS scraper_b, \
               toFloat32(length(arrayIntersect(a.ids, b.ids))) \
                 / length(arrayDistinct(arrayConcat(a.ids, b.ids))) AS jaccard, \
               now() AS computed_at \
             FROM \
               (SELECT job_id, scraper_id, \
                  groupArray(concat(restaurant_id,'|',meal_type,'|',coalesce(date,''))) AS ids \
                FROM scraped_meals WHERE job_id = '{job_id}' \
                GROUP BY job_id, scraper_id) a \
             JOIN \
               (SELECT job_id, scraper_id, \
                  groupArray(concat(restaurant_id,'|',meal_type,'|',coalesce(date,''))) AS ids \
                FROM scraped_meals WHERE job_id = '{job_id}' \
                GROUP BY job_id, scraper_id) b USING (job_id) \
             WHERE a.scraper_id < b.scraper_id"
        );
        self.client.query(&sql).execute().await
    }

    pub async fn handle_restaurants(
        &self,
        batch_id: Uuid,
        job_id: Uuid,
        scraper_id: &str,
        region: &str,
        restaurants: &[htc::models::restaurants::RestaurantSchema],
    ) {
        if let Err(e) = self.insert_batch(batch_id, job_id, scraper_id, region, "restaurants").await {
            warn!(scraper_id, %job_id, "insert_batch failed: {e}");
            return;
        }
        if let Err(e) = self.insert_restaurants(batch_id, job_id, scraper_id, region, restaurants).await {
            warn!(scraper_id, %job_id, "insert_restaurants failed: {e}");
            return;
        }
        match self.batch_count(job_id).await {
            Ok(n) if n >= 2 => {
                info!(scraper_id, %job_id, batches = n, "Computing Jaccard for restaurants");
                if let Err(e) = self.compute_jaccard_restaurants(job_id).await {
                    warn!(%job_id, "Jaccard computation failed: {e}");
                }
            }
            Ok(_) => {}
            Err(e) => warn!(%job_id, "batch_count failed: {e}"),
        }
    }

    pub async fn handle_meals(
        &self,
        batch_id: Uuid,
        job_id: Uuid,
        scraper_id: &str,
        region: &str,
        meals: &[htc::models::meals::MealSchema],
    ) {
        if let Err(e) = self.insert_batch(batch_id, job_id, scraper_id, region, "meals").await {
            warn!(scraper_id, %job_id, "insert_batch failed: {e}");
            return;
        }
        if let Err(e) = self.insert_meals(batch_id, job_id, scraper_id, region, meals).await {
            warn!(scraper_id, %job_id, "insert_meals failed: {e}");
            return;
        }
        match self.batch_count(job_id).await {
            Ok(n) if n >= 2 => {
                info!(scraper_id, %job_id, batches = n, "Computing Jaccard for meals");
                if let Err(e) = self.compute_jaccard_meals(job_id).await {
                    warn!(%job_id, "Jaccard computation failed: {e}");
                }
            }
            Ok(_) => {}
            Err(e) => warn!(%job_id, "batch_count failed: {e}"),
        }
    }
}
