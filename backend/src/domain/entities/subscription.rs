use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// A consommateur following a marchand ("marchand ami") to be notified by
/// email whenever that marchand publishes a new démarque.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub consommateur_id: Uuid,
    pub marchand_id: Uuid,
    pub created_at: DateTime<Utc>,
}
