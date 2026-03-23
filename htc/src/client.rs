use crate::{
    models::{meals::MealSchema, restaurants::RestaurantSchema},
    regions::CrousRegion,
    verifiable::SignedPayload,
};
use reqwest::{Client, Response};

#[derive(Clone)]
pub struct HTCClient {
    pub url: String,
    pub client: Client,
    pub private_key: String,
    pub author: String,
}

#[derive(thiserror::Error, Debug)]
pub enum ClientError {
    #[error("Couldn't put restaurant : {0}")]
    PutRestaurantFailed(String),
    #[error("Couldn't put meals : {0}")]
    PutMealsFailed(String),
    #[error("Couldn't sign payload : {0}")]
    PayloadSigningFailed(String),
    #[error("Couldn't get restaurants : {0}")]
    GetRestaurantsFailed(String),
}

impl HTCClient {
    pub fn new(url: String, private_key: String, author: String) -> Self {
        HTCClient {
            url,
            client: reqwest::Client::new(),
            private_key,
            author,
        }
    }

    pub async fn put_restaurants(
        &self,
        restaurants: Vec<RestaurantSchema>,
        region: CrousRegion,
    ) -> Result<(), ClientError> {
        let client = Client::new();

        let payload = SignedPayload::<Vec<RestaurantSchema>>::sign(
            restaurants,
            &self.private_key,
            &self.author,
        )
        .map_err(|e| ClientError::PayloadSigningFailed(e.to_string()))?;

        let response = client
            .put(format!("{}/{}/restaurants", self.url, region.to_string()))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ClientError::PutRestaurantFailed(e.to_string()))?;

        response
            .error_for_status()
            .map_err(|e| ClientError::PutRestaurantFailed(e.to_string()))?;
        Ok(())
    }

    pub async fn get_restaurants(
        &self,
        region: CrousRegion,
    ) -> Result<Vec<RestaurantSchema>, ClientError> {
        let client = Client::new();
        let restaurants = client
            .get(format!("{}/{}/restaurants", self.url, region.to_string()))
            .send()
            .await
            .map_err(|e| ClientError::GetRestaurantsFailed(e.to_string()))?
            .json::<Vec<RestaurantSchema>>()
            .await
            .map_err(|e| ClientError::GetRestaurantsFailed(e.to_string()))?;
        Ok(restaurants)
    }

    pub async fn put_meals(
        &self,
        meals: Vec<MealSchema>,
        region: CrousRegion,
    ) -> Result<Response, ClientError> {
        let client = Client::new();

        let payload =
            SignedPayload::<Vec<MealSchema>>::sign(meals, &self.private_key, &self.author)
                .map_err(|e| ClientError::PayloadSigningFailed(e.to_string()))?;
        client
            .put(format!("{}/{}/meals", self.url, region))
            .json(&payload)
            .send()
            .await
            .map_err(|e| ClientError::PutRestaurantFailed(e.to_string()))
    }
}
