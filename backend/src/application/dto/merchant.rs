use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::Merchant;

#[derive(Debug, Serialize)]
pub struct MerchantResponseDto {
    pub id: Uuid,
    pub nom: String,
    pub adresse: String,
    pub categorie: String,
    pub note: Option<Decimal>,
}

impl From<Merchant> for MerchantResponseDto {
    fn from(merchant: Merchant) -> Self {
        Self {
            id: merchant.id,
            nom: merchant.nom,
            adresse: merchant.adresse,
            categorie: merchant.categorie,
            note: merchant.note,
        }
    }
}
