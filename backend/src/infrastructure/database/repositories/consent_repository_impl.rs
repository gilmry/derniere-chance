use async_trait::async_trait;

use crate::application::ports::{ConsentRepository, RepoError};
use crate::domain::entities::{BetaConsent, ConsentSubject};
use crate::infrastructure::database::DbPool;

const COLUMNS: &str = "id, consommateur_id, marchand_id, version, accepte_le, retire_le";

pub struct PostgresConsentRepository {
    pool: DbPool,
}

impl PostgresConsentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

/// Colonne porteuse du sujet, et son index partiel unique associé. Les deux
/// vont de pair : `ON CONFLICT` doit désigner l'index qui correspond à la
/// colonne renseignée, sinon Postgres ne sait pas quel conflit inférer.
fn subject_column(subject: ConsentSubject) -> &'static str {
    match subject {
        ConsentSubject::Consumer(_) => "consommateur_id",
        ConsentSubject::Merchant(_) => "marchand_id",
    }
}

#[async_trait]
impl ConsentRepository for PostgresConsentRepository {
    async fn grant(
        &self,
        subject: ConsentSubject,
        version: &str,
    ) -> Result<BetaConsent, RepoError> {
        let column = subject_column(subject);

        // ON CONFLICT sur l'index partiel : un second appel ne crée pas de
        // doublon et ne renvoie donc aucune ligne, d'où le repli sur le
        // consentement déjà actif.
        let inserted = sqlx::query_as::<_, BetaConsent>(&format!(
            "INSERT INTO consentements_beta ({column}, version) VALUES ($1, $2)
             ON CONFLICT ({column}) WHERE retire_le IS NULL DO NOTHING
             RETURNING {COLUMNS}"
        ))
        .bind(subject.id())
        .bind(version)
        .fetch_optional(&self.pool)
        .await?;

        match inserted {
            Some(consent) => Ok(consent),
            None => self
                .find_active(subject)
                .await?
                .ok_or(RepoError::NotFound),
        }
    }

    async fn find_active(
        &self,
        subject: ConsentSubject,
    ) -> Result<Option<BetaConsent>, RepoError> {
        let column = subject_column(subject);

        sqlx::query_as::<_, BetaConsent>(&format!(
            "SELECT {COLUMNS} FROM consentements_beta
             WHERE {column} = $1 AND retire_le IS NULL"
        ))
        .bind(subject.id())
        .fetch_optional(&self.pool)
        .await
        .map_err(Into::into)
    }

    async fn withdraw(&self, subject: ConsentSubject) -> Result<u64, RepoError> {
        let column = subject_column(subject);

        sqlx::query(&format!(
            "UPDATE consentements_beta SET retire_le = now()
             WHERE {column} = $1 AND retire_le IS NULL"
        ))
        .bind(subject.id())
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected())
        .map_err(Into::into)
    }
}
