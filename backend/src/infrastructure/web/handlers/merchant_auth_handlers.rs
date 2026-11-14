use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;

use crate::application::dto::{LoginRequest, RegisterMerchantRequest, UpdateMerchantDto};
use crate::application::use_cases::{
    MerchantAuthError, ADRESSE_INTROUVABLE, ETABLISSEMENT_INTROUVABLE,
};
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{
    bad_request, conflict, internal_error, not_found, service_unavailable, too_many_requests,
    unauthorized, unprocessable,
};
use crate::infrastructure::web::limiteur;

const TROP_DE_DEMANDES: &str = "trop de demandes depuis cette adresse, réessayez plus tard";

/// Le géocodeur ne répond pas : rien n'a été enregistré.
const GEOCODAGE_INDISPONIBLE: &str =
    "la recherche d'adresse ne répond pas, réessayez dans quelques minutes";
use crate::infrastructure::web::middleware::ConsentedMerchant;

pub async fn register(
    req: HttpRequest,
    state: web::Data<AppState>,
    dto: web::Json<RegisterMerchantRequest>,
) -> HttpResponse {
    if !limiteur::inscriptions().autoriser(&limiteur::cle_client(&req)) {
        return too_many_requests(TROP_DE_DEMANDES);
    }
    match state
        .merchant_auth_use_cases
        .register(dto.into_inner())
        .await
    {
        Ok(response) => HttpResponse::Created().json(response),
        Err(MerchantAuthError::EmailTaken) => conflict("an account already exists for this email"),
        Err(MerchantAuthError::StaleConsentVersion) => conflict(
            "la politique de confidentialité a changé, recharge la page avant de t'inscrire",
        ),
        Err(MerchantAuthError::InvalidInput(message)) => bad_request(&message),
        Err(MerchantAuthError::GeocodingUnavailable) => service_unavailable(GEOCODAGE_INDISPONIBLE),
        Err(MerchantAuthError::AdresseIntrouvable) => {
            unprocessable("adresse_introuvable", ADRESSE_INTROUVABLE)
        }
        Err(MerchantAuthError::EtablissementIntrouvable) => {
            unprocessable("etablissement_introuvable", ETABLISSEMENT_INTROUVABLE)
        }
        Err(err) => {
            tracing::error!(?err, "merchant register failed");
            internal_error()
        }
    }
}

pub async fn login(state: web::Data<AppState>, dto: web::Json<LoginRequest>) -> HttpResponse {
    let email = dto.email.clone();
    match state.merchant_auth_use_cases.login(dto.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(MerchantAuthError::InvalidCredentials) => {
            tracing::warn!(%email, "merchant login failed: invalid credentials");
            unauthorized("invalid email or password")
        }
        Err(err) => {
            tracing::error!(?err, "merchant login failed");
            internal_error()
        }
    }
}

pub async fn me(state: web::Data<AppState>, merchant: ConsentedMerchant) -> HttpResponse {
    match state
        .merchant_auth_use_cases
        .get_own_profile(merchant.marchand_id)
        .await
    {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(MerchantAuthError::NotFound) => not_found("merchant not found"),
        Err(err) => {
            tracing::error!(?err, "merchant profile fetch failed");
            internal_error()
        }
    }
}

pub async fn update_me(
    state: web::Data<AppState>,
    merchant: ConsentedMerchant,
    dto: web::Json<UpdateMerchantDto>,
) -> HttpResponse {
    match state
        .merchant_auth_use_cases
        .update_profile(merchant.marchand_id, dto.into_inner())
        .await
    {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(MerchantAuthError::NotFound) => not_found("merchant not found"),
        Err(MerchantAuthError::InvalidInput(message)) => bad_request(&message),
        Err(MerchantAuthError::GeocodingUnavailable) => service_unavailable(GEOCODAGE_INDISPONIBLE),
        Err(MerchantAuthError::AdresseIntrouvable) => {
            unprocessable("adresse_introuvable", ADRESSE_INTROUVABLE)
        }
        Err(MerchantAuthError::EtablissementIntrouvable) => {
            unprocessable("etablissement_introuvable", ETABLISSEMENT_INTROUVABLE)
        }
        Err(err) => {
            tracing::error!(?err, "merchant profile update failed");
            internal_error()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SuggestionsQuery {
    q: String,
    pays: String,
}

/// `GET /adresses/suggestions?q=12 rue de la&pays=FR` → `["12 Rue de la Gare, ..."]`.
pub async fn suggest_addresses(
    req: HttpRequest,
    state: web::Data<AppState>,
    query: web::Query<SuggestionsQuery>,
) -> HttpResponse {
    if !limiteur::suggestions().autoriser(&limiteur::cle_client(&req)) {
        return too_many_requests(TROP_DE_DEMANDES);
    }
    match state
        .merchant_auth_use_cases
        .suggerer_adresses(&query.q, &query.pays)
        .await
    {
        Ok(suggestions) => HttpResponse::Ok().json(suggestions),
        Err(MerchantAuthError::InvalidInput(message)) => bad_request(&message),
        Err(MerchantAuthError::GeocodingUnavailable) => service_unavailable(GEOCODAGE_INDISPONIBLE),
        Err(err) => {
            tracing::error!(?err, "address suggestions failed");
            internal_error()
        }
    }
}
