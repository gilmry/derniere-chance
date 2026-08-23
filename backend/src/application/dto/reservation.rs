use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::application::ports::ProductWithMerchant;
use crate::domain::entities::{Reservation, ReservationStatus};

/// What the consommateur sees on the "Panier réservé !" screen: the code to
/// present in store, plus enough merchant/produit context to render it
/// without a follow-up request.
#[derive(Debug, Serialize)]
pub struct ReservationConfirmationDto {
    pub id: Uuid,
    pub code: String,
    pub statut: ReservationStatus,
    pub marchand_nom: String,
    pub produit_nom: String,
    pub prix_demarque: Decimal,
    pub retrait_debut: DateTime<Utc>,
    pub retrait_fin: DateTime<Utc>,
}

/// What the marchand sees after avoir validé un code en boutique.
#[derive(Debug, Serialize)]
pub struct PickupValidationDto {
    pub code: String,
    pub produit_nom: String,
}

impl ReservationConfirmationDto {
    pub fn new(reservation: Reservation, offer: ProductWithMerchant) -> Self {
        Self {
            id: reservation.id,
            code: reservation.code,
            statut: reservation.statut,
            marchand_nom: offer.marchand_nom,
            produit_nom: offer.nom,
            prix_demarque: offer.prix_demarque,
            retrait_debut: offer.retrait_debut,
            retrait_fin: offer.retrait_fin,
        }
    }
}
