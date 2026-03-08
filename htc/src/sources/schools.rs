use scraper::school_api::ApiSchool;
use sqlx::types::uuid;

use crate::models::schools::School;

pub struct SchoolApiScrapedData {
    pub api_data: ApiSchool,
}

impl Into<School> for SchoolApiScrapedData {
    fn into(self) -> School {
        School {
            school_id: uuid::uuid!("00000000-0000-0000-0000-000000000000"),
            long_name: self.api_data.nom.clone(),
            name: self.api_data.sigle.unwrap_or_else(|| self.api_data.nom[..4].to_string()),
            coordinates: Some(format!("{},{}", self.api_data.point_geo.lat, self.api_data.point_geo.lon)),
        }
    }
}
