use actix_web::{web, HttpResponse};

use crate::application::dto::GrantConsentRequest;
use crate::application::use_cases::ConsentError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{conflict, internal_error};
use crate::infrastructure::web::middleware::AuthenticatedConsumer;

/// Où en est le consentement de la personne connectée. Sert au portier
/// côté navigateur (voir `frontend/src/lib/consent.ts`) ainsi qu'à
/// l'affichage de la page /consentement.
pub async fn status(state: web::Data<AppState>, consumer: AuthenticatedConsumer) -> HttpResponse {
    match state.consent_use_cases.status(consumer.consommateur_id).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(err) => {
            tracing::error!(?err, "consent status failed");
            internal_error()
        }
    }
}

/// Donne (ou renouvelle après changement de version) le consentement.
pub async fn grant(
    state: web::Data<AppState>,
    consumer: AuthenticatedConsumer,
    dto: web::Json<GrantConsentRequest>,
) -> HttpResponse {
    match state
        .consent_use_cases
        .grant(consumer.consommateur_id, &dto.consent_version)
        .await
    {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(ConsentError::StaleVersion) => conflict(
            "la politique de confidentialité a changé, recharge la page avant d'accepter",
        ),
        Err(err) => {
            tracing::error!(?err, "consent grant failed");
            internal_error()
        }
    }
}

/// Retire le consentement et anonymise le compte dans la foulée. Le jeton
/// éventuellement encore en circulation devient inutilisable : le portier
/// `ConsentedConsumer` bloque, et la reconnexion est impossible puisque le
/// hash du mot de passe a été vidé.
pub async fn withdraw(state: web::Data<AppState>, consumer: AuthenticatedConsumer) -> HttpResponse {
    match state
        .consent_use_cases
        .withdraw(consumer.consommateur_id)
        .await
    {
        Ok(()) => {
            tracing::info!(
                consommateur_id = %consumer.consommateur_id,
                "consentement bêta retiré, compte anonymisé"
            );
            HttpResponse::NoContent().finish()
        }
        Err(err) => {
            tracing::error!(?err, "consent withdraw failed");
            internal_error()
        }
    }
}
