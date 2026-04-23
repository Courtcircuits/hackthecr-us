use reqwest::Client;
use utoipa::ToSchema;

use crate::{models::{meals::MealSchema, restaurants::RestaurantSchema}, orders::Order, verifiable::SignedPayload};

#[derive(Clone)]
pub struct BuffetClient {
    pub url: String,
    pub client: Client,
    pub private_key: String,
    pub author: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, ToSchema)]
pub struct ScrapingResult<T>
where
    T: serde::Serialize + ToSchema + Clone,
{
    pub data: T,
    pub order: Order,
}

#[derive(thiserror::Error, Debug)]
pub enum BuffetClientError {
    #[error("Couldn't put restaurant : {0}")]
    PutRestaurantFailed(String),
    #[error("Couldn't sign payload : {0}")]
    PayloadSigningFailed(String),
    #[error("Couldn't poll job : {0}")]
    PollJobFailed(String),
}

impl BuffetClient {
    pub fn new(url: String, private_key: String, author: String) -> Self {
        BuffetClient {
            url,
            client: reqwest::Client::new(),
            private_key,
            author,
        }
    }

    pub async fn put_restaurants(
        &self,
        restaurants: Vec<RestaurantSchema>,
        order: Order,
    ) -> Result<(), BuffetClientError> {
        let client = Client::new();

        let payload = SignedPayload::<ScrapingResult<Vec<RestaurantSchema>>>::sign(
            ScrapingResult {
                data: restaurants,
                order,
            },
            &self.private_key,
            &self.author,
        )
        .map_err(|e| BuffetClientError::PutRestaurantFailed(e.to_string()))?;

        let response = client
            .put(format!("{}/restaurants", self.url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| BuffetClientError::PutRestaurantFailed(e.to_string()))?;

        response
            .error_for_status()
            .map_err(|e| BuffetClientError::PutRestaurantFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn put_meals(
        &self,
        meals: Vec<MealSchema>,
        order: Order,
    ) -> Result<(), BuffetClientError> {
        let client = Client::new();

        let payload = SignedPayload::<ScrapingResult<Vec<MealSchema>>>::sign(
            ScrapingResult {
                data: meals,
                order,
            },
            &self.private_key,
            &self.author,
        )
        .map_err(|e| BuffetClientError::PutRestaurantFailed(e.to_string()))?;

        let response = client
            .put(format!("{}/meals", self.url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| BuffetClientError::PutRestaurantFailed(e.to_string()))?;

        response
            .error_for_status()
            .map_err(|e| BuffetClientError::PutRestaurantFailed(e.to_string()))?;

        Ok(())
    }

    pub async fn get_next_job(&self) -> Result<Option<Order>, BuffetClientError> {
        let response = self.client
            .post(format!("{}/jobs/next", self.url))
            .send()
            .await
            .map_err(|e| BuffetClientError::PollJobFailed(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let order = response
            .error_for_status()
            .map_err(|e| BuffetClientError::PollJobFailed(e.to_string()))?
            .json::<Order>()
            .await
            .map_err(|e| BuffetClientError::PollJobFailed(e.to_string()))?;

        Ok(Some(order))
    }
}
