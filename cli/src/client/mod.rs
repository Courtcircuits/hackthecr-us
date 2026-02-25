use htc_core::models::restaurants::{Restaurant, RestaurantSchema};
use reqwest::Client;



pub struct HTCClient {
    pub url: String,
    pub client: Client
}



#[derive(thiserror::Error, Debug)]
pub enum RestaurantClientError {
    #[error("Could't put restaurant : {0}")]
    PutRestaurantFailed(String)
}



impl HTCClient {
    pub fn new(url: String) -> Self{
        println!("Saving in {}", url);
        HTCClient{ 
            url,
            client: reqwest::Client::new()
        }
    }


    pub async fn put_restaurants(&self, restaurants: Vec<Restaurant>) -> Result<(), RestaurantClientError> {
        let client = Client::new();

        let restaurants: Vec<RestaurantSchema> = restaurants.iter().map(|r| r.into()).collect();

        let response = client.put(format!("{}/restaurants", self.url))
            .json(&restaurants)
            .send()
            .await
            .map_err(|e| RestaurantClientError::PutRestaurantFailed(e.to_string()))?;

        println!("{:?}", response);

        Ok(())

    }
}
