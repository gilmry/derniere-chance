use async_trait::async_trait;
use thiserror::Error;

use crate::domain::entities::{Merchant, Product};

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("failed to send email: {0}")]
    SendFailed(String),
}

/// Outbound email port. No concrete transactional-email provider is wired up
/// yet (see VISION.md §8) - `infrastructure::email::LoggingEmailSender` is a
/// stand-in adapter that only logs, so the notification flow (and its tests)
/// don't depend on a real SMTP/API integration. Swap in a Resend/SMTP adapter
/// behind this same trait when that's decided, with no change to the use cases.
#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_new_offer_notification(
        &self,
        to_email: &str,
        merchant: &Merchant,
        product: &Product,
    ) -> Result<(), EmailError>;
}
