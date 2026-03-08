use std::future::Future;

use sqlx::PgPool;
use sqlx::types::Uuid;

pub struct School {
    pub school_id: Uuid,
    pub long_name: String,
    pub name: String,
    pub coordinates: Option<String>,
}

pub enum SchoolModelError {
    NotFound,
    DatabaseError(String),
}

pub trait SchoolModel {
    fn create_school(&self, school: School) -> impl Future<Output = Result<(), SchoolModelError>> + Send;
    fn get_school_by_name(&self, name: String) -> impl Future<Output = Result<School, SchoolModelError>> + Send;
    fn get_all_schools(&self) -> impl Future<Output = Result<Vec<School>, SchoolModelError>> + Send;
}

impl SchoolModel for PgPool {
    async fn create_school(&self, school: School) -> Result<(), SchoolModelError> {
        sqlx::query!(
            "INSERT INTO schools (school_id, long_name, name, coordinates) VALUES ($1, $2, $3, $4)",
            school.school_id,
            school.long_name,
            school.name,
            school.coordinates
        )
        .execute(self)
        .await
        .map_err(|e| SchoolModelError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_school_by_name(&self, name: String) -> Result<School, SchoolModelError> {
        let row = sqlx::query!(
            "SELECT school_id, long_name, name, coordinates FROM schools WHERE name = $1",
            name
        )
        .fetch_optional(self)
        .await
        .map_err(|e| SchoolModelError::DatabaseError(e.to_string()))?
        .ok_or(SchoolModelError::NotFound)?;

        Ok(School {
            school_id: row.school_id,
            long_name: row.long_name,
            name: row.name,
            coordinates: row.coordinates,
        })
    }

    async fn get_all_schools(&self) -> Result<Vec<School>, SchoolModelError> {
        let rows = sqlx::query!(
            "SELECT school_id, long_name, name, coordinates FROM schools"
        )
        .fetch_all(self)
        .await
        .map_err(|e| SchoolModelError::DatabaseError(e.to_string()))?;

        let schools = rows
            .into_iter()
            .map(|row| School {
                school_id: row.school_id,
                long_name: row.long_name,
                name: row.name,
                coordinates: row.coordinates,
            })
            .collect();

        Ok(schools)
    }
}
