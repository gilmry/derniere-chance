use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::Merchant;

#[derive(Debug, Deserialize)]
pub struct UpdateMerchantDto {
    pub nom: String,
    pub adresse: String,
    pub categorie: String,
}

#[derive(Debug, Serialize)]
pub struct MerchantResponseDto {
    pub id: Uuid,
    pub nom: String,
    pub adresse: String,
    pub categorie: String,
    pub note: Option<Decimal>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub logo_url: Option<String>,
    /// Distance au marchand en km si le consommateur a partagé sa position
    /// (`?lat=&lon=`) et que le marchand a la sienne. Voir OfferDto::distance_km.
    pub distance_km: Option<f64>,
}

impl From<Merchant> for MerchantResponseDto {
    fn from(merchant: Merchant) -> Self {
        Self {
            id: merchant.id,
            nom: merchant.nom,
            adresse: merchant.adresse,
            categorie: merchant.categorie,
            note: merchant.note,
            latitude: merchant.latitude,
            longitude: merchant.longitude,
            logo_url: merchant.logo_url,
            distance_km: None,
        }
    }
}
