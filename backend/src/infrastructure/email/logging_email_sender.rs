use async_trait::async_trait;

use crate::application::ports::{EmailError, EmailSender};
use crate::domain::entities::{Merchant, Product};

/// Placeholder `EmailSender` adapter: logs what would be sent instead of
/// calling a real provider. No transactional-email service is wired up yet
/// (see VISION.md §8) - replace this with a Resend/SMTP adapter behind the
/// same `EmailSender` port when that's decided; nothing above this line in
/// the call stack (use cases, handlers) needs to change.
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
}
