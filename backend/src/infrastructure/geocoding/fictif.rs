use async_trait::async_trait;

use crate::application::ports::{
    AddressSuggester, Coordonnees, Etablissement, GeocodeError, Geocoder,
};

/// Grand-Place de Bruxelles : toutes les adresses de dev et d'e2e y tombent.
pub const POSITION_FICTIVE: Coordonnees = Coordonnees {
    latitude: 50.8467,
    longitude: 4.3525,
};

/// Géocodeur sans réseau, pour le dev et les e2e : leurs adresses sont
/// fictives et n'ont rien à faire chez un vrai service. Toute adresse est
/// placée sur [`POSITION_FICTIVE`], sauf celles qui contiennent
/// « introuvable », qui permettent d'éprouver le message d'erreur. De même,
/// tout établissement est trouvé, sauf un nom contenant « introuvable ».
pub struct FictifGeocoder;

#[async_trait]
impl Geocoder for FictifGeocoder {
    async fn geocode(
        &self,
        adresse: &str,
        _pays: &str,
    ) -> Result<Option<Coordonnees>, GeocodeError> {
        if adresse.to_lowercase().contains("introuvable") {
            Ok(None)
        } else {
            Ok(Some(POSITION_FICTIVE))
        }
    }

    async fn trouver_etablissement(
        &self,
        nom: &str,
        _pays: &str,
        autour: Coordonnees,
    ) -> Result<Option<Etablissement>, GeocodeError> {
        // Sans adresse : celle que le commerçant a saisie est gardée.
        if nom.to_lowercase().contains("introuvable") {
            Ok(None)
        } else {
            Ok(Some(Etablissement {
                position: autour,
                adresse: None,
            }))
        }
    }
}

/// Suggestions sans réseau pour le dev et les e2e : deux adresses inventées
/// construites à partir de la saisie, pour éprouver la liste déroulante.
pub struct FictifSuggester;

#[async_trait]
impl AddressSuggester for FictifSuggester {
    async fn suggest(&self, saisie: &str, _pays: &str) -> Result<Vec<String>, GeocodeError> {
        let saisie = saisie.trim();
        Ok(vec![
            format!("{saisie}, 1000 Bruxelles"),
            format!("{saisie}, 4000 Liège"),
        ])
    }
}
