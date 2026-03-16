use std::sync::Arc;

use htc::models::admins::{Admin, AdminErrors, AdminModel as _};
use sqlx::PgPool;
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

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
    #[error("Admin already exists : {0}")]
    AlreadyExists(String)
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
    fn get_public_key(&self, name: &str)
    -> impl Future<Output = Result<String, AdminError>> + Send;

    fn create_default_admin_key(
        &self,
        admin_key: &str,
    ) -> impl Future<Output = Result<(), AdminError>> + Send;
}

impl AdminService for AdminServiceImpl {
    async fn get_public_key(&self, name: &str) -> Result<String, AdminError> {
        let admin = self.pool.get_admin(name.to_string()).await?;
        Ok(admin.ssh_key)
    }

    async fn create_default_admin_key(&self, admin_key: &str) -> Result<(), AdminError> {
        let admin = self.pool.get_admin("admin".to_string()).await;
        match admin {
            Ok(admin) => {
                return Err(AdminError::AlreadyExists(admin.ssh_key))
            },
            Err(AdminErrors::NotFound(_,_ )) => {
                self.pool.create_admin(Admin{
                    admin_id: Uuid::new_v4(),
                    ssh_key: admin_key.to_string(),
                    name: "admin".to_string()
                }).await.map_err(|e| AdminError::AlreadyExists(e.to_string()))?;
            },
            Err(_) => {
                info!("Other unhandled error");
            }
        };
        Ok(())
    }
}

impl AdminServiceImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}
