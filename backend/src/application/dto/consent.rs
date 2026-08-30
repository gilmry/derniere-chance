use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Corps du POST de consentement. La version est renvoyée par le client pour
/// que le serveur puisse vérifier que la personne a bien coché la case en
/// face du texte *actuellement* en vigueur, et pas d'une version périmée
/// restée affichée dans un onglet.
#[derive(Debug, Deserialize)]
pub struct GrantConsentRequest {
    pub consent_version: String,
}

#[derive(Debug, Serialize)]
pub struct ConsentStatusDto {
    /// Vrai seulement si le consentement porte sur `version_courante` : une
    /// nouvelle version du texte redemande donc l'accord, comme l'exige un
    /// changement substantiel du traitement.
    pub consenti: bool,
    pub version_acceptee: Option<String>,
    pub accepte_le: Option<DateTime<Utc>>,
    pub version_courante: String,
}
