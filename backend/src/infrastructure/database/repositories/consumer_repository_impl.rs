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
             RETURNING id, email, password_hash, created_at, anonymise_le",
        )
        .bind(new.email)
        .bind(new.password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<Consumer>, RepoError> {
        sqlx::query_as::<_, Consumer>(
            "SELECT id, email, password_hash, created_at, anonymise_le FROM consommateurs WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Consumer>, RepoError> {
        sqlx::query_as::<_, Consumer>(
            "SELECT id, email, password_hash, created_at, anonymise_le FROM consommateurs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn list_all(&self) -> Result<Vec<Consumer>, RepoError> {
        sqlx::query_as::<_, Consumer>(
            "SELECT id, email, password_hash, created_at, anonymise_le FROM consommateurs ORDER BY created_at DESC",
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

    async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<(), RepoError> {
        // `anonymise_le IS NULL` fait le gardien en base plutôt qu'au-dessus :
        // un compte dont le consentement a été retiré ne doit pas pouvoir se
        // rouvrir, quel que soit le chemin applicatif qui y mène.
        sqlx::query(
            "UPDATE consommateurs
                SET password_hash = $2
              WHERE id = $1 AND anonymise_le IS NULL",
        )
        .bind(id)
        .bind(password_hash)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }

    async fn anonymize(&self, id: Uuid) -> Result<(), RepoError> {
        // L'email est dérivé de l'UUID, qui est déjà la clé primaire : il ne
        // révèle donc rien de plus que ce que la ligne portait déjà, tout en
        // respectant la contrainte UNIQUE. Le hash est vidé pour que
        // `bcrypt::verify` échoue systématiquement, ce qui rend toute
        // reconnexion impossible. COALESCE garde la date du premier retrait
        // si l'opération est rejouée.
        sqlx::query(
            "UPDATE consommateurs
                SET email = 'anonyme-' || id::text || '@invalid',
                    password_hash = '',
                    anonymise_le = COALESCE(anonymise_le, now())
              WHERE id = $1",
        )
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
