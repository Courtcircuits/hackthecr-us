use reqwest::Client;

#[derive(thiserror::Error, Debug)]
pub enum RestaurantClient {
    #[error("Could't put restaurant'")]
    PutRestaurantFailed
}

// async fn put_restaurants(restaurants: Vec<Restaurants>) -> Result<(), RestaurantClient> {
//     let client = Client::new();
//     client.
//
// }
