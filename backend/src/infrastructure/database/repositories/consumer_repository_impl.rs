use async_trait::async_trait;
use uuid::Uuid;

use crate::application::ports::{ConsumerRepository, NewConsumer, RepoError};
use crate::domain::entities::Consumer;
use crate::infrastructure::database::DbPool;

pub struct PostgresConsumerRepository {
    pool: DbPool,
}

impl PostgresConsumerRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConsumerRepository for PostgresConsumerRepository {
    async fn create(&self, new: NewConsumer) -> Result<Consumer, RepoError> {
        sqlx::query_as::<_, Consumer>(
            "INSERT INTO consommateurs (email, password_hash) VALUES ($1, $2)
             RETURNING id, email, password_hash, created_at",
        )
        .bind(new.email)
        .bind(new.password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<Consumer>, RepoError> {
        sqlx::query_as::<_, Consumer>(
            "SELECT id, email, password_hash, created_at FROM consommateurs WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Consumer>, RepoError> {
        sqlx::query_as::<_, Consumer>(
            "SELECT id, email, password_hash, created_at FROM consommateurs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn list_all(&self) -> Result<Vec<Consumer>, RepoError> {
        sqlx::query_as::<_, Consumer>(
            "SELECT id, email, password_hash, created_at FROM consommateurs ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepoError> {
        sqlx::query("DELETE FROM consommateurs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    async fn count(&self) -> Result<i64, RepoError> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM consommateurs")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }
}
