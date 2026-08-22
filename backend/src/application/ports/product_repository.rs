use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Serialize;
use uuid::Uuid;

use crate::application::ports::RepoError;
use crate::domain::entities::{Product, ProductStatus};

pub struct NewProduct {
    pub marchand_id: Uuid,
    pub nom: String,
    pub description: String,
    pub prix_initial: Decimal,
    pub prix_demarque: Decimal,
    pub quantite: i32,
    pub retrait_debut: DateTime<Utc>,
    pub retrait_fin: DateTime<Utc>,
}

/// A produit flattened with its marchand's public info - the shape the
/// catalogue (feed, offer detail, merchant page) reads, so a single query can
/// serve it instead of an N+1 fetch per offer.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ProductWithMerchant {
    pub id: Uuid,
    pub marchand_id: Uuid,
    pub marchand_nom: String,
    pub marchand_categorie: String,
    pub marchand_note: Option<Decimal>,
    pub marchand_latitude: Option<f64>,
    pub marchand_longitude: Option<f64>,
    pub nom: String,
    pub description: String,
    pub prix_initial: Decimal,
    pub prix_demarque: Decimal,
    pub quantite: i32,
    pub retrait_debut: DateTime<Utc>,
    pub retrait_fin: DateTime<Utc>,
    pub statut: ProductStatus,
}

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn create(&self, new: NewProduct) -> Result<Product, RepoError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Product>, RepoError>;
    async fn find_with_merchant(&self, id: Uuid) -> Result<Option<ProductWithMerchant>, RepoError>;
    async fn list_active(
        &self,
        categorie: Option<&str>,
    ) -> Result<Vec<ProductWithMerchant>, RepoError>;
    async fn list_active_by_merchant(
        &self,
        marchand_id: Uuid,
    ) -> Result<Vec<Product>, RepoError>;
    async fn list_by_merchant(&self, marchand_id: Uuid) -> Result<Vec<Product>, RepoError>;
    async fn update_status(
        &self,
        id: Uuid,
        statut: ProductStatus,
    ) -> Result<Product, RepoError>;
    /// Atomically claims one unit for a reservation: decrements `quantite`
    /// and flips to `Ecoule` once it hits zero. Fails with `RepoError::NotFound`
    /// if the produit isn't `Publie` or has no stock left, so two concurrent
    /// reservations can never both succeed on the last unit.
    async fn reserve_unit(&self, id: Uuid) -> Result<Product, RepoError>;
}
