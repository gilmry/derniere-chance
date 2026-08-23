use actix_web::{web, HttpResponse};

use crate::application::dto::{LoginRequest, RegisterMerchantRequest};
use crate::application::use_cases::MerchantAuthError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{
    conflict, internal_error, not_found, unauthorized,
};
use crate::infrastructure::web::middleware::AuthenticatedMerchant;

pub async fn register(
    state: web::Data<AppState>,
    dto: web::Json<RegisterMerchantRequest>,
) -> HttpResponse {
    match state.merchant_auth_use_cases.register(dto.into_inner()).await {
        Ok(response) => HttpResponse::Created().json(response),
        Err(MerchantAuthError::EmailTaken) => conflict("an account already exists for this email"),
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

pub async fn me(state: web::Data<AppState>, merchant: AuthenticatedMerchant) -> HttpResponse {
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
