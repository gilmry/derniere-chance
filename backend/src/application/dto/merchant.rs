use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::entities::Merchant;

#[derive(Debug, Deserialize)]
pub struct UpdateMerchantDto {
    pub nom: String,
    pub adresse: String,
    /// Code ISO du pays. Absent : le pays actuel est gardé, pour les clients
    /// (MCP notamment) écrits avant l'ajout du champ.
    #[serde(default)]
    pub pays: Option<String>,
    pub categorie: String,
    /// Case cochée par le commerçant : son commerce existe bien à cette
    /// adresse, même si la recherche ne le trouve pas. Seule façon de faire
    /// accepter un commerce non trouvé (ajout manuel).
    #[serde(default)]
    pub adresse_confirmee: bool,
}

#[derive(Debug, Serialize)]
pub struct MerchantResponseDto {
    pub id: Uuid,
    pub nom: String,
    pub adresse: String,
    pub pays: String,
    pub categorie: String,
    pub note: Option<Decimal>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    /// `false` : commerce non trouvé par la recherche, ajouté manuellement
    /// par le commerçant ; la position est celle de son adresse, ou seulement
    /// de sa ville si la rue est inconnue.
    pub adresse_verifiee: bool,
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
            pays: merchant.pays,
            categorie: merchant.categorie,
            note: merchant.note,
            latitude: merchant.latitude,
            longitude: merchant.longitude,
            adresse_verifiee: merchant.adresse_confirmee_le.is_none(),
            logo_url: merchant.logo_url,
            distance_km: None,
        }
    }
}
