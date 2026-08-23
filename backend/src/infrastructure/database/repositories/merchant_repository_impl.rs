use async_trait::async_trait;
use uuid::Uuid;

use crate::application::ports::{MerchantRepository, NewMerchant, RepoError};
use crate::domain::entities::Merchant;
use crate::infrastructure::database::DbPool;

pub struct PostgresMerchantRepository {
    pool: DbPool,
}

impl PostgresMerchantRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MerchantRepository for PostgresMerchantRepository {
    async fn create(&self, new: NewMerchant) -> Result<Merchant, RepoError> {
        sqlx::query_as::<_, Merchant>(
            "INSERT INTO marchands (nom, adresse, categorie, email, password_hash, latitude, longitude)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             RETURNING id, nom, adresse, categorie, note, email, password_hash, latitude, longitude, created_at",
        )
        .bind(new.nom)
        .bind(new.adresse)
        .bind(new.categorie)
        .bind(new.email)
        .bind(new.password_hash)
        .bind(new.latitude)
        .bind(new.longitude)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<Merchant>, RepoError> {
        sqlx::query_as::<_, Merchant>(
            "SELECT id, nom, adresse, categorie, note, email, password_hash, latitude, longitude, created_at
             FROM marchands WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Merchant>, RepoError> {
        sqlx::query_as::<_, Merchant>(
            "SELECT id, nom, adresse, categorie, note, email, password_hash, latitude, longitude, created_at
             FROM marchands WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn list_all(&self) -> Result<Vec<Merchant>, RepoError> {
        sqlx::query_as::<_, Merchant>(
            "SELECT id, nom, adresse, categorie, note, email, password_hash, latitude, longitude, created_at
             FROM marchands ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn delete(&self, id: Uuid) -> Result<(), RepoError> {
        sqlx::query("DELETE FROM marchands WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(Into::into)
    }

    async fn count(&self) -> Result<i64, RepoError> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM marchands")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }
}
