use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Compte du backoffice admin léger (un seul, bootstrapé au démarrage depuis
/// ADMIN_EMAIL/ADMIN_PASSWORD - pas d'auto-inscription publique).
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Admin {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}
