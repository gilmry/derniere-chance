use actix_web::{web, HttpResponse};

use crate::application::dto::GrantConsentRequest;
use crate::application::use_cases::ConsentError;
use crate::domain::entities::ConsentSubject;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{conflict, internal_error};
use crate::infrastructure::web::middleware::{AuthenticatedConsumer, AuthenticatedMerchant};

// Ces trois handlers sont volontairement sur `Authenticated*` et non sur les
// portiers `Consented*` : il faut bien pouvoir consentir, ou se rétracter,
// sans avoir déjà consenti.

async fn status(state: &AppState, subject: ConsentSubject) -> HttpResponse {
    match state.consent_use_cases.status(subject).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(err) => {
            tracing::error!(?err, role = subject.role(), "consent status failed");
            internal_error()
        }
    }
}

async fn grant(state: &AppState, subject: ConsentSubject, version: &str) -> HttpResponse {
    match state.consent_use_cases.grant(subject, version).await {
        Ok(status) => HttpResponse::Ok().json(status),
        Err(ConsentError::StaleVersion) => conflict(
            "la politique de confidentialité a changé, recharge la page avant d'accepter",
        ),
        Err(err) => {
            tracing::error!(?err, role = subject.role(), "consent grant failed");
            internal_error()
        }
    }
}

/// Retire le consentement et anonymise le compte dans la foulée. Le jeton
/// éventuellement encore en circulation devient inutilisable : le portier
/// bloque, et la reconnexion est impossible puisque le hash du mot de passe
/// a été vidé.
async fn withdraw(state: &AppState, subject: ConsentSubject) -> HttpResponse {
    match state.consent_use_cases.withdraw(subject).await {
        Ok(()) => {
            tracing::info!(
                id = %subject.id(),
                role = subject.role(),
                "consentement bêta retiré, compte anonymisé"
            );
            HttpResponse::NoContent().finish()
        }
        Err(err) => {
            tracing::error!(?err, role = subject.role(), "consent withdraw failed");
            internal_error()
        }
    }
}

// --- Consommateur ---

pub async fn consumer_status(
    state: web::Data<AppState>,
    consumer: AuthenticatedConsumer,
) -> HttpResponse {
    status(&state, ConsentSubject::Consumer(consumer.consommateur_id)).await
}

pub async fn consumer_grant(
    state: web::Data<AppState>,
    consumer: AuthenticatedConsumer,
    dto: web::Json<GrantConsentRequest>,
) -> HttpResponse {
    grant(
        &state,
        ConsentSubject::Consumer(consumer.consommateur_id),
        &dto.consent_version,
    )
    .await
}

pub async fn consumer_withdraw(
    state: web::Data<AppState>,
    consumer: AuthenticatedConsumer,
) -> HttpResponse {
    withdraw(&state, ConsentSubject::Consumer(consumer.consommateur_id)).await
}

// --- Marchand ---

pub async fn merchant_status(
    state: web::Data<AppState>,
    merchant: AuthenticatedMerchant,
) -> HttpResponse {
    status(&state, ConsentSubject::Merchant(merchant.marchand_id)).await
}

pub async fn merchant_grant(
    state: web::Data<AppState>,
    merchant: AuthenticatedMerchant,
    dto: web::Json<GrantConsentRequest>,
) -> HttpResponse {
    grant(
        &state,
        ConsentSubject::Merchant(merchant.marchand_id),
        &dto.consent_version,
    )
    .await
}

pub async fn merchant_withdraw(
    state: web::Data<AppState>,
    merchant: AuthenticatedMerchant,
) -> HttpResponse {
    withdraw(&state, ConsentSubject::Merchant(merchant.marchand_id)).await
}
