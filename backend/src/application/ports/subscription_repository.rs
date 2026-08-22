use async_trait::async_trait;
use serde::Serialize;
use uuid::Uuid;

use crate::application::ports::RepoError;
use crate::domain::entities::Merchant;

/// A subscriber's contact info, as read for fan-out when a marchand publishes.
#[derive(Debug, Clone, Serialize)]
pub struct SubscriberContact {
    pub consommateur_id: Uuid,
    pub email: String,
}

#[async_trait]
pub trait SubscriptionRepository: Send + Sync {
    async fn follow(&self, consommateur_id: Uuid, marchand_id: Uuid) -> Result<(), RepoError>;
    async fn unfollow(&self, consommateur_id: Uuid, marchand_id: Uuid) -> Result<(), RepoError>;
    async fn is_following(
        &self,
        consommateur_id: Uuid,
        marchand_id: Uuid,
    ) -> Result<bool, RepoError>;
    async fn list_followed_merchants(
        &self,
        consommateur_id: Uuid,
    ) -> Result<Vec<Merchant>, RepoError>;
    async fn list_subscribers(
        &self,
        marchand_id: Uuid,
    ) -> Result<Vec<SubscriberContact>, RepoError>;
}
