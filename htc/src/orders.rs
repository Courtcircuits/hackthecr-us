use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{models::Entity, regions::CrousRegion};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Order {
    #[schema(value_type = String)]
    #[serde(default = "Uuid::new_v4")]
    pub job_id: Uuid,
    #[schema(value_type = String)]
    pub target: CrousRegion,
    pub entity: Entity,
    pub date: String,
}

impl Order {
    pub fn new(target: CrousRegion, entity: Entity) -> Self {
        let date = chrono::Utc::now().to_rfc3339();
        Self {
            job_id: Uuid::new_v4(),
            target,
            entity,
            date,
        }
    }
}
