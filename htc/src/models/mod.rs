use std::str::FromStr;

pub mod admins;
pub mod keywords;
pub mod meals;
pub mod restaurants;
pub mod schools;
pub mod scrape_batch;

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
#[serde(try_from = "String")]
pub enum Entity {
    Restaurants,
    Meals(String),
    Schools,
}

#[derive(thiserror::Error, Debug)]
pub enum EntityError {
    #[error("I dont know as an entity error : {0}")]
    Unknown(String),
}

impl FromStr for Entity {
    type Err = EntityError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "schools" => Ok(Entity::Schools),
            "restaurants" => Ok(Entity::Restaurants),
            &_ => {
                if s.starts_with("meals") {
                    Ok(Entity::Meals(s.replace("meals-", "")))
                } else {
                    Err(EntityError::Unknown("Unknown entity".to_string()))
                }
            }
        }
    }
}

impl TryFrom<String> for Entity {
    type Error = EntityError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Entity::from_str(&s)
    }
}

impl ToString for Entity {
    fn to_string(&self) -> String {
        match self {
            Entity::Restaurants => "restaurants".to_string(),
            Entity::Meals(restaurant_id) => format!("meals-{}", restaurant_id),
            Entity::Schools => "schools".to_string(),
        }
    }
}
