use htc::{
    models::restaurants::{Restaurant, RestaurantSchema},
    verifiable::SignedPayload,
};
use reqwest::Client;

pub struct HTCClient {
    pub url: String,
    pub client: Client,
    pub private_key: String,
    pub author: String,
}

#[derive(thiserror::Error, Debug)]
pub enum RestaurantClientError {
    #[error("Couldn't put restaurant : {0}")]
    PutRestaurantFailed(String),
    #[error("Couldn't sign payload : {0}")]
    PayloadSigningFailed(String),
    #[error("Couldn't get restaurants : {0}")]
    GetRestaurantsFailed(String),
}

impl HTCClient {
    pub fn new(url: String, private_key: String, author: String) -> Self {
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
            &self.private_key,
            &self.author,
        )
        .map_err(|e| RestaurantClientError::PayloadSigningFailed(e.to_string()))?;

        println!("{:?}", payload);

        let response = client
            .put(format!("{}/restaurants", self.url))
            .json(&payload)
            .send()
            .await
            .map_err(|e| RestaurantClientError::PutRestaurantFailed(e.to_string()))?;

        println!("{:?}", response);

        Ok(())
    }

    pub async fn get_restaurants(
        &self,
    ) -> Result<Vec<RestaurantSchema>, RestaurantClientError> {
        let client = Client::new();
        let restaurants = client
            .get(format!("{}/restaurants", self.url))
            .send()
            .await
            .map_err(|e| RestaurantClientError::GetRestaurantsFailed(e.to_string()))?
            .json::<Vec<RestaurantSchema>>()
            .await
            .map_err(|e| RestaurantClientError::GetRestaurantsFailed(e.to_string()))?;
        Ok(restaurants)
    }
}
