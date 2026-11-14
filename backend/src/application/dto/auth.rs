use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterMerchantRequest {
    pub nom: String,
    pub adresse: String,
    /// Code ISO du pays du commerce. Absent : France, comme les comptes créés
    /// avant l'ajout du champ. La position n'est plus fournie par le client :
    /// le backend la calcule depuis l'adresse et le pays (`Geocoder`).
    #[serde(default)]
    pub pays: Option<String>,
    pub categorie: String,
    pub email: String,
    pub password: String,
    /// Case cochée par le commerçant quand son commerce n'est pas trouvé :
    /// il l'ajoute manuellement. Voir `UpdateMerchantDto`.
    #[serde(default)]
    pub adresse_confirmee: bool,
    /// Version du texte de consentement bêta cochée à l'inscription. Champ
    /// obligatoire, comme pour un consommateur : nom, adresse et position
    /// d'un commerçant en personne physique sont des données personnelles,
    /// et elles sont publiées sur la carte.
    pub consent_version: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterConsumerRequest {
    pub email: String,
    pub password: String,
    /// Version du texte de consentement bêta cochée à l'inscription. Champ
    /// obligatoire : une requête sans lui est rejetée par la désérialisation,
    /// donc aucun compte ne peut être créé sans consentement explicite.
    pub consent_version: String,
}

/// JWT claims shared by marchand and consommateur tokens. `role` disambiguates
/// which principal `sub` refers to, since the two live in separate tables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub role: String,
    pub exp: usize,
}

/// Demande d'un lien de réinitialisation. Rien d'autre que l'adresse : la
/// réponse est identique que le compte existe ou non.
#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

/// Pose du nouveau mot de passe, avec le jeton reçu par email.
#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub password: String,
}
