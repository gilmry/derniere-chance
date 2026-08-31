use async_trait::async_trait;
use thiserror::Error;

use crate::domain::entities::{Merchant, Product};

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("failed to send email: {0}")]
    SendFailed(String),
}

/// Outbound email port. Two adapters sit behind it:
/// `infrastructure::email::MailjetEmailSender` (envoi réel, retenu quand la
/// configuration Mailjet est présente) et `LoggingEmailSender`, qui se
/// contente de logguer et sert de repli en dev, en CI et dans les tests - le
/// flux de notification ne dépend donc d'aucun appel réseau.
#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send_new_offer_notification(
        &self,
        to_email: &str,
        merchant: &Merchant,
        product: &Product,
    ) -> Result<(), EmailError>;
}
