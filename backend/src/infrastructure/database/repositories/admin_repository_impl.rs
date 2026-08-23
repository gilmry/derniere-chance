use async_trait::async_trait;

use crate::application::ports::{AdminRepository, RepoError};
use crate::domain::entities::Admin;
use crate::infrastructure::database::DbPool;

pub struct PostgresAdminRepository {
    pool: DbPool,
}

impl PostgresAdminRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AdminRepository for PostgresAdminRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<Admin>, RepoError> {
        sqlx::query_as::<_, Admin>(
            "SELECT id, email, password_hash, created_at FROM admins WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }
}
