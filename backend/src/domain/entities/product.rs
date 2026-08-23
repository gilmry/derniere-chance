use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "product_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ProductStatus {
    Publie,
    Ecoule,
    Expire,
}

/// A "panier" (basket) a marchand puts up as a démarque: a discounted batch
/// of soon-to-be-wasted stock, available in limited quantity within a pickup
/// window.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Product {
    pub id: Uuid,
    pub marchand_id: Uuid,
    pub nom: String,
    pub description: String,
    pub prix_initial: Decimal,
    pub prix_demarque: Decimal,
    pub quantite: i32,
    pub retrait_debut: DateTime<Utc>,
    pub retrait_fin: DateTime<Utc>,
    pub statut: ProductStatus,
    pub photo_url: Option<String>,
    pub created_at: DateTime<Utc>,
}
