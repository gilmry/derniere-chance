use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::application::ports::{AddressSuggester, GeocodeError};

/// Écart minimal entre deux requêtes envoyées au serveur public de Photon,
/// pour rester un usager raisonnable même si plusieurs personnes tapent en
/// même temps.
const ECART_MINIMAL: Duration = Duration::from_millis(250);
const DELAI_MAXIMAL: Duration = Duration::from_secs(5);
/// Demandes en attente au plus : au-delà, refus immédiat (voir `FileBornee`).
static FILE: super::file::FileBornee = super::file::FileBornee::new(8);
/// Photon ne filtre pas par pays : on en demande davantage et on garde ceux
/// du pays choisi.
const RESULTATS_DEMANDES: usize = 15;
const RESULTATS_RENDUS: usize = 5;

/// Suggestions d'adresses par Photon (komoot), moteur de recherche à la
/// frappe fondé sur OpenStreetMap. Nominatim, qui valide l'adresse, interdit
/// justement cet usage sur son serveur public : d'où un second service.
/// Les requêtes passent par le backend, jamais directement du navigateur :
/// l'adresse IP des visiteurs n'est pas transmise à un tiers.
pub struct PhotonSuggester {
    base_url: String,
    client: reqwest::Client,
    derniere_requete: Mutex<Option<Instant>>,
}

#[derive(Debug, Deserialize)]
struct Reponse {
    features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
struct Feature {
    properties: Proprietes,
}

#[derive(Debug, Default, Deserialize)]
struct Proprietes {
    name: Option<String>,
    housenumber: Option<String>,
    street: Option<String>,
    postcode: Option<String>,
    city: Option<String>,
    countrycode: Option<String>,
}

impl PhotonSuggester {
    pub fn new(base_url: String, app_base_url: &str) -> Self {
        let user_agent = format!(
            "DerniereChance/{} (+{})",
            env!("CARGO_PKG_VERSION"),
            app_base_url
        );
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .timeout(DELAI_MAXIMAL)
            .build()
            .expect("client HTTP des suggestions d'adresses");
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
            derniere_requete: Mutex::new(None),
        }
    }

    fn url_de_recherche(&self, saisie: &str) -> String {
        format!(
            "{}/api/?limit={}&lang=fr&q={}",
            self.base_url,
            RESULTATS_DEMANDES,
            urlencoding::encode(saisie.trim())
        )
    }
}

#[async_trait]
impl AddressSuggester for PhotonSuggester {
    async fn suggest(&self, saisie: &str, pays: &str) -> Result<Vec<String>, GeocodeError> {
        let _place = FILE
            .entrer()
            .ok_or_else(|| GeocodeError::Indisponible("trop de demandes en attente".into()))?;
        let mut derniere = self.derniere_requete.lock().await;
        if let Some(instant) = *derniere {
            let ecoule = instant.elapsed();
            if ecoule < ECART_MINIMAL {
                tokio::time::sleep(ECART_MINIMAL - ecoule).await;
            }
        }
        let reponse = self.client.get(self.url_de_recherche(saisie)).send().await;
        *derniere = Some(Instant::now());
        drop(derniere);

        let reponse: Reponse = reponse
            .and_then(|r| r.error_for_status())
            .map_err(|err| GeocodeError::Indisponible(err.to_string()))?
            .json()
            .await
            .map_err(|err| GeocodeError::Indisponible(err.to_string()))?;
        Ok(libelles(reponse, pays))
    }
}

/// Libellés « 12 Rue de la Gare, 35410 Châteaugiron » des résultats du pays,
/// sans doublon, dans l'ordre de pertinence de Photon.
fn libelles(reponse: Reponse, pays: &str) -> Vec<String> {
    let mut vus = Vec::new();
    for feature in reponse.features {
        let p = feature.properties;
        if !p
            .countrycode
            .as_deref()
            .is_some_and(|c| c.eq_ignore_ascii_case(pays))
        {
            continue;
        }
        let Some(libelle) = libelle(&p) else {
            continue;
        };
        if !vus.contains(&libelle) {
            vus.push(libelle);
        }
        if vus.len() == RESULTATS_RENDUS {
            break;
        }
    }
    vus
}

/// Une adresse n'est proposée que si elle a une rue (ou un nom de lieu) et une
/// ville : une ville seule ou un pays ne sont pas l'adresse d'un commerce.
fn libelle(p: &Proprietes) -> Option<String> {
    let voie = match (&p.housenumber, &p.street, &p.name) {
        (Some(numero), Some(rue), _) => format!("{numero} {rue}"),
        (None, Some(rue), _) => rue.clone(),
        (_, None, Some(nom)) => nom.clone(),
        _ => return None,
    };
    let ville = p.city.as_ref()?;
    Some(match &p.postcode {
        Some(code) => format!("{voie}, {code} {ville}"),
        None => format!("{voie}, {ville}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPONSE: &str = r#"{"type":"FeatureCollection","features":[
        {"properties":{"housenumber":"12","street":"Rue de la Gare","postcode":"35410","city":"Châteaugiron","countrycode":"FR","type":"house"}},
        {"properties":{"housenumber":"12","street":"Rue de la Gare","postcode":"35410","city":"Châteaugiron","countrycode":"FR","type":"house"}},
        {"properties":{"housenumber":"12","street":"Rue de la Gare","postcode":"1000","city":"Bruxelles","countrycode":"BE","type":"house"}},
        {"properties":{"name":"Châteaugiron","countrycode":"FR","type":"city"}},
        {"properties":{"name":"Halles","postcode":"35410","city":"Châteaugiron","countrycode":"FR","type":"house"}}
    ]}"#;

    #[test]
    fn garde_les_adresses_du_pays_sans_doublon_ni_ville_seule() {
        let reponse: Reponse = serde_json::from_str(REPONSE).unwrap();
        assert_eq!(
            libelles(reponse, "FR"),
            vec![
                "12 Rue de la Gare, 35410 Châteaugiron".to_string(),
                "Halles, 35410 Châteaugiron".to_string(),
            ]
        );
    }

    #[test]
    fn filtre_sur_le_pays_choisi() {
        let reponse: Reponse = serde_json::from_str(REPONSE).unwrap();
        assert_eq!(
            libelles(reponse, "be"),
            vec!["12 Rue de la Gare, 1000 Bruxelles".to_string()]
        );
    }

    #[test]
    fn l_url_encode_la_saisie() {
        let s = PhotonSuggester::new("https://photon.example/".into(), "https://app.example");
        assert_eq!(
            s.url_de_recherche(" 12 rue de la gare "),
            "https://photon.example/api/?limit=15&lang=fr&q=12%20rue%20de%20la%20gare"
        );
    }
}
