use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::application::ports::{NewPasswordReset, PasswordResetRepository, RepoError};
use crate::domain::entities::AccountSubject;
use crate::infrastructure::database::DbPool;

pub struct PostgresPasswordResetRepository {
    pool: DbPool,
}

impl PostgresPasswordResetRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

/// Les deux colonnes sont exclusives l'une de l'autre (contrainte
/// `reinitialisation_un_seul_sujet`), donc ce découpage est total.
fn columns(subject: AccountSubject) -> (Option<Uuid>, Option<Uuid>) {
    match subject {
        AccountSubject::Consumer(id) => (Some(id), None),
        AccountSubject::Merchant(id) => (None, Some(id)),
    }
}

fn subject_from_row(
    consommateur_id: Option<Uuid>,
    marchand_id: Option<Uuid>,
) -> Option<AccountSubject> {
    match (consommateur_id, marchand_id) {
        (Some(id), None) => Some(AccountSubject::Consumer(id)),
        (None, Some(id)) => Some(AccountSubject::Merchant(id)),
        // Impossible tant que la contrainte tient ; on préfère rendre `None`
        // qu'ouvrir un compte sur une ligne qu'on ne sait pas interpréter.
        _ => None,
    }
}

#[async_trait]
impl PasswordResetRepository for PostgresPasswordResetRepository {
    async fn create(&self, new: NewPasswordReset) -> Result<(), RepoError> {
        let (consommateur_id, marchand_id) = columns(new.subject);

        // Ménage opportuniste : les jetons périmés ou consommés de ce compte
        // n'ont plus d'usage, et les garder ferait grossir la table pour rien.
        // Fait ici plutôt que par une tâche planifiée, qu'il faudrait
        // surveiller.
        sqlx::query(
            "DELETE FROM reinitialisations_mot_de_passe
              WHERE (consommateur_id IS NOT DISTINCT FROM $1)
                AND (marchand_id IS NOT DISTINCT FROM $2)
                AND (expire_le <= now() OR utilise_le IS NOT NULL)",
        )
        .bind(consommateur_id)
        .bind(marchand_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO reinitialisations_mot_de_passe
                 (token_hash, consommateur_id, marchand_id, expire_le)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(&new.token_hash)
        .bind(consommateur_id)
        .bind(marchand_id)
        .bind(new.expires_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }

    async fn consume(&self, token_hash: &str) -> Result<Option<AccountSubject>, RepoError> {
        // Marquage et lecture dans la même requête : deux ouvertures
        // simultanées du même lien ne doivent pas aboutir toutes les deux.
        let row: Option<(Option<Uuid>, Option<Uuid>)> = sqlx::query_as(
            "UPDATE reinitialisations_mot_de_passe
                SET utilise_le = now()
              WHERE token_hash = $1
                AND utilise_le IS NULL
                AND expire_le > now()
          RETURNING consommateur_id, marchand_id",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|(consommateur_id, marchand_id)| {
            subject_from_row(consommateur_id, marchand_id)
        }))
    }

    async fn invalidate_all(&self, subject: AccountSubject) -> Result<u64, RepoError> {
        let (consommateur_id, marchand_id) = columns(subject);

        let result = sqlx::query(
            "UPDATE reinitialisations_mot_de_passe
                SET utilise_le = now()
              WHERE (consommateur_id IS NOT DISTINCT FROM $1)
                AND (marchand_id IS NOT DISTINCT FROM $2)
                AND utilise_le IS NULL",
        )
        .bind(consommateur_id)
        .bind(marchand_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn last_request_at(
        &self,
        subject: AccountSubject,
    ) -> Result<Option<DateTime<Utc>>, RepoError> {
        let (consommateur_id, marchand_id) = columns(subject);

        let row: Option<(DateTime<Utc>,)> = sqlx::query_as(
            "SELECT cree_le
               FROM reinitialisations_mot_de_passe
              WHERE (consommateur_id IS NOT DISTINCT FROM $1)
                AND (marchand_id IS NOT DISTINCT FROM $2)
                AND utilise_le IS NULL
                AND expire_le > now()
           ORDER BY cree_le DESC
              LIMIT 1",
        )
        .bind(consommateur_id)
        .bind(marchand_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(cree_le,)| cree_le))
    }
}
