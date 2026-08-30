use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Consumer {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    /// Renseigné quand le consommateur a retiré son consentement bêta :
    /// `email` ne vaut alors plus qu'un identifiant technique.
    pub anonymise_le: Option<DateTime<Utc>>,
}
