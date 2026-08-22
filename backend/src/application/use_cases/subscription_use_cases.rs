use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::MerchantResponseDto;
use crate::application::ports::{MerchantRepository, RepoError, SubscriptionRepository};

#[derive(Debug, Error)]
pub enum SubscriptionError {
    #[error("marchand not found")]
    MerchantNotFound,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

/// Consommateur-side "marchand ami" follow/unfollow and the followed list
/// shown on their profile.
pub struct SubscriptionUseCases {
    subscription_repo: Arc<dyn SubscriptionRepository>,
    merchant_repo: Arc<dyn MerchantRepository>,
}

impl SubscriptionUseCases {
    pub fn new(
        subscription_repo: Arc<dyn SubscriptionRepository>,
        merchant_repo: Arc<dyn MerchantRepository>,
    ) -> Self {
        Self {
            subscription_repo,
            merchant_repo,
        }
    }

    pub async fn follow(
        &self,
        consommateur_id: Uuid,
        marchand_id: Uuid,
    ) -> Result<(), SubscriptionError> {
        if self.merchant_repo.find_by_id(marchand_id).await?.is_none() {
            return Err(SubscriptionError::MerchantNotFound);
        }
        self.subscription_repo
            .follow(consommateur_id, marchand_id)
            .await?;
        Ok(())
    }

    pub async fn unfollow(
        &self,
        consommateur_id: Uuid,
        marchand_id: Uuid,
    ) -> Result<(), SubscriptionError> {
        self.subscription_repo
            .unfollow(consommateur_id, marchand_id)
            .await?;
        Ok(())
    }

    pub async fn list_followed(
        &self,
        consommateur_id: Uuid,
    ) -> Result<Vec<MerchantResponseDto>, SubscriptionError> {
        let merchants = self
            .subscription_repo
            .list_followed_merchants(consommateur_id)
            .await?;
        Ok(merchants.into_iter().map(Into::into).collect())
    }
}
