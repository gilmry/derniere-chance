use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ReservationStatus {
    Reservee,
    Recuperee,
    Expiree,
}

/// A consommateur's claim on `quantite` units of a produit. `code` is what
/// they present in store; the marchand validates it to mark the reservation
/// `Recuperee`. Plusieurs paniers d'un même commerce réservés ensemble sont
/// autant de lignes partageant le même code.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Reservation {
    pub id: Uuid,
    pub produit_id: Uuid,
    pub consommateur_id: Uuid,
    pub code: String,
    pub quantite: i32,
    pub statut: ReservationStatus,
    pub created_at: DateTime<Utc>,
}
