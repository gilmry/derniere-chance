use std::sync::Arc;

use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

use crate::application::ports::{
    ConsumerRepository, MerchantRepository, ProductRepository, ProductWithMerchant, RepoError,
    ReservationRepository,
};
use crate::domain::entities::{Consumer, Merchant, ProductStatus};

#[derive(Debug, Error)]
pub enum AdminError {
    #[error("resource not found")]
    NotFound,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

#[derive(Debug, Serialize)]
pub struct AdminStatsDto {
    pub marchands: i64,
    pub consommateurs: i64,
    pub produits_actifs: i64,
    pub reservations: i64,
}

/// Backoffice admin léger : lister/supprimer marchands, consommateurs et
/// produits, statistiques globales. Un seul compte admin (bootstrapé au
/// démarrage), pas de RBAC, pas d'audit log - suffisant pour un opérateur
/// unique qui nettoie des comptes de test ou surveille l'activité.
pub struct AdminUseCases {
    merchant_repo: Arc<dyn MerchantRepository>,
    consumer_repo: Arc<dyn ConsumerRepository>,
    product_repo: Arc<dyn ProductRepository>,
    reservation_repo: Arc<dyn ReservationRepository>,
}

impl AdminUseCases {
    pub fn new(
        merchant_repo: Arc<dyn MerchantRepository>,
        consumer_repo: Arc<dyn ConsumerRepository>,
        product_repo: Arc<dyn ProductRepository>,
        reservation_repo: Arc<dyn ReservationRepository>,
    ) -> Self {
        Self {
            merchant_repo,
            consumer_repo,
            product_repo,
            reservation_repo,
        }
    }

    pub async fn list_merchants(&self) -> Result<Vec<Merchant>, AdminError> {
        Ok(self.merchant_repo.list_all().await?)
    }

    pub async fn delete_merchant(&self, id: Uuid) -> Result<(), AdminError> {
        if self.merchant_repo.find_by_id(id).await?.is_none() {
            return Err(AdminError::NotFound);
        }
        self.merchant_repo.delete(id).await?;
        Ok(())
    }

    pub async fn list_consumers(&self) -> Result<Vec<Consumer>, AdminError> {
        Ok(self.consumer_repo.list_all().await?)
    }

    pub async fn delete_consumer(&self, id: Uuid) -> Result<(), AdminError> {
        if self.consumer_repo.find_by_id(id).await?.is_none() {
            return Err(AdminError::NotFound);
        }
        self.consumer_repo.delete(id).await?;
        Ok(())
    }

    pub async fn list_products(&self) -> Result<Vec<ProductWithMerchant>, AdminError> {
        Ok(self.product_repo.list_all().await?)
    }

    pub async fn delete_product(&self, id: Uuid) -> Result<(), AdminError> {
        if self.product_repo.find_by_id(id).await?.is_none() {
            return Err(AdminError::NotFound);
        }
        self.product_repo.delete(id).await?;
        Ok(())
    }

    /// Dépublier de force (ex. contenu inapproprié) : bascule sur `Ecoule`
    /// plutôt que supprimer, pour garder l'historique des réservations liées.
    pub async fn unpublish_product(&self, id: Uuid) -> Result<(), AdminError> {
        self.product_repo
            .update_status(id, ProductStatus::Ecoule)
            .await
            .map_err(|err| match err {
                RepoError::NotFound => AdminError::NotFound,
                other => AdminError::Internal(other),
            })?;
        Ok(())
    }

    pub async fn stats(&self) -> Result<AdminStatsDto, AdminError> {
        Ok(AdminStatsDto {
            marchands: self.merchant_repo.count().await?,
            consommateurs: self.consumer_repo.count().await?,
            produits_actifs: self.product_repo.count_active().await?,
            reservations: self.reservation_repo.count().await?,
        })
    }
}
