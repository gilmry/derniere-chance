use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::{MerchantProfileDto, OfferDto};
use crate::application::ports::{MerchantRepository, ProductRepository, RepoError};

#[derive(Debug, Error)]
pub enum CatalogError {
    #[error("resource not found")]
    NotFound,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

/// Read-only browsing use cases, open to anyone (no auth) - the feed, an
/// offer's detail page, and a marchand's public fiche.
pub struct CatalogUseCases {
    product_repo: Arc<dyn ProductRepository>,
    merchant_repo: Arc<dyn MerchantRepository>,
}

impl CatalogUseCases {
    pub fn new(
        product_repo: Arc<dyn ProductRepository>,
        merchant_repo: Arc<dyn MerchantRepository>,
    ) -> Self {
        Self {
            product_repo,
            merchant_repo,
        }
    }

    pub async fn list_active_offers(
        &self,
        categorie: Option<String>,
    ) -> Result<Vec<OfferDto>, CatalogError> {
        let offers = self.product_repo.list_active(categorie.as_deref()).await?;
        Ok(offers.into_iter().map(Into::into).collect())
    }

    pub async fn get_offer(&self, id: Uuid) -> Result<OfferDto, CatalogError> {
        self.product_repo
            .find_with_merchant(id)
            .await?
            .map(Into::into)
            .ok_or(CatalogError::NotFound)
    }

    pub async fn get_merchant_profile(&self, id: Uuid) -> Result<MerchantProfileDto, CatalogError> {
        let merchant = self
            .merchant_repo
            .find_by_id(id)
            .await?
            .ok_or(CatalogError::NotFound)?;
        let offres = self.product_repo.list_active_by_merchant(id).await?;

        Ok(MerchantProfileDto {
            marchand: merchant.into(),
            offres: offres.into_iter().map(Into::into).collect(),
        })
    }
}
