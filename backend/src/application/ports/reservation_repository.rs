use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::application::ports::RepoError;
use crate::domain::entities::{Reservation, ReservationStatus};

/// Plusieurs paniers d'un même commerce réservés ensemble, sous un code.
pub struct NewReservationGroup {
    pub consommateur_id: Uuid,
    pub code: String,
    /// (produit, quantité), sans doublon de produit.
    pub lignes: Vec<(Uuid, i32)>,
}

/// A reservation flattened with its produit/marchand info - what "Mes
/// réservations" (profil consommateur) reads, one query instead of N+1.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ReservationSummary {
    pub id: Uuid,
    pub code: String,
    pub quantite: i32,
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
    /// Tout ou rien : retire les quantités du stock et crée une ligne par
    /// panier, dans une seule transaction. `RepoError::Conflict` si le code
    /// est déjà pris (le cas d'usage en tire un autre), `RepoError::NotFound`
    /// si un panier n'est plus disponible en quantité suffisante ; rien
    /// n'est alors réservé.
    async fn create_group(&self, new: NewReservationGroup) -> Result<Vec<Reservation>, RepoError>;
    /// Paniers (quantités comprises) qu'un client a réservés chez ce
    /// commerce et pas encore retirés, fenêtre de retrait non échue.
    async fn paniers_en_attente(
        &self,
        consommateur_id: Uuid,
        marchand_id: Uuid,
    ) -> Result<i64, RepoError>;
    /// Toutes les lignes réservées sous ce code (vide si inconnu).
    async fn find_by_code(&self, code: &str) -> Result<Vec<Reservation>, RepoError>;
    async fn mark_recuperee_by_code(&self, code: &str) -> Result<u64, RepoError>;
    async fn merchant_daily_stats(
        &self,
        marchand_id: Uuid,
    ) -> Result<MerchantDailyStats, RepoError>;
    async fn consumer_stats(&self, consommateur_id: Uuid) -> Result<ConsumerStats, RepoError>;
    async fn list_by_consumer(
        &self,
        consommateur_id: Uuid,
    ) -> Result<Vec<ReservationSummary>, RepoError>;
    /// Backoffice admin uniquement.
    async fn count(&self) -> Result<i64, RepoError>;
}
