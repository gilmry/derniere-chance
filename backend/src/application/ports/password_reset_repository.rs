use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::application::ports::RepoError;
use crate::domain::entities::AccountSubject;

pub struct NewPasswordReset {
    pub subject: AccountSubject,
    /// Empreinte SHA-256 du jeton. Le jeton en clair ne quitte jamais le
    /// use case : il part dans l'email et rien d'autre ne le conserve.
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
}

#[async_trait]
pub trait PasswordResetRepository: Send + Sync {
    async fn create(&self, new: NewPasswordReset) -> Result<(), RepoError>;

    /// Consomme le jeton et renvoie le compte visé, ou `None` s'il est
    /// inconnu, expiré ou déjà utilisé.
    ///
    /// Marquage et lecture dans la même requête : deux appels concurrents sur
    /// un même lien ne doivent pas réussir tous les deux.
    async fn consume(&self, token_hash: &str) -> Result<Option<AccountSubject>, RepoError>;

    /// Invalide les jetons encore valides d'un compte. Appelé après un
    /// changement réussi : les liens envoyés avant ne doivent plus ouvrir le
    /// compte, y compris celui d'une demande faite par un tiers.
    async fn invalidate_all(&self, subject: AccountSubject) -> Result<u64, RepoError>;

    /// Date de la dernière demande encore valide pour ce compte, s'il y en a
    /// une. Sert à ne pas réexpédier un lien à chaque clic sur le formulaire.
    async fn last_request_at(
        &self,
        subject: AccountSubject,
    ) -> Result<Option<DateTime<Utc>>, RepoError>;
}
