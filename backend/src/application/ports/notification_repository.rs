use async_trait::async_trait;
use uuid::Uuid;

use crate::application::ports::RepoError;
use crate::domain::entities::{Notification, NotificationStatus};

pub struct NewNotification {
    pub produit_id: Uuid,
    pub consommateur_id: Uuid,
    pub statut: NotificationStatus,
}

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn log(&self, new: NewNotification) -> Result<Notification, RepoError>;
}
