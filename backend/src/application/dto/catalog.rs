use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::application::dto::MerchantResponseDto;
use crate::application::dto::ProductResponseDto;
use crate::application::ports::ProductWithMerchant;
use crate::domain::entities::ProductStatus;
use crate::domain::services::pricing::discount_percent;

/// One démarque as shown in the feed or its own detail page: the produit
/// plus the marchand fields needed to render it without a second request.
#[derive(Debug, Serialize)]
pub struct OfferDto {
    pub id: Uuid,
    pub marchand_id: Uuid,
    pub marchand_nom: String,
    pub marchand_categorie: String,
    pub marchand_note: Option<Decimal>,
    pub nom: String,
    pub description: String,
    pub prix_initial: Decimal,
    pub prix_demarque: Decimal,
    pub reduction_pct: i32,
    pub quantite: i32,
    pub retrait_debut: DateTime<Utc>,
    pub retrait_fin: DateTime<Utc>,
    pub statut: ProductStatus,
}

impl From<ProductWithMerchant> for OfferDto {
    fn from(p: ProductWithMerchant) -> Self {
        Self {
            id: p.id,
            marchand_id: p.marchand_id,
            marchand_nom: p.marchand_nom,
            marchand_categorie: p.marchand_categorie,
            marchand_note: p.marchand_note,
            reduction_pct: discount_percent(p.prix_initial, p.prix_demarque),
            nom: p.nom,
            description: p.description,
            prix_initial: p.prix_initial,
            prix_demarque: p.prix_demarque,
            quantite: p.quantite,
            retrait_debut: p.retrait_debut,
            retrait_fin: p.retrait_fin,
            statut: p.statut,
        }
    }
}

/// A marchand's public fiche: its info plus its currently active offers.
#[derive(Debug, Serialize)]
pub struct MerchantProfileDto {
    #[serde(flatten)]
    pub marchand: MerchantResponseDto,
    pub offres: Vec<ProductResponseDto>,
}
