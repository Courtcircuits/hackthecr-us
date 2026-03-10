use std::path::PathBuf;

use htc::{
    models::restaurants::{Restaurant, RestaurantSchema},
    verifiable::SignedPayload,
};
use reqwest::Client;

pub struct HTCClient {
    pub url: String,
    pub client: Client,
    pub private_key: PathBuf,
    pub author: String,
}

#[derive(thiserror::Error, Debug)]
pub enum RestaurantClientError {
    #[error("Couldn't put restaurant : {0}")]
    PutRestaurantFailed(String),
    #[error("Couldn't sign payload : {0}")]
    PayloadSigningFailed(String),
}

impl HTCClient {
    pub fn new(url: String, private_key: PathBuf, author: String) -> Self {
        println!("Saving in {}", url);
        HTCClient {
            url,
            client: reqwest::Client::new(),
            private_key,
            author,
        }
    }

    pub async fn put_restaurants(
        &self,
        restaurants: Vec<Restaurant>,
    ) -> Result<(), RestaurantClientError> {
        let client = Client::new();

        let restaurants: Vec<RestaurantSchema> = restaurants.iter().map(|r| r.into()).collect();
        let payload = SignedPayload::<Vec<RestaurantSchema>>::sign(
            restaurants,
            self.private_key.clone(),
            &self.author,
        )
        .map_err(|e| RestaurantClientError::PayloadSigningFailed(e.to_string()))?;

        let response = client
            .put(format!("{}/restaurants", self.url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| RestaurantClientError::PutRestaurantFailed(e.to_string()))?;

        println!("{:?}", response);

        Ok(())
    }
}
