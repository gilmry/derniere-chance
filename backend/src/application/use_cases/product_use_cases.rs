use std::sync::Arc;

use thiserror::Error;
use uuid::Uuid;

use crate::application::dto::{CreateProductDto, ProductResponseDto};
use crate::application::ports::{
    EmailSender, MerchantRepository, NewNotification, NewProduct, NotificationRepository,
    ProductRepository, RepoError, SubscriptionRepository,
};
use crate::domain::entities::{NotificationStatus, ProductStatus};

#[derive(Debug, Error)]
pub enum ProductError {
    #[error("resource not found")]
    NotFound,
    #[error("access denied: this produit belongs to another marchand")]
    Forbidden,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

/// Marchand backoffice use cases: publish a démarque (and notify followers),
/// mark one as écoulé, and list the marchand's own produits for the
/// dashboard.
pub struct ProductUseCases {
    product_repo: Arc<dyn ProductRepository>,
    merchant_repo: Arc<dyn MerchantRepository>,
    subscription_repo: Arc<dyn SubscriptionRepository>,
    notification_repo: Arc<dyn NotificationRepository>,
    email_sender: Arc<dyn EmailSender>,
}

impl ProductUseCases {
    pub fn new(
        product_repo: Arc<dyn ProductRepository>,
        merchant_repo: Arc<dyn MerchantRepository>,
        subscription_repo: Arc<dyn SubscriptionRepository>,
        notification_repo: Arc<dyn NotificationRepository>,
        email_sender: Arc<dyn EmailSender>,
    ) -> Self {
        Self {
            product_repo,
            merchant_repo,
            subscription_repo,
            notification_repo,
            email_sender,
        }
    }

    pub async fn publish(
        &self,
        marchand_id: Uuid,
        dto: CreateProductDto,
    ) -> Result<ProductResponseDto, ProductError> {
        if dto.prix_demarque > dto.prix_initial {
            return Err(ProductError::InvalidInput(
                "le prix démarqué doit être inférieur ou égal au prix initial".into(),
            ));
        }
        if dto.quantite <= 0 {
            return Err(ProductError::InvalidInput(
                "la quantité doit être positive".into(),
            ));
        }
        if dto.retrait_fin <= dto.retrait_debut {
            return Err(ProductError::InvalidInput(
                "la fin du retrait doit être après le début".into(),
            ));
        }

        let merchant = self
            .merchant_repo
            .find_by_id(marchand_id)
            .await?
            .ok_or(ProductError::NotFound)?;

        let product = self
            .product_repo
            .create(NewProduct {
                marchand_id,
                nom: dto.nom,
                description: dto.description,
                prix_initial: dto.prix_initial,
                prix_demarque: dto.prix_demarque,
                quantite: dto.quantite,
                retrait_debut: dto.retrait_debut,
                retrait_fin: dto.retrait_fin,
            })
            .await?;

        self.notify_subscribers(&merchant, &product).await;

        Ok(product.into())
    }

    /// Best-effort fan-out: a slow or failing email adapter must never break
    /// publishing a démarque, so failures are logged (as `Echouee`) rather
    /// than surfaced to the marchand.
    async fn notify_subscribers(
        &self,
        merchant: &crate::domain::entities::Merchant,
        product: &crate::domain::entities::Product,
    ) {
        let subscribers = match self.subscription_repo.list_subscribers(merchant.id).await {
            Ok(subs) => subs,
            Err(err) => {
                tracing::error!(?err, "failed to list subscribers, skipping notifications");
                return;
            }
        };

        for subscriber in subscribers {
            let sent = self
                .email_sender
                .send_new_offer_notification(&subscriber.email, merchant, product)
                .await;

            let statut = if sent.is_ok() {
                NotificationStatus::Envoyee
            } else {
                tracing::warn!(email = %subscriber.email, "notification email failed");
                NotificationStatus::Echouee
            };

            if let Err(err) = self
                .notification_repo
                .log(NewNotification {
                    produit_id: product.id,
                    consommateur_id: subscriber.consommateur_id,
                    statut,
                })
                .await
            {
                tracing::error!(?err, "failed to log notification");
            }
        }
    }

    pub async fn mark_ecoule(
        &self,
        marchand_id: Uuid,
        product_id: Uuid,
    ) -> Result<ProductResponseDto, ProductError> {
        let product = self.owned_product(marchand_id, product_id).await?;
        let updated = self
            .product_repo
            .update_status(product.id, ProductStatus::Ecoule)
            .await?;
        Ok(updated.into())
    }

    pub async fn list_mine(
        &self,
        marchand_id: Uuid,
    ) -> Result<Vec<ProductResponseDto>, ProductError> {
        let products = self.product_repo.list_by_merchant(marchand_id).await?;
        Ok(products.into_iter().map(Into::into).collect())
    }

    async fn owned_product(
        &self,
        marchand_id: Uuid,
        product_id: Uuid,
    ) -> Result<crate::domain::entities::Product, ProductError> {
        let product = self
            .product_repo
            .find_by_id(product_id)
            .await?
            .ok_or(ProductError::NotFound)?;
        if product.marchand_id != marchand_id {
            return Err(ProductError::Forbidden);
        }
        Ok(product)
    }
}
