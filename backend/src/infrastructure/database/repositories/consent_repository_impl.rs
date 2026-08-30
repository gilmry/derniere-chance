use async_trait::async_trait;
use uuid::Uuid;

use crate::application::ports::{ConsentRepository, RepoError};
use crate::domain::entities::BetaConsent;
use crate::infrastructure::database::DbPool;

const COLUMNS: &str = "id, consommateur_id, version, accepte_le, retire_le";

pub struct PostgresConsentRepository {
    pool: DbPool,
}

impl PostgresConsentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConsentRepository for PostgresConsentRepository {
    async fn grant(&self, consommateur_id: Uuid, version: &str) -> Result<BetaConsent, RepoError> {
        // ON CONFLICT sur l'index partiel : un second appel ne crée pas de
        // doublon et ne renvoie donc aucune ligne, d'où le repli sur le
        // consentement déjà actif.
        let inserted = sqlx::query_as::<_, BetaConsent>(&format!(
            "INSERT INTO consentements_beta (consommateur_id, version) VALUES ($1, $2)
             ON CONFLICT (consommateur_id) WHERE retire_le IS NULL DO NOTHING
             RETURNING {COLUMNS}"
        ))
        .bind(consommateur_id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?;

        match inserted {
            Some(consent) => Ok(consent),
            None => self
                .find_active(consommateur_id)
                .await?
                .ok_or(RepoError::NotFound),
        }
    }

    async fn find_active(&self, consommateur_id: Uuid) -> Result<Option<BetaConsent>, RepoError> {
        sqlx::query_as::<_, BetaConsent>(&format!(
            "SELECT {COLUMNS} FROM consentements_beta
             WHERE consommateur_id = $1 AND retire_le IS NULL"
        ))
        .bind(consommateur_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn withdraw(&self, consommateur_id: Uuid) -> Result<u64, RepoError> {
        sqlx::query(
            "UPDATE consentements_beta SET retire_le = now()
             WHERE consommateur_id = $1 AND retire_le IS NULL",
        )
        .bind(consommateur_id)
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected())
        .map_err(Into::into)
    }
}
