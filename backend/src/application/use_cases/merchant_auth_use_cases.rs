use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use thiserror::Error;

use uuid::Uuid;

use crate::application::dto::{
    AuthResponse, Claims, LoginRequest, MerchantResponseDto, RegisterMerchantRequest,
    UpdateMerchantDto,
};
use crate::application::ports::{
    AddressSuggester, ConsentRepository, Coordonnees, Etablissement, EventNotifier, GeocodeError,
    Geocoder,
    MerchantRepository, MerchantUpdate, NewMerchant, RepoError,
};
use crate::application::use_cases::BETA_CONSENT_VERSION;
use crate::domain::entities::{ConsentSubject, Merchant};
use crate::domain::services::{password, pays};

#[derive(Debug, Error)]
pub enum MerchantAuthError {
    #[error("an account already exists for this email")]
    EmailTaken,
    #[error("invalid email or password")]
    InvalidCredentials,
    #[error("consent version is no longer current")]
    StaleConsentVersion,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("password hashing failed")]
    HashingFailed,
    #[error("merchant not found")]
    NotFound,
    /// L'adresse est inconnue dans ce pays : le commerçant peut la corriger,
    /// ou cocher la case pour ajouter son commerce manuellement.
    #[error("address not found")]
    AdresseIntrouvable,
    /// L'adresse est connue, mais aucun établissement de ce nom n'y figure :
    /// même issue possible, la case d'ajout manuel.
    #[error("establishment not found")]
    EtablissementIntrouvable,
    /// Le géocodeur ne répond pas : rien n'est enregistré, le commerçant
    /// réessaie plus tard (voir `GeocodeError::Indisponible`).
    #[error("geocoding service unavailable")]
    GeocodingUnavailable,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

const TOKEN_LIFETIME_HOURS: i64 = 24;
const ROLE: &str = "marchand";

pub struct MerchantAuthUseCases {
    merchant_repo: Arc<dyn MerchantRepository>,
    consent_repo: Arc<dyn ConsentRepository>,
    jwt_secret: String,
    event_notifier: Arc<dyn EventNotifier>,
    geocoder: Arc<dyn Geocoder>,
    suggester: Arc<dyn AddressSuggester>,
}

/// Message renvoyé quand l'adresse saisie est inconnue dans ce pays.
pub const ADRESSE_INTROUVABLE: &str = "adresse non trouvée dans ce pays : vérifiez-la (de \
     préférence en choisissant une suggestion), ou cochez la case pour ajouter votre commerce \
     manuellement";

/// Message renvoyé quand l'adresse est connue mais pas le commerce. Beaucoup
/// de petits commerces ne sont pas encore dans OpenStreetMap : le message le
/// dit, pour que le commerçant ne croie pas son adresse en cause.
pub const ETABLISSEMENT_INTROUVABLE: &str = "adresse trouvée, mais aucun commerce de ce nom n'y \
     est référencé (beaucoup de petits commerces ne le sont pas encore). Vérifiez le nom, ou \
     cochez la case pour ajouter votre commerce manuellement";

/// Longueurs acceptées pour une saisie à compléter : en dessous, trop de
/// résultats pour être utiles ; au-dessus, ce n'est plus une adresse.
const SAISIE_MIN: usize = 3;
const SAISIE_MAX: usize = 200;

/// Position et statut d'une adresse une fois résolue.
struct AdresseResolue {
    latitude: Option<f64>,
    longitude: Option<f64>,
    confirmee_le: Option<DateTime<Utc>>,
    /// Adresse complète de l'établissement trouvé, qui remplace la saisie
    /// (« Fournil des Saveurs Châteaugiron » devient « Rue des Comptoirs,
    /// 35410 Châteaugiron »). `None` : la saisie est gardée telle quelle.
    retenue: Option<String>,
}

/// Le service connaît souvent la rue d'un commerce sans son numéro : on
/// reprend alors celui que le commerçant a tapé, s'il a bien tapé cette rue.
fn completer_numero(retenue: String, saisie: &str) -> String {
    if retenue.starts_with(|c: char| c.is_ascii_digit()) {
        return retenue;
    }
    let Some(numero) = saisie
        .split_whitespace()
        .next()
        .filter(|mot| mot.starts_with(|c: char| c.is_ascii_digit()))
    else {
        return retenue;
    };
    let rue = retenue.split(',').next().unwrap_or("").trim().to_lowercase();
    if !rue.is_empty() && saisie.to_lowercase().contains(&rue) {
        format!("{} {retenue}", numero.trim_end_matches(','))
    } else {
        retenue
    }
}

/// Ce qui suit la première virgule : « 35410 Châteaugiron » pour
/// « 12 Rue Toute Neuve, 35410 Châteaugiron ». Sert à placer approximativement
/// un commerce dont la rue est inconnue.
fn localite(adresse: &str) -> Option<&str> {
    let (_, reste) = adresse.split_once(',')?;
    let reste = reste.trim();
    (!reste.is_empty()).then_some(reste)
}

impl MerchantAuthUseCases {
    pub fn new(
        merchant_repo: Arc<dyn MerchantRepository>,
        consent_repo: Arc<dyn ConsentRepository>,
        jwt_secret: String,
        event_notifier: Arc<dyn EventNotifier>,
        geocoder: Arc<dyn Geocoder>,
        suggester: Arc<dyn AddressSuggester>,
    ) -> Self {
        Self {
            merchant_repo,
            consent_repo,
            jwt_secret,
            event_notifier,
            geocoder,
            suggester,
        }
    }

    /// Adresses complètes proposées pendant la saisie. Une saisie trop courte
    /// ne déclenche aucune recherche.
    pub async fn suggerer_adresses(
        &self,
        saisie: &str,
        pays: &str,
    ) -> Result<Vec<String>, MerchantAuthError> {
        let pays = pays::normaliser(pays).map_err(MerchantAuthError::InvalidInput)?;
        let longueur = saisie.trim().chars().count();
        if !(SAISIE_MIN..=SAISIE_MAX).contains(&longueur) {
            return Ok(vec![]);
        }
        self.suggester
            .suggest(saisie, &pays)
            .await
            .map_err(|GeocodeError::Indisponible(cause)| {
                tracing::warn!(%cause, "suggestions d'adresses indisponibles");
                MerchantAuthError::GeocodingUnavailable
            })
    }

    /// Vérifie que le commerce existe : son adresse d'abord, puis un
    /// établissement de ce nom tout près. Trouvé : sa position. Sinon refusé,
    /// sauf si le commerçant a coché la case pour l'ajouter manuellement ; on
    /// garde alors la date de cet engagement et la meilleure position connue
    /// (l'adresse si elle existe, la ville sinon).
    async fn resoudre(
        &self,
        nom: &str,
        adresse: &str,
        pays: &str,
        confirmee: bool,
    ) -> Result<AdresseResolue, MerchantAuthError> {
        let point = match self.localiser(adresse, pays).await {
            Ok(point) => Some(point),
            Err(MerchantAuthError::AdresseIntrouvable) => None,
            Err(err) => return Err(err),
        };
        if let Some(point) = point {
            if let Some(etablissement) = self.chercher_etablissement(nom, pays, point).await? {
                return Ok(AdresseResolue {
                    latitude: Some(etablissement.position.latitude),
                    longitude: Some(etablissement.position.longitude),
                    confirmee_le: None,
                    retenue: etablissement
                        .adresse
                        .map(|retenue| completer_numero(retenue, adresse)),
                });
            }
        }
        if !confirmee {
            return Err(if point.is_some() {
                MerchantAuthError::EtablissementIntrouvable
            } else {
                MerchantAuthError::AdresseIntrouvable
            });
        }
        let position = match point {
            Some(point) => Some(point),
            None => match localite(adresse) {
                Some(localite) => self.geocoder.geocode(localite, pays).await.ok().flatten(),
                None => None,
            },
        };
        Ok(AdresseResolue {
            latitude: position.map(|c| c.latitude),
            longitude: position.map(|c| c.longitude),
            confirmee_le: Some(Utc::now()),
            retenue: None,
        })
    }

    async fn chercher_etablissement(
        &self,
        nom: &str,
        pays: &str,
        autour: Coordonnees,
    ) -> Result<Option<Etablissement>, MerchantAuthError> {
        self.geocoder
            .trouver_etablissement(nom, pays, autour)
            .await
            .map_err(|GeocodeError::Indisponible(cause)| {
                tracing::warn!(%cause, "recherche d'établissement impossible, rien n'est enregistré");
                MerchantAuthError::GeocodingUnavailable
            })
    }

    /// Position du commerce d'après son adresse et son pays : c'est elle, et
    /// non l'endroit où se trouvait le commerçant en remplissant le
    /// formulaire, qui doit apparaître sur la carte.
    async fn localiser(&self, adresse: &str, pays: &str) -> Result<Coordonnees, MerchantAuthError> {
        if adresse.trim().is_empty() {
            return Err(MerchantAuthError::InvalidInput(
                "l'adresse est obligatoire".into(),
            ));
        }
        match self.geocoder.geocode(adresse, pays).await {
            Ok(Some(coordonnees)) => Ok(coordonnees),
            Ok(None) => Err(MerchantAuthError::AdresseIntrouvable),
            Err(GeocodeError::Indisponible(cause)) => {
                tracing::warn!(%cause, "géocodage impossible, rien n'est enregistré");
                Err(MerchantAuthError::GeocodingUnavailable)
            }
        }
    }

    pub async fn register(
        &self,
        dto: RegisterMerchantRequest,
    ) -> Result<AuthResponse, MerchantAuthError> {
        // Vérifié avant toute écriture : inutile de créer un compte si la
        // case cochée portait sur un texte qui n'est plus en vigueur.
        if dto.consent_version != BETA_CONSENT_VERSION {
            return Err(MerchantAuthError::StaleConsentVersion);
        }

        // Même règle qu'à la réinitialisation : le compte ne doit pas pouvoir
        // naître avec un mot de passe que le parcours « mot de passe oublié »
        // refuserait ensuite.
        password::validate(&dto.password).map_err(MerchantAuthError::InvalidInput)?;

        let pays = pays::normaliser(dto.pays.as_deref().unwrap_or(pays::PAYS_PAR_DEFAUT))
            .map_err(MerchantAuthError::InvalidInput)?;

        if self
            .merchant_repo
            .find_by_email(&dto.email)
            .await?
            .is_some()
        {
            return Err(MerchantAuthError::EmailTaken);
        }

        // Avant le hachage, coûteux : un commerce non trouvé n'a pas à le payer.
        let adresse = self
            .resoudre(&dto.nom, &dto.adresse, &pays, dto.adresse_confirmee)
            .await?;

        let password_hash = bcrypt::hash(&dto.password, bcrypt::DEFAULT_COST)
            .map_err(|_| MerchantAuthError::HashingFailed)?;

        let merchant = self
            .merchant_repo
            .create(NewMerchant {
                nom: dto.nom,
                adresse: adresse.retenue.unwrap_or(dto.adresse),
                pays,
                categorie: dto.categorie,
                email: dto.email,
                password_hash,
                latitude: adresse.latitude,
                longitude: adresse.longitude,
                adresse_confirmee_le: adresse.confirmee_le,
            })
            .await?;

        // Enregistré avant la notification : annoncer une inscription qui
        // serait ensuite bloquée faute de consentement induirait en erreur.
        // Si cette écriture échoue, le compte existe sans consentement et le
        // portier `ConsentedMerchant` le bloque jusqu'à ce que le commerçant
        // consente depuis /pro/consentement : l'échec est fermé.
        self.consent_repo
            .grant(ConsentSubject::Merchant(merchant.id), BETA_CONSENT_VERSION)
            .await?;

        self.event_notifier
            .notify(
                "nouveau_marchand",
                format!(
                    "Nouveau marchand inscrit : {} ({}, {})",
                    merchant.nom, merchant.categorie, merchant.email
                ),
            )
            .await;

        let token = self.mint_token(&merchant, Duration::hours(TOKEN_LIFETIME_HOURS))?;
        Ok(AuthResponse { token })
    }

    pub async fn login(&self, dto: LoginRequest) -> Result<AuthResponse, MerchantAuthError> {
        let merchant = self
            .merchant_repo
            .find_by_email(&dto.email)
            .await?
            .ok_or(MerchantAuthError::InvalidCredentials)?;

        let valid = bcrypt::verify(&dto.password, &merchant.password_hash)
            .map_err(|_| MerchantAuthError::InvalidCredentials)?;
        if !valid {
            return Err(MerchantAuthError::InvalidCredentials);
        }

        let token = self.mint_token(&merchant, Duration::hours(TOKEN_LIFETIME_HOURS))?;
        Ok(AuthResponse { token })
    }

    /// Exposed (not just used internally) so `OAuthUseCases` can mint the
    /// exact same JWT shape for the MCP/OAuth flow, just with a shorter,
    /// refreshable lifetime instead of the 24h `/marchands/connexion` one -
    /// no REST handler needs to special-case a token that came from OAuth.
    pub fn mint_token(
        &self,
        merchant: &Merchant,
        lifetime: Duration,
    ) -> Result<String, MerchantAuthError> {
        let exp = (Utc::now() + lifetime).timestamp() as usize;
        let claims = Claims {
            sub: merchant.id,
            email: merchant.email.clone(),
            role: ROLE.to_string(),
            exp,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|_| MerchantAuthError::InvalidToken)
    }

    pub async fn update_logo(
        &self,
        marchand_id: Uuid,
        logo_url: &str,
    ) -> Result<Merchant, MerchantAuthError> {
        Ok(self.merchant_repo.update_logo(marchand_id, logo_url).await?)
    }

    pub async fn get_own_profile(
        &self,
        marchand_id: Uuid,
    ) -> Result<MerchantResponseDto, MerchantAuthError> {
        let merchant = self
            .merchant_repo
            .find_by_id(marchand_id)
            .await?
            .ok_or(MerchantAuthError::NotFound)?;
        Ok(merchant.into())
    }

    pub async fn update_profile(
        &self,
        marchand_id: Uuid,
        dto: UpdateMerchantDto,
    ) -> Result<MerchantResponseDto, MerchantAuthError> {
        let actuel = self
            .merchant_repo
            .find_by_id(marchand_id)
            .await?
            .ok_or(MerchantAuthError::NotFound)?;

        let pays = match dto.pays.as_deref() {
            Some(code) => pays::normaliser(code).map_err(MerchantAuthError::InvalidInput)?,
            None => actuel.pays.clone(),
        };

        // Revérifiée quand le nom, l'adresse ou le pays change, et aussi quand
        // il n'y a encore ni position ni confirmation. Sinon on garde ce qui
        // existe, sans rappeler le géocodeur pour un simple changement de
        // catégorie.
        let adresse_changee = dto.nom.trim() != actuel.nom.trim()
            || dto.adresse.trim() != actuel.adresse.trim()
            || pays != actuel.pays;
        let deja_resolue = actuel.latitude.is_some() || actuel.adresse_confirmee_le.is_some();
        let adresse = if !adresse_changee && deja_resolue {
            AdresseResolue {
                latitude: actuel.latitude,
                longitude: actuel.longitude,
                confirmee_le: actuel.adresse_confirmee_le,
                retenue: None,
            }
        } else {
            self.resoudre(&dto.nom, &dto.adresse, &pays, dto.adresse_confirmee)
                .await?
        };

        let merchant = self
            .merchant_repo
            .update(
                marchand_id,
                MerchantUpdate {
                    nom: dto.nom,
                    adresse: adresse.retenue.unwrap_or(dto.adresse),
                    pays,
                    categorie: dto.categorie,
                    latitude: adresse.latitude,
                    longitude: adresse.longitude,
                    adresse_confirmee_le: adresse.confirmee_le,
                },
            )
            .await
            .map_err(|err| match err {
                RepoError::NotFound => MerchantAuthError::NotFound,
                other => other.into(),
            })?;
        Ok(merchant.into())
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, MerchantAuthError> {
        let claims = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|_| MerchantAuthError::InvalidToken)?;

        if claims.role != ROLE {
            return Err(MerchantAuthError::InvalidToken);
        }
        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Mutex;

    use crate::domain::entities::BetaConsent;

    const RENNES: Coordonnees = Coordonnees {
        latitude: 48.1113,
        longitude: -1.6800,
    };
    const BRUXELLES: Coordonnees = Coordonnees {
        latitude: 50.8467,
        longitude: 4.3525,
    };

    /// Répond d'après l'adresse : « panne » simule un service tombé, une
    /// adresse qui contient « Inconnue » est introuvable, sinon « Rennes » et
    /// « Bruxelles » sont connues et le reste est introuvable. Garde la trace
    /// de chaque appel (adresse, pays). Tout établissement est trouvé sur le
    /// point de son adresse, sauf un nom qui contient « Inconnu » ; les noms
    /// cherchés sont gardés dans `recherches`.
    #[derive(Default)]
    struct FakeGeocoder {
        appels: Mutex<Vec<(String, String)>>,
        recherches: Mutex<Vec<String>>,
    }

    #[async_trait::async_trait]
    impl Geocoder for FakeGeocoder {
        async fn geocode(
            &self,
            adresse: &str,
            pays: &str,
        ) -> Result<Option<Coordonnees>, GeocodeError> {
            self.appels
                .lock()
                .unwrap()
                .push((adresse.to_string(), pays.to_string()));
            if adresse.contains("panne") {
                Err(GeocodeError::Indisponible("délai dépassé".into()))
            } else if adresse.contains("Inconnue") {
                Ok(None)
            } else if adresse.contains("Rennes") {
                Ok(Some(RENNES))
            } else if adresse.contains("Bruxelles") {
                Ok(Some(BRUXELLES))
            } else {
                Ok(None)
            }
        }

        async fn trouver_etablissement(
            &self,
            nom: &str,
            _pays: &str,
            autour: Coordonnees,
        ) -> Result<Option<Etablissement>, GeocodeError> {
            self.recherches.lock().unwrap().push(nom.to_string());
            if nom.contains("Inconnu") {
                return Ok(None);
            }
            // Seul « Fournil » a une adresse connue du service.
            let adresse = nom
                .contains("Fournil")
                .then(|| "Rue des Comptoirs, 35000 Rennes".to_string());
            Ok(Some(Etablissement {
                position: autour,
                adresse,
            }))
        }
    }

    #[derive(Default)]
    struct FakeMerchantRepo {
        rows: Mutex<Vec<Merchant>>,
    }

    #[async_trait::async_trait]
    impl MerchantRepository for FakeMerchantRepo {
        async fn create(&self, new: NewMerchant) -> Result<Merchant, RepoError> {
            let merchant = Merchant {
                id: Uuid::new_v4(),
                nom: new.nom,
                adresse: new.adresse,
                pays: new.pays,
                categorie: new.categorie,
                note: None,
                email: new.email,
                password_hash: new.password_hash,
                latitude: new.latitude,
                longitude: new.longitude,
                adresse_confirmee_le: new.adresse_confirmee_le,
                logo_url: None,
                created_at: Utc::now(),
                anonymise_le: None,
            };
            self.rows.lock().unwrap().push(merchant.clone());
            Ok(merchant)
        }
        async fn find_by_email(&self, email: &str) -> Result<Option<Merchant>, RepoError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|m| m.email == email)
                .cloned())
        }
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Merchant>, RepoError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|m| m.id == id)
                .cloned())
        }
        async fn list_all(&self) -> Result<Vec<Merchant>, RepoError> {
            Ok(self.rows.lock().unwrap().clone())
        }
        async fn delete(&self, _id: Uuid) -> Result<(), RepoError> {
            Ok(())
        }
        async fn anonymize(&self, _id: Uuid) -> Result<(), RepoError> {
            Ok(())
        }
        async fn update_password(&self, _id: Uuid, _hash: &str) -> Result<(), RepoError> {
            Ok(())
        }
        async fn count(&self) -> Result<i64, RepoError> {
            Ok(self.rows.lock().unwrap().len() as i64)
        }
        async fn update_logo(&self, _id: Uuid, _logo_url: &str) -> Result<Merchant, RepoError> {
            unimplemented!("hors du périmètre de ces tests")
        }
        async fn update(&self, id: Uuid, changes: MerchantUpdate) -> Result<Merchant, RepoError> {
            let mut rows = self.rows.lock().unwrap();
            let row = rows
                .iter_mut()
                .find(|m| m.id == id)
                .ok_or(RepoError::NotFound)?;
            row.nom = changes.nom;
            row.adresse = changes.adresse;
            row.pays = changes.pays;
            row.categorie = changes.categorie;
            row.latitude = changes.latitude;
            row.longitude = changes.longitude;
            row.adresse_confirmee_le = changes.adresse_confirmee_le;
            Ok(row.clone())
        }
    }

    struct FakeConsentRepo;

    #[async_trait::async_trait]
    impl ConsentRepository for FakeConsentRepo {
        async fn grant(
            &self,
            subject: ConsentSubject,
            version: &str,
        ) -> Result<BetaConsent, RepoError> {
            Ok(BetaConsent {
                id: Uuid::new_v4(),
                consommateur_id: None,
                marchand_id: Some(subject.id()),
                version: version.to_string(),
                accepte_le: Utc::now(),
                retire_le: None,
            })
        }
        async fn find_active(
            &self,
            _subject: ConsentSubject,
        ) -> Result<Option<BetaConsent>, RepoError> {
            Ok(None)
        }
        async fn withdraw(&self, _subject: ConsentSubject) -> Result<u64, RepoError> {
            Ok(0)
        }
    }

    struct FakeSuggester;

    #[async_trait::async_trait]
    impl AddressSuggester for FakeSuggester {
        async fn suggest(&self, saisie: &str, pays: &str) -> Result<Vec<String>, GeocodeError> {
            Ok(vec![format!("{saisie} ({pays})")])
        }
    }

    struct FakeNotifier;

    #[async_trait::async_trait]
    impl EventNotifier for FakeNotifier {
        async fn notify(&self, _event: &str, _message: String) {}
    }

    struct Banc {
        use_cases: MerchantAuthUseCases,
        repo: Arc<FakeMerchantRepo>,
        geocoder: Arc<FakeGeocoder>,
    }

    fn banc() -> Banc {
        let repo = Arc::new(FakeMerchantRepo::default());
        let geocoder = Arc::new(FakeGeocoder::default());
        let use_cases = MerchantAuthUseCases::new(
            repo.clone(),
            Arc::new(FakeConsentRepo),
            "secret-de-test".into(),
            Arc::new(FakeNotifier),
            geocoder.clone(),
            Arc::new(FakeSuggester),
        );
        Banc {
            use_cases,
            repo,
            geocoder,
        }
    }

    fn inscription(adresse: &str, pays: Option<&str>) -> RegisterMerchantRequest {
        RegisterMerchantRequest {
            nom: "Chez Léa".into(),
            adresse: adresse.into(),
            pays: pays.map(Into::into),
            categorie: "Boulangerie".into(),
            email: "lea@example.org".into(),
            password: "une phrase assez longue".into(),
            adresse_confirmee: false,
            consent_version: BETA_CONSENT_VERSION.into(),
        }
    }

    fn modification(adresse: &str, pays: Option<&str>) -> UpdateMerchantDto {
        UpdateMerchantDto {
            nom: "Chez Léa".into(),
            adresse: adresse.into(),
            pays: pays.map(Into::into),
            categorie: "Boulangerie".into(),
            adresse_confirmee: false,
        }
    }

    fn seul_marchand(repo: &FakeMerchantRepo) -> Merchant {
        let rows = repo.rows.lock().unwrap();
        assert_eq!(rows.len(), 1);
        rows[0].clone()
    }

    #[tokio::test]
    async fn l_inscription_place_le_commerce_a_son_adresse() {
        let b = banc();
        b.use_cases
            .register(inscription("4 Rue du Marché, 1000 Bruxelles", Some("be")))
            .await
            .unwrap();

        let m = seul_marchand(&b.repo);
        assert_eq!(m.pays, "BE");
        assert_eq!(
            (m.latitude, m.longitude),
            (Some(BRUXELLES.latitude), Some(BRUXELLES.longitude))
        );
        assert_eq!(
            *b.geocoder.appels.lock().unwrap(),
            vec![(
                "4 Rue du Marché, 1000 Bruxelles".to_string(),
                "BE".to_string()
            )]
        );
    }

    #[tokio::test]
    async fn sans_pays_l_inscription_vaut_pour_la_france() {
        let b = banc();
        b.use_cases
            .register(inscription("1 Place de la Mairie, 35000 Rennes", None))
            .await
            .unwrap();
        assert_eq!(seul_marchand(&b.repo).pays, "FR");
    }

    #[tokio::test]
    async fn une_adresse_introuvable_bloque_l_inscription() {
        let b = banc();
        let err = b
            .use_cases
            .register(inscription("Nulle part", Some("FR")))
            .await
            .unwrap_err();
        assert!(matches!(err, MerchantAuthError::AdresseIntrouvable));
        assert!(b.repo.rows.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn un_pays_hors_europe_est_refuse_sans_appeler_le_geocodeur() {
        let b = banc();
        let err = b
            .use_cases
            .register(inscription("1 Main Street, Rennes", Some("US")))
            .await
            .unwrap_err();
        assert!(matches!(err, MerchantAuthError::InvalidInput(_)));
        assert!(b.geocoder.appels.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn une_panne_du_geocodeur_n_enregistre_rien() {
        let b = banc();
        let err = b
            .use_cases
            .register(inscription("panne", Some("FR")))
            .await
            .unwrap_err();
        assert!(matches!(err, MerchantAuthError::GeocodingUnavailable));
        assert!(b.repo.rows.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn changer_d_adresse_deplace_le_commerce() {
        let b = banc();
        b.use_cases
            .register(inscription("4 Rue du Marché, 1000 Bruxelles", Some("BE")))
            .await
            .unwrap();
        let id = seul_marchand(&b.repo).id;

        let profil = b
            .use_cases
            .update_profile(
                id,
                modification("1 Place de la Mairie, 35000 Rennes", Some("FR")),
            )
            .await
            .unwrap();

        assert_eq!(profil.pays, "FR");
        assert_eq!(
            (profil.latitude, profil.longitude),
            (Some(RENNES.latitude), Some(RENNES.longitude))
        );
        let m = seul_marchand(&b.repo);
        assert_eq!(
            (m.latitude, m.longitude),
            (Some(RENNES.latitude), Some(RENNES.longitude))
        );
    }

    #[tokio::test]
    async fn sans_changement_d_adresse_le_geocodeur_n_est_pas_rappele() {
        let b = banc();
        b.use_cases
            .register(inscription("4 Rue du Marché, 1000 Bruxelles", Some("BE")))
            .await
            .unwrap();
        let id = seul_marchand(&b.repo).id;

        // Pays absent : celui du compte est gardé.
        b.use_cases
            .update_profile(id, modification("4 Rue du Marché, 1000 Bruxelles", None))
            .await
            .unwrap();

        assert_eq!(b.geocoder.appels.lock().unwrap().len(), 1);
        assert_eq!(seul_marchand(&b.repo).pays, "BE");
    }

    #[tokio::test]
    async fn un_commerce_sans_position_en_recoit_une_a_la_modification() {
        let b = banc();
        b.use_cases
            .register(inscription("1 Place de la Mairie, 35000 Rennes", None))
            .await
            .unwrap();
        let id = {
            // Comme un compte créé avant le géocodage, quand le navigateur
            // avait refusé de donner sa position.
            let mut rows = b.repo.rows.lock().unwrap();
            rows[0].latitude = None;
            rows[0].longitude = None;
            rows[0].id
        };

        b.use_cases
            .update_profile(id, modification("1 Place de la Mairie, 35000 Rennes", None))
            .await
            .unwrap();

        assert_eq!(seul_marchand(&b.repo).latitude, Some(RENNES.latitude));
    }

    #[tokio::test]
    async fn une_nouvelle_adresse_introuvable_laisse_le_profil_intact() {
        let b = banc();
        b.use_cases
            .register(inscription("4 Rue du Marché, 1000 Bruxelles", Some("BE")))
            .await
            .unwrap();
        let avant = seul_marchand(&b.repo);

        let err = b
            .use_cases
            .update_profile(avant.id, modification("Nulle part", None))
            .await
            .unwrap_err();

        assert!(matches!(err, MerchantAuthError::AdresseIntrouvable));
        let apres = seul_marchand(&b.repo);
        assert_eq!(apres.adresse, avant.adresse);
        assert_eq!(apres.latitude, avant.latitude);
    }

    fn appels(b: &Banc) -> Vec<String> {
        b.geocoder
            .appels
            .lock()
            .unwrap()
            .iter()
            .map(|(adresse, _)| adresse.clone())
            .collect()
    }

    #[tokio::test]
    async fn une_adresse_introuvable_confirmee_est_acceptee_et_placee_dans_sa_ville() {
        let b = banc();
        let mut dto = inscription("12 Rue Inconnue, 35000 Rennes", Some("FR"));
        dto.adresse_confirmee = true;
        b.use_cases.register(dto).await.unwrap();

        let m = seul_marchand(&b.repo);
        assert!(m.adresse_confirmee_le.is_some());
        assert_eq!(m.adresse, "12 Rue Inconnue, 35000 Rennes");
        assert_eq!(m.latitude, Some(RENNES.latitude));
        assert_eq!(
            appels(&b),
            vec!["12 Rue Inconnue, 35000 Rennes", "35000 Rennes"]
        );
    }

    #[tokio::test]
    async fn une_adresse_introuvable_confirmee_sans_ville_connue_reste_sans_position() {
        let b = banc();
        let mut dto = inscription("12 Rue Inconnue, Zzzville", Some("FR"));
        dto.adresse_confirmee = true;
        b.use_cases.register(dto).await.unwrap();

        let m = seul_marchand(&b.repo);
        assert!(m.adresse_confirmee_le.is_some());
        assert_eq!((m.latitude, m.longitude), (None, None));
    }

    #[tokio::test]
    async fn sans_virgule_pas_de_recherche_de_ville() {
        let b = banc();
        let mut dto = inscription("12 Rue Inconnue Rennes", Some("FR"));
        dto.adresse_confirmee = true;
        b.use_cases.register(dto).await.unwrap();

        assert_eq!(seul_marchand(&b.repo).latitude, None);
        assert_eq!(appels(&b).len(), 1);
    }

    #[tokio::test]
    async fn une_adresse_trouvee_n_est_pas_marquee_confirmee_meme_case_cochee() {
        let b = banc();
        let mut dto = inscription("1 Place de la Mairie, 35000 Rennes", None);
        dto.adresse_confirmee = true;
        b.use_cases.register(dto).await.unwrap();

        let m = seul_marchand(&b.repo);
        assert!(m.adresse_confirmee_le.is_none());
        assert_eq!(m.latitude, Some(RENNES.latitude));
    }

    #[tokio::test]
    async fn la_case_ne_contourne_pas_une_panne_du_geocodeur() {
        let b = banc();
        let mut dto = inscription("panne", Some("FR"));
        dto.adresse_confirmee = true;
        let err = b.use_cases.register(dto).await.unwrap_err();
        assert!(matches!(err, MerchantAuthError::GeocodingUnavailable));
        assert!(b.repo.rows.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn une_confirmation_survit_a_un_changement_de_categorie() {
        let b = banc();
        let mut dto = inscription("12 Rue Inconnue, Zzzville", Some("FR"));
        dto.adresse_confirmee = true;
        b.use_cases.register(dto).await.unwrap();
        let avant = seul_marchand(&b.repo);

        // Case non renvoyée : nom et adresse inchangés, rien n'est recalculé.
        let mut changement = modification("12 Rue Inconnue, Zzzville", None);
        changement.categorie = "Primeur".into();
        b.use_cases
            .update_profile(avant.id, changement)
            .await
            .unwrap();

        let apres = seul_marchand(&b.repo);
        assert_eq!(apres.categorie, "Primeur");
        assert_eq!(apres.adresse_confirmee_le, avant.adresse_confirmee_le);
        assert_eq!(appels(&b).len(), 2, "pas de nouvel appel au géocodeur");
    }

    #[tokio::test]
    async fn l_inscription_cherche_l_etablissement_a_son_adresse() {
        let b = banc();
        b.use_cases
            .register(inscription("1 Place de la Mairie, 35000 Rennes", None))
            .await
            .unwrap();
        assert_eq!(*b.geocoder.recherches.lock().unwrap(), vec!["Chez Léa"]);
        assert!(seul_marchand(&b.repo).adresse_confirmee_le.is_none());
    }

    #[tokio::test]
    async fn l_adresse_de_l_etablissement_trouve_remplace_la_saisie() {
        let b = banc();
        let mut dto = inscription("Fournil des saveurs Rennes", None);
        dto.nom = "Le Fournil des Saveurs".into();
        b.use_cases.register(dto).await.unwrap();
        assert_eq!(
            seul_marchand(&b.repo).adresse,
            "Rue des Comptoirs, 35000 Rennes"
        );
    }

    #[test]
    fn le_numero_tape_complete_une_adresse_sans_numero() {
        let retenue = || "Rue des Comptoirs, 35410 Châteaugiron".to_string();
        assert_eq!(
            completer_numero(retenue(), "3 rue des comptoirs, Châteaugiron"),
            "3 Rue des Comptoirs, 35410 Châteaugiron"
        );
        // Autre rue tapée : le numéro ne vaut pas pour celle-ci.
        assert_eq!(
            completer_numero(retenue(), "3 Rue de Rennes, Châteaugiron"),
            retenue()
        );
        assert_eq!(
            completer_numero(
                "8 Rue des Comptoirs, 35410 Châteaugiron".into(),
                "3 rue des comptoirs"
            ),
            "8 Rue des Comptoirs, 35410 Châteaugiron"
        );
    }

    #[tokio::test]
    async fn un_etablissement_inconnu_a_une_adresse_connue_est_refuse() {
        let b = banc();
        let mut dto = inscription("1 Place de la Mairie, 35000 Rennes", None);
        dto.nom = "Commerce Inconnu".into();
        let err = b.use_cases.register(dto).await.unwrap_err();
        assert!(matches!(err, MerchantAuthError::EtablissementIntrouvable));
        assert!(b.repo.rows.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn un_etablissement_inconnu_ajoute_manuellement_est_place_a_son_adresse() {
        let b = banc();
        let mut dto = inscription("1 Place de la Mairie, 35000 Rennes", None);
        dto.nom = "Commerce Inconnu".into();
        dto.adresse_confirmee = true;
        b.use_cases.register(dto).await.unwrap();

        let m = seul_marchand(&b.repo);
        assert!(m.adresse_confirmee_le.is_some());
        assert_eq!(
            (m.latitude, m.longitude),
            (Some(RENNES.latitude), Some(RENNES.longitude))
        );
    }

    #[tokio::test]
    async fn changer_de_nom_reverifie_l_etablissement() {
        let b = banc();
        b.use_cases
            .register(inscription("1 Place de la Mairie, 35000 Rennes", None))
            .await
            .unwrap();
        let id = seul_marchand(&b.repo).id;

        let mut changement = modification("1 Place de la Mairie, 35000 Rennes", None);
        changement.nom = "Nouveau Nom Inconnu".into();
        let err = b
            .use_cases
            .update_profile(id, changement)
            .await
            .unwrap_err();
        assert!(matches!(err, MerchantAuthError::EtablissementIntrouvable));
        assert_eq!(seul_marchand(&b.repo).nom, "Chez Léa");
    }

    #[tokio::test]
    async fn une_nouvelle_adresse_trouvee_efface_la_confirmation() {
        let b = banc();
        let mut dto = inscription("12 Rue Inconnue, Zzzville", Some("FR"));
        dto.adresse_confirmee = true;
        b.use_cases.register(dto).await.unwrap();
        let id = seul_marchand(&b.repo).id;

        let profil = b
            .use_cases
            .update_profile(id, modification("1 Place de la Mairie, 35000 Rennes", None))
            .await
            .unwrap();

        assert!(profil.adresse_verifiee);
        assert!(seul_marchand(&b.repo).adresse_confirmee_le.is_none());
    }

    #[tokio::test]
    async fn une_nouvelle_adresse_introuvable_demande_la_case() {
        let b = banc();
        b.use_cases
            .register(inscription("1 Place de la Mairie, 35000 Rennes", None))
            .await
            .unwrap();
        let id = seul_marchand(&b.repo).id;

        let err = b
            .use_cases
            .update_profile(id, modification("12 Rue Inconnue, 35000 Rennes", None))
            .await
            .unwrap_err();
        assert!(matches!(err, MerchantAuthError::AdresseIntrouvable));

        let mut dto = modification("12 Rue Inconnue, 35000 Rennes", None);
        dto.adresse_confirmee = true;
        let profil = b.use_cases.update_profile(id, dto).await.unwrap();
        assert!(!profil.adresse_verifiee);
    }

    #[tokio::test]
    async fn les_suggestions_demandent_trois_caracteres_et_un_pays_europeen() {
        let b = banc();
        assert!(b
            .use_cases
            .suggerer_adresses(" 12", "FR")
            .await
            .unwrap()
            .is_empty());
        assert!(matches!(
            b.use_cases.suggerer_adresses("12 rue", "US").await,
            Err(MerchantAuthError::InvalidInput(_))
        ));
        assert_eq!(
            b.use_cases.suggerer_adresses("12 rue", "be").await.unwrap(),
            vec!["12 rue (BE)".to_string()]
        );
    }
}
