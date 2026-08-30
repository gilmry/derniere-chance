use actix_web::{web, HttpResponse};

use crate::application::dto::{LoginRequest, RegisterConsumerRequest};
use crate::application::use_cases::ConsumerAuthError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{conflict, internal_error, unauthorized};

pub async fn register(
    state: web::Data<AppState>,
    dto: web::Json<RegisterConsumerRequest>,
) -> HttpResponse {
    match state.consumer_auth_use_cases.register(dto.into_inner()).await {
        Ok(response) => HttpResponse::Created().json(response),
        Err(ConsumerAuthError::EmailTaken) => conflict("an account already exists for this email"),
        Err(ConsumerAuthError::StaleConsentVersion) => conflict(
            "la politique de confidentialité a changé, recharge la page avant de t'inscrire",
        ),
        Err(err) => {
            tracing::error!(?err, "consumer register failed");
            internal_error()
        }
    }
}

pub async fn login(state: web::Data<AppState>, dto: web::Json<LoginRequest>) -> HttpResponse {
    let email = dto.email.clone();
    match state.consumer_auth_use_cases.login(dto.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(ConsumerAuthError::InvalidCredentials) => {
            tracing::warn!(%email, "consumer login failed: invalid credentials");
            unauthorized("invalid email or password")
        }
        Err(err) => {
            tracing::error!(?err, "consumer login failed");
            internal_error()
        }
    }
}
