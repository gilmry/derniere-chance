use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Trace d'un consentement au programme bêta. `version` est la version du
/// texte effectivement affiché au moment de l'acceptation : c'est elle qui
/// permet de démontrer *à quoi* la personne a consenti (RGPD art. 7 §1), pas
/// seulement qu'elle a coché une case.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct BetaConsent {
    pub id: Uuid,
    pub consommateur_id: Uuid,
    pub version: String,
    pub accepte_le: DateTime<Utc>,
    /// `None` tant que le consentement est actif.
    pub retire_le: Option<DateTime<Utc>>,
}
