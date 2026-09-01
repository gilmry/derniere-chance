use async_trait::async_trait;
use uuid::Uuid;

use crate::application::ports::RepoError;
use crate::domain::entities::Merchant;

pub struct NewMerchant {
    pub nom: String,
    pub adresse: String,
    pub categorie: String,
    pub email: String,
    pub password_hash: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

pub struct MerchantUpdate {
    pub nom: String,
    pub adresse: String,
    pub categorie: String,
}

#[async_trait]
pub trait MerchantRepository: Send + Sync {
    async fn create(&self, new: NewMerchant) -> Result<Merchant, RepoError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Merchant>, RepoError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Merchant>, RepoError>;
    /// Backoffice admin uniquement.
    async fn list_all(&self) -> Result<Vec<Merchant>, RepoError>;
    /// Backoffice admin uniquement - cascade sur produits/abonnements
    /// (ON DELETE CASCADE), donc sur réservations/notifications par ricochet.
    async fn delete(&self, id: Uuid) -> Result<(), RepoError>;
    /// Retrait du consentement bêta : efface le nom, l'adresse, la position
    /// et le logo - tout ce qui était publié sur la carte - et rend la
    /// connexion impossible, sans supprimer la ligne. Les paniers déjà
    /// retirés par des clients y restent rattachés. Idempotent.
    async fn anonymize(&self, id: Uuid) -> Result<(), RepoError>;
    /// Remplace l'empreinte du mot de passe. Ne touche pas à un compte
    /// anonymisé : son empreinte vide doit le rester, sans quoi un retrait de
    /// consentement se rouvrirait par une réinitialisation.
    async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<(), RepoError>;
    async fn count(&self) -> Result<i64, RepoError>;
    async fn update_logo(&self, id: Uuid, logo_url: &str) -> Result<Merchant, RepoError>;
    async fn update(&self, id: Uuid, changes: MerchantUpdate) -> Result<Merchant, RepoError>;
}
