use async_trait::async_trait;

use crate::application::ports::RepoError;
use crate::domain::entities::Admin;

/// Pas de `create` ici volontairement : le seul compte admin est créé par
/// `infrastructure::bootstrap::bootstrap_admin` (SQL direct), il n'y a pas
/// d'inscription publique - backoffice léger, un seul propriétaire.
#[async_trait]
pub trait AdminRepository: Send + Sync {
    async fn find_by_email(&self, email: &str) -> Result<Option<Admin>, RepoError>;
}
