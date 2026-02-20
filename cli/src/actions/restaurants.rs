use scraper::restaurant_list::RestaurantListScraper;

pub struct RestaurantsAction {
    pub target: String,
    pub dry_run: bool,
}

impl RestaurantsAction {
    pub fn new(target: String, dry_run: bool) -> Self {
        Self { target, dry_run }
    }

    // pub async fn execute(&self) {
    //     let list_data = RestaurantListScraper::new()
    // }
}
