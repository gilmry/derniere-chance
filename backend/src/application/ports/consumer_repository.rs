use async_trait::async_trait;
use uuid::Uuid;

use crate::application::ports::RepoError;
use crate::domain::entities::Consumer;

pub struct NewConsumer {
    pub email: String,
    pub password_hash: String,
}

#[async_trait]
pub trait ConsumerRepository: Send + Sync {
    async fn create(&self, new: NewConsumer) -> Result<Consumer, RepoError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Consumer>, RepoError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Consumer>, RepoError>;
    /// Backoffice admin uniquement.
    async fn list_all(&self) -> Result<Vec<Consumer>, RepoError>;
    /// Backoffice admin uniquement - cascade sur abonnements/réservations
    /// (ON DELETE CASCADE).
    async fn delete(&self, id: Uuid) -> Result<(), RepoError>;
    /// Retrait du consentement bêta : remplace les données identifiantes par
    /// des valeurs neutres et rend toute connexion impossible, sans
    /// supprimer la ligne - la preuve de consentement et les réservations
    /// déjà honorées par les commerçants y sont rattachées. Idempotent.
    async fn anonymize(&self, id: Uuid) -> Result<(), RepoError>;
    async fn count(&self) -> Result<i64, RepoError>;
}
