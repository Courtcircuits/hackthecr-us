use sqlx::{PgPool, types::Uuid};
use thiserror::Error;

pub struct Admin {
    admin_id: Uuid,
    name: String,
    ssh_key: String,
}

#[derive(Debug, Error)]
pub enum AdminErrors {
    #[error("Could't create admin : {0}")]
    AdminCreationError(String),
    #[error("Unknown admin model error : {0}")]
    UnknownError(String),
    #[error("Admin {0} not found: {1}")]
    NotFound(String, String)
}

pub trait AdminModel {
    fn create_admin(&self, admin: Admin) -> impl Future<Output = Result<(), AdminErrors>> + Send;
    fn get_admin(&self, name: String) -> impl Future<Output = Result<Admin, AdminErrors>> + Send;
}

impl AdminModel for PgPool {
    async fn create_admin(&self, admin: Admin) -> Result<(), AdminErrors> {
        sqlx::query!(
            "INSERT INTO admins(admin_id, ssh_key, name) VALUES ($1, $2, $3)",
            admin.admin_id,
            admin.ssh_key,
            admin.name
        )
        .execute(self)
        .await
        .map_err(|e| AdminErrors::AdminCreationError(e.to_string()))?;

        Ok(())
    }

    async fn get_admin(&self, name: String) -> Result<Admin, AdminErrors> {
        let row = sqlx::query!("SELECT admin_id, ssh_key, name FROM admins WHERE name = $1", name)
            .fetch_one(self)
            .await
            .map_err(|e| AdminErrors::NotFound(name, e.to_string()))?;

        let admin: Admin = Admin {
            ssh_key: row.ssh_key,
            admin_id: row.admin_id,
            name: row.name
        };

        Ok(admin)
    }
}
