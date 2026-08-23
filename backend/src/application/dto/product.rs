use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::{Product, ProductStatus};
use crate::domain::services::pricing::discount_percent;

#[derive(Debug, Deserialize)]
pub struct CreateProductDto {
    pub nom: String,
    pub description: String,
    pub prix_initial: Decimal,
    pub prix_demarque: Decimal,
    pub quantite: i32,
    pub retrait_debut: DateTime<Utc>,
    pub retrait_fin: DateTime<Utc>,
    /// URL publique renvoyée par POST /marchands/moi/produits/photo -
    /// facultative, un panier reste publiable sans photo.
    pub photo_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProductResponseDto {
    pub id: Uuid,
    pub marchand_id: Uuid,
    pub nom: String,
    pub description: String,
    pub prix_initial: Decimal,
    pub prix_demarque: Decimal,
    pub reduction_pct: i32,
    pub quantite: i32,
    pub retrait_debut: DateTime<Utc>,
    pub retrait_fin: DateTime<Utc>,
    pub statut: ProductStatus,
    pub photo_url: Option<String>,
}

impl From<Product> for ProductResponseDto {
    fn from(product: Product) -> Self {
        Self {
            id: product.id,
            marchand_id: product.marchand_id,
            reduction_pct: discount_percent(product.prix_initial, product.prix_demarque),
            nom: product.nom,
            description: product.description,
            prix_initial: product.prix_initial,
            prix_demarque: product.prix_demarque,
            quantite: product.quantite,
            retrait_debut: product.retrait_debut,
            retrait_fin: product.retrait_fin,
            statut: product.statut,
            photo_url: product.photo_url,
        }
    }
}
