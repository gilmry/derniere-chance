use async_trait::async_trait;

use crate::application::ports::{NewNotification, NotificationRepository, RepoError};
use crate::domain::entities::Notification;
use crate::infrastructure::database::DbPool;

pub struct PostgresNotificationRepository {
    pool: DbPool,
}

impl PostgresNotificationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NotificationRepository for PostgresNotificationRepository {
    async fn log(&self, new: NewNotification) -> Result<Notification, RepoError> {
        sqlx::query_as::<_, Notification>(
            "INSERT INTO notifications (produit_id, consommateur_id, statut) VALUES ($1, $2, $3)
             RETURNING id, produit_id, consommateur_id, statut, created_at",
        )
        .bind(new.produit_id)
        .bind(new.consommateur_id)
        .bind(new.statut)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }
}
