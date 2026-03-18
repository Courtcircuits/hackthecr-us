use std::sync::Arc;

use sqlx::PgPool;

pub struct MealsServiceImpl {
    ppol: Arc<PgPool>
}
