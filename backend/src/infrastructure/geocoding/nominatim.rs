use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::Mutex;
use tokio::time::Instant;

use crate::application::ports::{Coordonnees, Etablissement, GeocodeError, Geocoder};

/// Écart minimal entre deux requêtes : la politique d'usage du serveur public
/// d'OpenStreetMap n'en autorise qu'une par seconde.
const ECART_MINIMAL: Duration = Duration::from_millis(1100);
const DELAI_MAXIMAL: Duration = Duration::from_secs(8);
/// Demandes en attente au plus : au-delà, refus immédiat (voir `FileBornee`).
static FILE: super::file::FileBornee = super::file::FileBornee::new(6);

/// Demi-côtés, en degrés, des carrés où l'on cherche l'établissement autour
/// du point de son adresse : environ 300 m d'abord, puis 1,5 km. Le second
/// couvre le cas fréquent d'un numéro absent d'OpenStreetMap : Nominatim
/// renvoie alors le milieu de la rue, parfois loin du commerce.
const RAYONS_ETABLISSEMENT: [f64; 2] = [0.003, 0.015];

/// Catégories OpenStreetMap qui ne sont pas des établissements : une rue
/// « du Marché » ou un quartier ne prouvent pas qu'un commerce « Le Marché »
/// existe.
const PAS_UN_ETABLISSEMENT: [&str; 5] = ["highway", "place", "boundary", "landuse", "railway"];

/// Géocodeur Nominatim (OpenStreetMap), recherche bornée au pays du commerce.
pub struct NominatimGeocoder {
    base_url: String,
    client: reqwest::Client,
    /// Instant de la dernière requête. Le verrou est gardé pendant l'attente
    /// et l'appel, ce qui met les requêtes en file une par une.
    derniere_requete: Mutex<Option<Instant>>,
}

#[derive(Debug, Deserialize)]
struct Resultat {
    /// Nominatim renvoie les coordonnées en texte (`"50.8467"`).
    lat: String,
    lon: String,
    /// Famille du lieu (`shop`, `amenity`, `highway`...).
    #[serde(default)]
    category: String,
    /// Détail de l'adresse, demandé (`addressdetails=1`) pour un établissement.
    #[serde(default)]
    address: Option<AdresseOsm>,
}

#[derive(Debug, Default, Deserialize)]
struct AdresseOsm {
    house_number: Option<String>,
    road: Option<String>,
    postcode: Option<String>,
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
    municipality: Option<String>,
}

impl AdresseOsm {
    /// « 3 Rue des Comptoirs, 35410 Châteaugiron ». Sans rue, pas d'adresse :
    /// mieux vaut garder la saisie du commerçant qu'une adresse tronquée.
    fn libelle(&self) -> Option<String> {
        let rue = self.road.as_deref()?.trim();
        let rue = match self.house_number.as_deref() {
            Some(numero) => format!("{} {rue}", numero.trim()),
            None => rue.to_string(),
        };
        let ville = self
            .city
            .as_deref()
            .or(self.town.as_deref())
            .or(self.village.as_deref())
            .or(self.municipality.as_deref());
        let localite = [self.postcode.as_deref(), ville]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" ");
        Some(if localite.is_empty() {
            rue
        } else {
            format!("{rue}, {localite}")
        })
    }
}

impl NominatimGeocoder {
    /// `app_base_url` identifie l'application dans le User-Agent, comme le
    /// demande la politique d'usage de Nominatim.
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
            .expect("client HTTP du géocodeur");
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
            derniere_requete: Mutex::new(None),
        }
    }

    fn url_de_recherche(&self, adresse: &str, pays: &str) -> String {
        format!(
            "{}/search?format=jsonv2&limit=1&q={}&countrycodes={}",
            self.base_url,
            urlencoding::encode(adresse.trim()),
            pays.to_ascii_lowercase()
        )
    }

    /// Recherche du nom bornée (`bounded=1`) à un petit carré autour du point.
    fn url_d_etablissement(&self, nom: &str, pays: &str, autour: Coordonnees, r: f64) -> String {
        format!(
            "{}/search?format=jsonv2&addressdetails=1&limit=5&q={}&countrycodes={}&bounded=1&viewbox={:.5},{:.5},{:.5},{:.5}",
            self.base_url,
            urlencoding::encode(nom.trim()),
            pays.to_ascii_lowercase(),
            autour.longitude - r,
            autour.latitude + r,
            autour.longitude + r,
            autour.latitude - r,
        )
    }

    /// Une requête, au plus une par seconde quel que soit l'appelant.
    async fn interroger(&self, url: String) -> Result<Vec<Resultat>, GeocodeError> {
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
        let reponse = self
            .client
            .get(url)
            .header(reqwest::header::ACCEPT_LANGUAGE, "fr")
            .send()
            .await;
        *derniere = Some(Instant::now());
        drop(derniere);

        let reponse = reponse
            .and_then(|r| r.error_for_status())
            .map_err(|err| GeocodeError::Indisponible(err.to_string()))?;
        let resultats: Vec<Resultat> = reponse
            .json()
            .await
            .map_err(|err| GeocodeError::Indisponible(err.to_string()))?;
        Ok(resultats)
    }
}

#[async_trait]
impl Geocoder for NominatimGeocoder {
    async fn geocode(
        &self,
        adresse: &str,
        pays: &str,
    ) -> Result<Option<Coordonnees>, GeocodeError> {
        lire_premier(
            self.interroger(self.url_de_recherche(adresse, pays))
                .await?,
        )
    }

    async fn trouver_etablissement(
        &self,
        nom: &str,
        pays: &str,
        autour: Coordonnees,
    ) -> Result<Option<Etablissement>, GeocodeError> {
        for rayon in RAYONS_ETABLISSEMENT {
            let resultats = self
                .interroger(self.url_d_etablissement(nom, pays, autour, rayon))
                .await?;
            if let Some(trouve) = premier_etablissement(resultats)? {
                return Ok(Some(trouve));
            }
        }
        Ok(None)
    }
}

fn garder_les_etablissements(resultats: Vec<Resultat>) -> Vec<Resultat> {
    resultats
        .into_iter()
        .filter(|r| !PAS_UN_ETABLISSEMENT.contains(&r.category.as_str()))
        .collect()
}

fn premier_etablissement(resultats: Vec<Resultat>) -> Result<Option<Etablissement>, GeocodeError> {
    let mut etablissements = garder_les_etablissements(resultats);
    if etablissements.is_empty() {
        return Ok(None);
    }
    let premier = etablissements.swap_remove(0);
    let adresse = premier.address.as_ref().and_then(AdresseOsm::libelle);
    Ok(lire_premier(vec![premier])?.map(|position| Etablissement { position, adresse }))
}

fn lire_premier(resultats: Vec<Resultat>) -> Result<Option<Coordonnees>, GeocodeError> {
    let Some(premier) = resultats.into_iter().next() else {
        return Ok(None);
    };
    let latitude = premier.lat.parse::<f64>();
    let longitude = premier.lon.parse::<f64>();
    match (latitude, longitude) {
        (Ok(latitude), Ok(longitude))
            if (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude) =>
        {
            Ok(Some(Coordonnees {
                latitude,
                longitude,
            }))
        }
        _ => Err(GeocodeError::Indisponible(format!(
            "coordonnées illisibles : {} / {}",
            premier.lat, premier.lon
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn geocodeur() -> NominatimGeocoder {
        NominatimGeocoder::new("https://nominatim.example/".into(), "https://app.example")
    }

    #[test]
    fn l_url_borne_la_recherche_au_pays_et_encode_l_adresse() {
        let url = geocodeur().url_de_recherche(" 4 Rue du Marché, 1000 Bruxelles ", "BE");
        assert_eq!(
            url,
            "https://nominatim.example/search?format=jsonv2&limit=1\
             &q=4%20Rue%20du%20March%C3%A9%2C%201000%20Bruxelles&countrycodes=be"
        );
    }

    #[test]
    fn lit_la_reponse_de_nominatim() {
        let json =
            r#"[{"place_id":1,"lat":"50.8467","lon":"4.3525","display_name":"Grand-Place"}]"#;
        let resultats: Vec<Resultat> = serde_json::from_str(json).unwrap();
        assert_eq!(
            lire_premier(resultats).unwrap(),
            Some(Coordonnees {
                latitude: 50.8467,
                longitude: 4.3525
            })
        );
    }

    #[test]
    fn une_reponse_vide_veut_dire_adresse_introuvable() {
        assert_eq!(lire_premier(vec![]).unwrap(), None);
    }

    #[test]
    fn des_coordonnees_illisibles_sont_une_panne_pas_une_position() {
        let resultats = vec![Resultat {
            lat: "nord".into(),
            lon: "4.35".into(),
            category: "shop".into(),
            address: None,
        }];
        assert!(lire_premier(resultats).is_err());
    }

    #[test]
    fn l_etablissement_est_cherche_dans_un_petit_carre_autour_de_l_adresse() {
        let url = geocodeur().url_d_etablissement(
            "Chez Léa",
            "FR",
            Coordonnees {
                latitude: 48.0,
                longitude: -1.5,
            },
            0.003,
        );
        assert_eq!(
            url,
            "https://nominatim.example/search?format=jsonv2&addressdetails=1&limit=5&q=Chez%20L%C3%A9a\
             &countrycodes=fr&bounded=1&viewbox=-1.50300,48.00300,-1.49700,47.99700"
        );
    }

    #[test]
    fn une_rue_du_meme_nom_ne_prouve_pas_l_etablissement() {
        let json = r#"[{"lat":"48.0","lon":"-1.5","category":"highway"},
                       {"lat":"48.001","lon":"-1.501","category":"shop"}]"#;
        let resultats: Vec<Resultat> = serde_json::from_str(json).unwrap();
        assert_eq!(
            premier_etablissement(resultats)
                .unwrap()
                .map(|e| e.position),
            Some(Coordonnees {
                latitude: 48.001,
                longitude: -1.501
            })
        );
    }

    #[test]
    fn l_adresse_de_l_etablissement_est_reformulee() {
        let json = r#"[{"lat":"48.0429736","lon":"-1.5155596","category":"shop",
            "name":"Le Fournil des Saveurs",
            "address":{"shop":"Le Fournil des Saveurs","road":"Rue des Comptoirs",
              "hamlet":"Le Petit Launay","town":"Châteaugiron","municipality":"Rennes",
              "postcode":"35410","country_code":"fr"}}]"#;
        let resultats: Vec<Resultat> = serde_json::from_str(json).unwrap();
        let trouve = premier_etablissement(resultats).unwrap().unwrap();
        assert_eq!(
            trouve.adresse.as_deref(),
            Some("Rue des Comptoirs, 35410 Châteaugiron")
        );
    }

    #[test]
    fn sans_rue_pas_d_adresse_reformulee() {
        let adresse = AdresseOsm {
            town: Some("Châteaugiron".into()),
            ..Default::default()
        };
        assert_eq!(adresse.libelle(), None);
        let adresse = AdresseOsm {
            house_number: Some("3".into()),
            road: Some("Rue des Comptoirs".into()),
            ..Default::default()
        };
        assert_eq!(adresse.libelle().as_deref(), Some("3 Rue des Comptoirs"));
    }
}
