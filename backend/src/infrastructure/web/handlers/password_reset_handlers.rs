use actix_web::{web, HttpResponse};

use crate::application::dto::{ForgotPasswordRequest, ResetPasswordRequest};
use crate::application::use_cases::PasswordResetError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{bad_request, internal_error};

/// Toujours 204, y compris pour une adresse inconnue : une réponse
/// distinguable ferait de ce point d'entrée un moyen de savoir qui est
/// inscrit. Non authentifié, par construction.
pub async fn forgot(
    state: web::Data<AppState>,
    dto: web::Json<ForgotPasswordRequest>,
) -> HttpResponse {
    match state
        .password_reset_use_cases
        .request(&dto.into_inner().email)
        .await
    {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(err) => {
            tracing::error!(?err, "demande de réinitialisation échouée");
            internal_error()
        }
    }
}

/// 400 sur un jeton invalide comme sur un mot de passe refusé : rien à
/// dissimuler ici, la personne détient déjà le lien.
pub async fn reset(
    state: web::Data<AppState>,
    dto: web::Json<ResetPasswordRequest>,
) -> HttpResponse {
    let dto = dto.into_inner();
    match state
        .password_reset_use_cases
        .confirm(&dto.token, &dto.password)
        .await
    {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(PasswordResetError::InvalidToken) => bad_request(
            "ce lien de réinitialisation est invalide ou a expiré, demandez-en un nouveau",
        ),
        Err(PasswordResetError::InvalidInput(message)) => bad_request(&message),
        Err(err) => {
            tracing::error!(?err, "réinitialisation échouée");
            internal_error()
        }
    }
}
