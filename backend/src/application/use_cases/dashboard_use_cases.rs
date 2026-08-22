use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::{ConsumerProfileDto, MerchantDashboardDto};
use crate::application::ports::{RepoError, ReservationRepository};

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

pub struct DashboardUseCases {
    reservation_repo: Arc<dyn ReservationRepository>,
}

impl DashboardUseCases {
    pub fn new(reservation_repo: Arc<dyn ReservationRepository>) -> Self {
        Self { reservation_repo }
    }

    pub async fn merchant_today(
        &self,
        marchand_id: Uuid,
    ) -> Result<MerchantDashboardDto, DashboardError> {
        let stats = self.reservation_repo.merchant_daily_stats(marchand_id).await?;
        Ok(stats.into())
    }

    pub async fn consumer_profile(
        &self,
        consommateur_id: Uuid,
    ) -> Result<ConsumerProfileDto, DashboardError> {
        let stats = self.reservation_repo.consumer_stats(consommateur_id).await?;
        Ok(stats.into())
    }
}
