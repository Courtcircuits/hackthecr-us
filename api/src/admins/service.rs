use std::{sync::Arc};

use sqlx::PgPool;
use thiserror::Error;

#[derive(Clone)]
pub struct AdminServiceImpl {
    pool: Arc<PgPool>
}

#[derive(Error, Debug)]
pub enum AdminError {
    #[error("{0} is unauthorized")]
    Unauthorized(String)
}

pub trait AdminService {
    // ref à Isalyne -> parce qu'elle fait le pannel admin
    fn is_admin(name: String) -> impl Future<Output = Result<(), AdminError>>;
}
