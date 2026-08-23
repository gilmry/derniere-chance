use actix_web::{web, HttpResponse};

use crate::application::dto::LoginRequest;
use crate::application::use_cases::AdminAuthError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{internal_error, unauthorized};

pub async fn login(state: web::Data<AppState>, dto: web::Json<LoginRequest>) -> HttpResponse {
    let email = dto.email.clone();
    match state.admin_auth_use_cases.login(dto.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(AdminAuthError::InvalidCredentials) => {
            tracing::warn!(%email, "admin login failed: invalid credentials");
            unauthorized("invalid email or password")
        }
        Err(err) => {
            tracing::error!(?err, "admin login failed");
            internal_error()
        }
    }
}
