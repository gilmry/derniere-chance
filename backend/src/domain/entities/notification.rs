use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "notification_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum NotificationStatus {
    Envoyee,
    Echouee,
}

/// Log entry for one email sent (or attempted) to a subscriber when a
/// followed marchand publishes a new produit.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub produit_id: Uuid,
    pub consommateur_id: Uuid,
    pub statut: NotificationStatus,
    pub created_at: DateTime<Utc>,
}
