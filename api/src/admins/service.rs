use std::sync::Arc;

use htc::models::admins::{AdminErrors, AdminModel as _};
use sqlx::PgPool;
use thiserror::Error;

#[derive(Clone)]
pub struct AdminServiceImpl {
    pool: Arc<PgPool>,
}

#[derive(Error, Debug)]
pub enum AdminError {
    #[error("{0} is unauthorized")]
    Unauthorized(String),
    #[error("Admin not found: {0}")]
    NotFound(String),
}

impl From<AdminErrors> for AdminError {
    fn from(e: AdminErrors) -> Self {
        match e {
            AdminErrors::NotFound(name, _) => AdminError::NotFound(name),
            e => AdminError::Unauthorized(e.to_string()),
        }
    }
}

pub trait AdminService {
    fn get_public_key(&self, name: &str) -> impl Future<Output = Result<String, AdminError>> + Send;
}

impl AdminService for AdminServiceImpl {
    async fn get_public_key(&self, name: &str) -> Result<String, AdminError> {
        let admin = self.pool.get_admin(name.to_string()).await?;
        Ok(admin.ssh_key)
    }
}

impl AdminServiceImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}
