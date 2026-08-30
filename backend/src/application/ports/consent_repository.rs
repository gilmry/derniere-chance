use async_trait::async_trait;

use crate::application::ports::RepoError;
use crate::domain::entities::{BetaConsent, ConsentSubject};

#[async_trait]
pub trait ConsentRepository: Send + Sync {
    /// Enregistre un consentement au programme bêta. Idempotent : si le sujet
    /// a déjà un consentement actif, celui-ci est renvoyé tel quel plutôt que
    /// dupliqué (la date d'origine fait foi).
    async fn grant(&self, subject: ConsentSubject, version: &str)
        -> Result<BetaConsent, RepoError>;

    /// Consentement actif (non retiré) du sujet, s'il y en a un.
    async fn find_active(&self, subject: ConsentSubject)
        -> Result<Option<BetaConsent>, RepoError>;

    /// Horodate le retrait de tous les consentements actifs du sujet. Renvoie
    /// le nombre de lignes touchées - 0 signifie qu'il n'y avait rien à
    /// retirer, ce qui rend un double retrait inoffensif.
    async fn withdraw(&self, subject: ConsentSubject) -> Result<u64, RepoError>;
}
