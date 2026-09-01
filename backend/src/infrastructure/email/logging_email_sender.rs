use async_trait::async_trait;

use crate::application::ports::{EmailError, EmailSender};
use crate::domain::entities::{Merchant, Product};

/// Adaptateur `EmailSender` de repli : journalise ce qui serait envoyé au
/// lieu d'appeler un fournisseur. `main` le retient quand la configuration
/// Mailjet est absente (poste de dev, CI, e2e), pour qu'aucun email ne parte
/// vers de vraies personnes depuis un jeu de données de test. L'envoi réel
/// passe par `super::MailjetEmailSender`.
pub struct LoggingEmailSender;

#[async_trait]
impl EmailSender for LoggingEmailSender {
    async fn send_new_offer_notification(
        &self,
        to_email: &str,
        merchant: &Merchant,
        product: &Product,
    ) -> Result<(), EmailError> {
        tracing::info!(
            to = %to_email,
            marchand = %merchant.nom,
            produit = %product.nom,
            "would send: new démarque notification email"
        );
        Ok(())
    }

    async fn send_password_reset(
        &self,
        to_email: &str,
        reset_url: &str,
        expires_in_minutes: i64,
    ) -> Result<(), EmailError> {
        // Le lien est journalisé en clair, à dessein : sans transporteur
        // configuré, c'est le seul moyen d'éprouver le parcours en dev. Ces
        // journaux ne quittent pas la machine de développement, et en
        // production un transporteur est configuré.
        tracing::info!(
            to = %to_email,
            %reset_url,
            expires_in_minutes,
            "would send: password reset email"
        );
        Ok(())
    }
}
