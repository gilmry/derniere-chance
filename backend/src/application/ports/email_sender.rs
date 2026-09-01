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

    /// Lien de réinitialisation de mot de passe. `reset_url` porte le jeton en
    /// clair : c'est le seul endroit où il existe encore, la base n'en garde
    /// que l'empreinte.
    async fn send_password_reset(
        &self,
        to_email: &str,
        reset_url: &str,
        expires_in_minutes: i64,
    ) -> Result<(), EmailError>;
}
