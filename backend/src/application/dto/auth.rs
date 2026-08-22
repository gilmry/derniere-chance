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
    pub categorie: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterConsumerRequest {
    pub email: String,
    pub password: String,
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
