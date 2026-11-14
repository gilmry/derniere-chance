//! Adaptateurs de géocodage (adresse → position), tous derrière le port
//! `application::ports::Geocoder`.
//!
//! - [`NominatimGeocoder`] : Nominatim, le moteur de recherche d'adresses
//!   d'OpenStreetMap. Couvre toute l'Europe, sans clé. C'est l'adaptateur par
//!   défaut, donc celui de la production.
//! - [`FictifGeocoder`] : répond sans réseau, pour le dev et les e2e, dont les
//!   adresses sont fictives (« Rue du Test 1, Bruxelles »).
//! - [`PhotonSuggester`] / [`FictifSuggester`] : suggestions d'adresses à la
//!   frappe (port `AddressSuggester`), choisies avec le géocodeur.
//!
//! Changer de fournisseur (OpenCage, Nominatim auto-hébergé...) ne demande
//! qu'un nouvel adaptateur ici : le reste du backend ne voit que le port.

use std::sync::Arc;

use crate::application::ports::{AddressSuggester, Geocoder};

mod fictif;
mod file;
mod nominatim;
mod photon;

pub use fictif::{FictifGeocoder, FictifSuggester};
pub use nominatim::NominatimGeocoder;
pub use photon::PhotonSuggester;

/// Serveur public de Photon (komoot), fait pour la recherche à la frappe.
const PHOTON_URL_PAR_DEFAUT: &str = "https://photon.komoot.io";

/// Serveur public d'OpenStreetMap. Sa politique d'usage : une requête par
/// seconde au plus et une application identifiée (User-Agent), ce que
/// l'adaptateur respecte. On ne géocode qu'à l'inscription et au changement
/// d'adresse, très loin de cette limite.
const NOMINATIM_URL_PAR_DEFAUT: &str = "https://nominatim.openstreetmap.org";

/// Suggestions d'adresses assorties au géocodeur : fictives avec le
/// géocodeur fictif, Photon sinon (`PHOTON_URL` pour un serveur à soi).
pub fn suggester_from_env(app_base_url: &str) -> Arc<dyn AddressSuggester> {
    if non_empty_env("GEOCODEUR").as_deref() == Some("fictif") {
        return Arc::new(FictifSuggester);
    }
    let url = non_empty_env("PHOTON_URL").unwrap_or_else(|| PHOTON_URL_PAR_DEFAUT.into());
    Arc::new(PhotonSuggester::new(url, app_base_url))
}

/// Choisit l'adaptateur d'après `GEOCODEUR` (`nominatim` par défaut, ou
/// `fictif`) et renvoie aussi son nom, pour que le démarrage dise lequel est
/// en service.
pub fn geocoder_from_env(app_base_url: &str) -> (Arc<dyn Geocoder>, &'static str) {
    match non_empty_env("GEOCODEUR").as_deref() {
        Some("fictif") => (Arc::new(FictifGeocoder), "fictif"),
        Some(other) if other != "nominatim" => {
            tracing::warn!(
                geocodeur = other,
                "GEOCODEUR inconnu, Nominatim est utilisé (valeurs possibles : nominatim, fictif)"
            );
            (nominatim_from_env(app_base_url), "nominatim")
        }
        _ => (nominatim_from_env(app_base_url), "nominatim"),
    }
}

fn nominatim_from_env(app_base_url: &str) -> Arc<dyn Geocoder> {
    let url = non_empty_env("NOMINATIM_URL").unwrap_or_else(|| NOMINATIM_URL_PAR_DEFAUT.into());
    Arc::new(NominatimGeocoder::new(url, app_base_url))
}

/// Les valeurs vides comptent comme absentes : docker-compose passe
/// `${GEOCODEUR:-}`, donc une variable non renseignée arrive comme chaîne vide.
fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
