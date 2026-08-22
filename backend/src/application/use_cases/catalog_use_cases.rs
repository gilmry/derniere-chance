use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::{MerchantProfileDto, OfferDto};
use crate::application::ports::{MerchantRepository, ProductRepository, RepoError};
use crate::domain::services::geoloc::distance_km;

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
        consumer_coords: Option<(f64, f64)>,
    ) -> Result<Vec<OfferDto>, CatalogError> {
        let offers = self.product_repo.list_active(categorie.as_deref()).await?;
        let mut offers: Vec<OfferDto> = offers
            .into_iter()
            .map(|p| {
                let dist = consumer_coords.and_then(|(clat, clon)| {
                    p.marchand_latitude
                        .zip(p.marchand_longitude)
                        .map(|(mlat, mlon)| distance_km(clat, clon, mlat, mlon))
                });
                let mut dto: OfferDto = p.into();
                dto.distance_km = dist;
                dto
            })
            .collect();

        // Sans position consommateur, on garde l'ordre du repo (retrait_fin
        // ASC). Avec position : proximité d'abord, ceux sans coordonnées
        // marchand (distance inconnue) relégués à la fin plutôt qu'exclus.
        if consumer_coords.is_some() {
            offers.sort_by(|a, b| match (a.distance_km, b.distance_km) {
                (Some(da), Some(db)) => da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            });
        }

        Ok(offers)
    }

    pub async fn get_offer(
        &self,
        id: Uuid,
        consumer_coords: Option<(f64, f64)>,
    ) -> Result<OfferDto, CatalogError> {
        let product = self
            .product_repo
            .find_with_merchant(id)
            .await?
            .ok_or(CatalogError::NotFound)?;
        let dist = consumer_coords.and_then(|(clat, clon)| {
            product
                .marchand_latitude
                .zip(product.marchand_longitude)
                .map(|(mlat, mlon)| distance_km(clat, clon, mlat, mlon))
        });
        let mut dto: OfferDto = product.into();
        dto.distance_km = dist;
        Ok(dto)
    }

    pub async fn get_merchant_profile(
        &self,
        id: Uuid,
        consumer_coords: Option<(f64, f64)>,
    ) -> Result<MerchantProfileDto, CatalogError> {
        let merchant = self
            .merchant_repo
            .find_by_id(id)
            .await?
            .ok_or(CatalogError::NotFound)?;
        let offres = self.product_repo.list_active_by_merchant(id).await?;

        let dist = consumer_coords.and_then(|(clat, clon)| {
            merchant
                .latitude
                .zip(merchant.longitude)
                .map(|(mlat, mlon)| distance_km(clat, clon, mlat, mlon))
        });
        let mut marchand: crate::application::dto::MerchantResponseDto = merchant.into();
        marchand.distance_km = dist;

        Ok(MerchantProfileDto {
            marchand,
            offres: offres.into_iter().map(Into::into).collect(),
        })
    }
}
