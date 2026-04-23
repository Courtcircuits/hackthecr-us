use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{models::Entity, regions::CrousRegion};


#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Order {
    #[schema(value_type = String)]
    target: CrousRegion,
    #[schema(value_type = String)]
    entity: Entity,
}
