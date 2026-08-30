use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::ConsentStatusDto;
use crate::application::ports::{ConsentRepository, ConsumerRepository, RepoError};

/// Version du texte de consentement au programme bêta actuellement en
/// vigueur. C'est cette chaîne qui est stockée en base pour dire *à quoi* la
/// personne a consenti, donc elle doit être incrémentée dès que le texte de
/// /confidentialite change sur le fond (nouvelle finalité, nouvelle
/// catégorie de données, nouveau sous-traitant).
///
/// Trois endroits à modifier ensemble, sinon les inscriptions sont refusées
/// tant que le frontend n'est pas redéployé (les deux images sortent du même
/// commit, donc le décalage ne dure que le temps d'un déploiement) :
///   1. cette constante,
///   2. `BETA_CONSENT_VERSION` dans `frontend/src/lib/consent.ts`,
///   3. la date de mise à jour affichée sur `/confidentialite`.
pub const BETA_CONSENT_VERSION: &str = "2026-08-30";

#[derive(Debug, Error)]
pub enum ConsentError {
    #[error("consent version is no longer current")]
    StaleVersion,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

pub struct ConsentUseCases {
    consent_repo: Arc<dyn ConsentRepository>,
    consumer_repo: Arc<dyn ConsumerRepository>,
}

impl ConsentUseCases {
    pub fn new(
        consent_repo: Arc<dyn ConsentRepository>,
        consumer_repo: Arc<dyn ConsumerRepository>,
    ) -> Self {
        Self {
            consent_repo,
            consumer_repo,
        }
    }

    pub async fn status(&self, consommateur_id: Uuid) -> Result<ConsentStatusDto, ConsentError> {
        let active = self.consent_repo.find_active(consommateur_id).await?;
        let consenti = active
            .as_ref()
            .is_some_and(|consent| consent.version == BETA_CONSENT_VERSION);

        Ok(ConsentStatusDto {
            consenti,
            version_acceptee: active.as_ref().map(|consent| consent.version.clone()),
            accepte_le: active.as_ref().map(|consent| consent.accepte_le),
            version_courante: BETA_CONSENT_VERSION.to_string(),
        })
    }

    pub async fn grant(
        &self,
        consommateur_id: Uuid,
        version: &str,
    ) -> Result<ConsentStatusDto, ConsentError> {
        if version != BETA_CONSENT_VERSION {
            return Err(ConsentError::StaleVersion);
        }

        // Une version périmée encore active (texte modifié depuis) doit être
        // retirée avant d'enregistrer la nouvelle : l'index partiel unique
        // n'autorise qu'un consentement actif à la fois, et on veut garder la
        // trace de l'ancien plutôt que de l'écraser.
        if let Some(active) = self.consent_repo.find_active(consommateur_id).await? {
            if active.version != BETA_CONSENT_VERSION {
                self.consent_repo.withdraw(consommateur_id).await?;
            }
        }

        self.consent_repo
            .grant(consommateur_id, BETA_CONSENT_VERSION)
            .await?;
        self.status(consommateur_id).await
    }

    /// Retrait du consentement. La base légale du traitement disparaissant,
    /// le compte est anonymisé dans la foulée : c'est le seul moyen de
    /// cesser le traitement sans effacer la preuve du consentement passé ni
    /// les réservations déjà honorées par les commerçants.
    pub async fn withdraw(&self, consommateur_id: Uuid) -> Result<(), ConsentError> {
        self.consent_repo.withdraw(consommateur_id).await?;
        self.consumer_repo.anonymize(consommateur_id).await?;
        Ok(())
    }

    /// Portier utilisé par l'extracteur `ConsentedConsumer` : vrai seulement
    /// si le consentement porte sur la version en vigueur.
    pub async fn has_current_consent(&self, consommateur_id: Uuid) -> Result<bool, ConsentError> {
        Ok(self
            .consent_repo
            .find_active(consommateur_id)
            .await?
            .is_some_and(|consent| consent.version == BETA_CONSENT_VERSION))
    }
}
