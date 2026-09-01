//! Adaptateurs d'envoi d'email, tous derrière le port
//! `application::ports::EmailSender`.
//!
//! Le corps des messages est rendu par [`message`], commun à tous : changer de
//! transporteur ne change rien à ce que le destinataire lit.

use std::sync::Arc;

use crate::application::ports::EmailSender;

mod logging_email_sender;
mod message;
mod smtp_email_sender;

pub use logging_email_sender::LoggingEmailSender;
pub use smtp_email_sender::SmtpEmailSender;

/// Nom d'expéditeur affiché quand la configuration n'en impose pas d'autre.
const DEFAULT_FROM_NAME: &str = "DernièreChance";

/// Choisit l'adaptateur d'après la configuration présente et renvoie aussi son
/// nom, pour que le démarrage dise lequel est en service.
///
/// Sans configuration SMTP, l'adaptateur qui journalise prend le relais, pour
/// qu'un poste de dev, la CI et les e2e tournent sans rien envoyer à de vraies
/// personnes.
///
/// Exposé plutôt qu'écrit dans `main` pour que le test de fumée
/// (`tests/email_smoke.rs`) éprouve exactement l'adaptateur qui tournera en
/// production, et non une construction parallèle qui pourrait diverger.
pub fn sender_from_env() -> (Arc<dyn EmailSender>, &'static str) {
    match SmtpEmailSender::from_env() {
        Some(smtp) => (Arc::new(smtp), "smtp"),
        None => (Arc::new(LoggingEmailSender), "journalisation seule"),
    }
}

/// Base des liens insérés dans les emails (fiche offre, profil).
fn app_base_url() -> String {
    non_empty_env("APP_BASE_URL")
        .unwrap_or_else(|| "https://derniere-chance.ecosolva.org".to_string())
        .trim_end_matches('/')
        .to_string()
}

/// Les valeurs vides comptent comme absentes : docker-compose passe
/// `${SMTP_TOKEN:-}`, donc une variable non renseignée arrive comme chaîne
/// vide plutôt qu'absente.
fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
