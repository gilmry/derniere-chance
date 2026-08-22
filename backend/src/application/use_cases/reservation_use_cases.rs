use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::ReservationConfirmationDto;
use crate::application::ports::{
    NewReservation, ProductRepository, RepoError, ReservationRepository,
};
use crate::domain::entities::ReservationStatus;
use crate::domain::services::reservation_code;

#[derive(Debug, Error)]
pub enum ReservationError {
    #[error("produit not found")]
    ProductNotFound,
    #[error("ce panier est épuisé")]
    SoldOut,
    #[error("code de retrait introuvable")]
    ReservationNotFound,
    #[error("ce panier appartient à un autre marchand")]
    Forbidden,
    #[error("ce code a déjà été utilisé")]
    AlreadyRedeemed,
    #[error("impossible de générer un code de retrait unique, réessayez")]
    CodeGenerationFailed,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

const MAX_CODE_ATTEMPTS: u8 = 5;

/// Reserving a panier and a marchand redeeming its pickup code in store.
pub struct ReservationUseCases {
    reservation_repo: Arc<dyn ReservationRepository>,
    product_repo: Arc<dyn ProductRepository>,
}

impl ReservationUseCases {
    pub fn new(
        reservation_repo: Arc<dyn ReservationRepository>,
        product_repo: Arc<dyn ProductRepository>,
    ) -> Self {
        Self {
            reservation_repo,
            product_repo,
        }
    }

    pub async fn reserve(
        &self,
        consommateur_id: Uuid,
        produit_id: Uuid,
    ) -> Result<ReservationConfirmationDto, ReservationError> {
        // Atomic decrement first: whoever claims the last unit wins, everyone
        // else fails fast instead of racing to insert a reservation.
        self.product_repo
            .reserve_unit(produit_id)
            .await
            .map_err(|err| match err {
                RepoError::NotFound => ReservationError::SoldOut,
                other => other.into(),
            })?;

        let mut last_error = ReservationError::CodeGenerationFailed;
        for _ in 0..MAX_CODE_ATTEMPTS {
            let code = reservation_code::generate();
            match self
                .reservation_repo
                .create(NewReservation {
                    produit_id,
                    consommateur_id,
                    code,
                })
                .await
            {
                Ok(reservation) => {
                    let offer = self
                        .product_repo
                        .find_with_merchant(produit_id)
                        .await?
                        .ok_or(ReservationError::ProductNotFound)?;
                    return Ok(ReservationConfirmationDto::new(reservation, offer));
                }
                Err(RepoError::Conflict(_)) => continue,
                Err(other) => {
                    last_error = other.into();
                    break;
                }
            }
        }

        Err(last_error)
    }

    pub async fn validate_pickup(
        &self,
        marchand_id: Uuid,
        code: &str,
    ) -> Result<(), ReservationError> {
        let reservation = self
            .reservation_repo
            .find_by_code(code)
            .await?
            .ok_or(ReservationError::ReservationNotFound)?;

        let product = self
            .product_repo
            .find_by_id(reservation.produit_id)
            .await?
            .ok_or(ReservationError::ProductNotFound)?;

        if product.marchand_id != marchand_id {
            return Err(ReservationError::Forbidden);
        }
        if reservation.statut != ReservationStatus::Reservee {
            return Err(ReservationError::AlreadyRedeemed);
        }

        self.reservation_repo.mark_recuperee(reservation.id).await?;
        Ok(())
    }
}
