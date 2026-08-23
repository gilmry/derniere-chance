use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::application::ports::RepoError;
use crate::domain::entities::{Reservation, ReservationStatus};

pub struct NewReservation {
    pub produit_id: Uuid,
    pub consommateur_id: Uuid,
    pub code: String,
}

/// A reservation flattened with its produit/marchand info - what "Mes
/// réservations" (profil consommateur) reads, one query instead of N+1.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ReservationSummary {
    pub id: Uuid,
    pub code: String,
    pub statut: ReservationStatus,
    pub marchand_nom: String,
    pub produit_nom: String,
    pub prix_demarque: Decimal,
    pub retrait_debut: DateTime<Utc>,
    pub retrait_fin: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Aggregate stats for a marchand's backoffice dashboard ("Aujourd'hui").
#[derive(Debug, Clone, Default)]
pub struct MerchantDailyStats {
    pub paniers_sauves: i64,
    pub chiffre_recupere: Decimal,
}

/// Aggregate stats for a consommateur's profile ("paniers sauvés / économisés").
#[derive(Debug, Clone, Default)]
pub struct ConsumerStats {
    pub paniers_sauves: i64,
    pub montant_economise: Decimal,
}

#[async_trait]
pub trait ReservationRepository: Send + Sync {
    /// Fails with `RepoError::Conflict` if `code` collides with an existing
    /// reservation - the use case regenerates the code and retries.
    async fn create(&self, new: NewReservation) -> Result<Reservation, RepoError>;
    async fn find_by_code(&self, code: &str) -> Result<Option<Reservation>, RepoError>;
    async fn mark_recuperee(&self, id: Uuid) -> Result<Reservation, RepoError>;
    async fn merchant_daily_stats(&self, marchand_id: Uuid) -> Result<MerchantDailyStats, RepoError>;
    async fn consumer_stats(&self, consommateur_id: Uuid) -> Result<ConsumerStats, RepoError>;
    async fn list_by_consumer(
        &self,
        consommateur_id: Uuid,
    ) -> Result<Vec<ReservationSummary>, RepoError>;
    /// Backoffice admin uniquement.
    async fn count(&self) -> Result<i64, RepoError>;
}
