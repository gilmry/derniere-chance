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

#[async_trait]
pub trait MerchantRepository: Send + Sync {
    async fn create(&self, new: NewMerchant) -> Result<Merchant, RepoError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Merchant>, RepoError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Merchant>, RepoError>;
}
