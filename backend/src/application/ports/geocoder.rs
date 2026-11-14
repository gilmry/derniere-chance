use async_trait::async_trait;
use thiserror::Error;

/// Point GPS d'une adresse.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coordonnees {
    pub latitude: f64,
    pub longitude: f64,
}

/// Établissement trouvé : sa position, et son adresse telle que la connaît
/// le service (« 3 Rue des Comptoirs, 35410 Châteaugiron »), quand il la
/// connaît assez pour la formuler.
#[derive(Debug, Clone, PartialEq)]
pub struct Etablissement {
    pub position: Coordonnees,
    pub adresse: Option<String>,
}

#[derive(Debug, Error)]
pub enum GeocodeError {
    /// Le service ne répond pas, ou répond quelque chose d'illisible. Rien
    /// n'est enregistré dans ce cas : mieux vaut faire réessayer le
    /// commerçant que de placer son commerce au mauvais endroit.
    #[error("service de géocodage indisponible : {0}")]
    Indisponible(String),
}

/// Transforme l'adresse saisie par un commerçant en point sur la carte.
/// Adaptateurs : `infrastructure::geocoding` (Nominatim/OpenStreetMap en
/// production, un géocodeur fictif en dev et en e2e).
#[async_trait]
pub trait Geocoder: Send + Sync {
    /// `pays` est un code ISO alpha-2 déjà validé (`domain::services::pays`).
    /// `Ok(None)` : le service a répondu mais ne connaît pas cette adresse
    /// dans ce pays.
    async fn geocode(&self, adresse: &str, pays: &str)
        -> Result<Option<Coordonnees>, GeocodeError>;

    /// Cherche un établissement nommé `nom` tout près de `autour` (le point
    /// de son adresse) : c'est ce qui prouve que le commerce existe, et pas
    /// seulement la rue. `Ok(None)` : aucun lieu de ce nom à cet endroit.
    async fn trouver_etablissement(
        &self,
        nom: &str,
        pays: &str,
        autour: Coordonnees,
    ) -> Result<Option<Etablissement>, GeocodeError>;
}

/// Propose des adresses complètes pendant que le commerçant tape la sienne.
/// Séparé de `Geocoder` : la recherche à la frappe doit être tolérante (mots
/// incomplets), alors que la validation de l'adresse doit être stricte pour
/// repérer une adresse qui n'existe pas.
#[async_trait]
pub trait AddressSuggester: Send + Sync {
    /// `pays` est un code ISO alpha-2 déjà validé ; seules les adresses de ce
    /// pays sont renvoyées, prêtes à afficher et à envoyer telles quelles.
    async fn suggest(&self, saisie: &str, pays: &str) -> Result<Vec<String>, GeocodeError>;
}
