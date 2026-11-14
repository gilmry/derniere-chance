use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Merchant {
    pub id: Uuid,
    pub nom: String,
    pub adresse: String,
    /// Code ISO 3166-1 alpha-2 (« FR », « BE »...), voir `domain::services::pays`.
    pub pays: String,
    pub categorie: String,
    pub note: Option<Decimal>,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    /// Renseigné quand le géocodeur ne connaît pas l'adresse et que le
    /// commerçant a coché la case confirmant que son commerce y existe bien :
    /// date de cet engagement. `None` : adresse trouvée par le géocodeur.
    pub adresse_confirmee_le: Option<DateTime<Utc>>,
    pub logo_url: Option<String>,
    pub created_at: DateTime<Utc>,
    /// Renseigné quand le marchand a retiré son consentement bêta : nom,
    /// adresse et position ne valent alors plus que des espaces réservés.
    pub anonymise_le: Option<DateTime<Utc>>,
}
