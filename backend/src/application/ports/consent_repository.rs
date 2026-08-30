use async_trait::async_trait;
use uuid::Uuid;

use crate::application::ports::RepoError;
use crate::domain::entities::BetaConsent;

#[async_trait]
pub trait ConsentRepository: Send + Sync {
    /// Enregistre un consentement au programme bêta. Idempotent : si le
    /// consommateur a déjà un consentement actif, celui-ci est renvoyé tel
    /// quel plutôt que dupliqué (la date d'origine fait foi).
    async fn grant(&self, consommateur_id: Uuid, version: &str) -> Result<BetaConsent, RepoError>;

    /// Consentement actif (non retiré) du consommateur, s'il y en a un.
    async fn find_active(&self, consommateur_id: Uuid) -> Result<Option<BetaConsent>, RepoError>;

    /// Horodate le retrait de tous les consentements actifs du consommateur.
    /// Renvoie le nombre de lignes touchées - 0 signifie qu'il n'y avait
    /// rien à retirer, ce qui rend un double retrait inoffensif.
    async fn withdraw(&self, consommateur_id: Uuid) -> Result<u64, RepoError>;
}
