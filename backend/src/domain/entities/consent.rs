use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// À qui se rapporte un consentement. Marchands et consommateurs sont deux
/// principaux distincts, avec leurs propres tables et leurs propres jetons ;
/// ce type évite d'avoir à dupliquer tout le circuit du consentement pour
/// chacun d'eux.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsentSubject {
    Consumer(Uuid),
    Merchant(Uuid),
}

impl ConsentSubject {
    /// Libellé du rôle, pour les journaux et les messages d'erreur.
    pub fn role(&self) -> &'static str {
        match self {
            Self::Consumer(_) => "consommateur",
            Self::Merchant(_) => "marchand",
        }
    }

    pub fn id(&self) -> Uuid {
        match self {
            Self::Consumer(id) | Self::Merchant(id) => *id,
        }
    }
}

/// Trace d'un consentement au programme bêta. `version` est la version du
/// texte effectivement affiché au moment de l'acceptation : c'est elle qui
/// permet de démontrer *à quoi* la personne a consenti (RGPD art. 7 §1), pas
/// seulement qu'elle a coché une case.
///
/// Exactement l'un de `consommateur_id` / `marchand_id` est renseigné,
/// contrainte tenue par la base (`consentement_un_seul_sujet`).
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct BetaConsent {
    pub id: Uuid,
    pub consommateur_id: Option<Uuid>,
    pub marchand_id: Option<Uuid>,
    pub version: String,
    pub accepte_le: DateTime<Utc>,
    /// `None` tant que le consentement est actif.
    pub retire_le: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_role_distingue_les_deux_principaux() {
        let id = Uuid::nil();
        assert_eq!(ConsentSubject::Consumer(id).role(), "consommateur");
        assert_eq!(ConsentSubject::Merchant(id).role(), "marchand");
    }

    #[test]
    fn deux_sujets_de_meme_id_ne_sont_pas_le_meme_sujet() {
        // Un marchand et un consommateur pourraient théoriquement porter le
        // même UUID : le portier ne doit jamais confondre leurs consentements.
        let id = Uuid::from_u128(42);
        assert_ne!(ConsentSubject::Consumer(id), ConsentSubject::Merchant(id));
    }

    #[test]
    fn l_identifiant_est_conserve_quel_que_soit_le_role() {
        let id = Uuid::from_u128(7);
        assert_eq!(ConsentSubject::Consumer(id).id(), id);
        assert_eq!(ConsentSubject::Merchant(id).id(), id);
    }
}
