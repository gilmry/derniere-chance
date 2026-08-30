use actix_web::{
    dev::Payload,
    error::{ErrorInternalServerError, ErrorUnauthorized},
    http::StatusCode,
    web, Error, FromRequest, HttpRequest, HttpResponse, ResponseError,
};
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use thiserror::Error as ThisError;
use uuid::Uuid;

use crate::domain::entities::ConsentSubject;
use crate::infrastructure::web::app_state::AppState;

fn bearer_token(req: &HttpRequest) -> Result<&str, Error> {
    let header = req
        .headers()
        .get("Authorization")
        .ok_or_else(|| ErrorUnauthorized("missing authorization header"))?
        .to_str()
        .map_err(|_| ErrorUnauthorized("invalid authorization header"))?;
    Ok(header.trim_start_matches("Bearer ").trim())
}

/// Marchand identity extracted and verified from the JWT `Authorization:
/// Bearer` header. Rejects tokens minted for a consommateur (see
/// `MerchantAuthUseCases::verify_token`'s role check), so a consommateur
/// token can never authenticate backoffice endpoints even if replayed there.
#[derive(Debug, Clone)]
pub struct AuthenticatedMerchant {
    pub marchand_id: Uuid,
    pub email: String,
}

fn verify_merchant(req: &HttpRequest) -> Result<AuthenticatedMerchant, Error> {
    let app_state = req
        .app_data::<web::Data<AppState>>()
        .ok_or_else(|| ErrorUnauthorized("internal server error"))?;
    let token = bearer_token(req)?;

    let claims = app_state
        .merchant_auth_use_cases
        .verify_token(token)
        .map_err(|_| ErrorUnauthorized("invalid or expired token"))?;

    Ok(AuthenticatedMerchant {
        marchand_id: claims.sub,
        email: claims.email,
    })
}

impl FromRequest for AuthenticatedMerchant {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(verify_merchant(req))
    }
}

/// Consommateur identity extracted and verified from the JWT `Authorization:
/// Bearer` header.
#[derive(Debug, Clone)]
pub struct AuthenticatedConsumer {
    pub consommateur_id: Uuid,
    pub email: String,
}

fn verify_consumer(req: &HttpRequest) -> Result<AuthenticatedConsumer, Error> {
    let app_state = req
        .app_data::<web::Data<AppState>>()
        .ok_or_else(|| ErrorUnauthorized("internal server error"))?;
    let token = bearer_token(req)?;

    let claims = app_state
        .consumer_auth_use_cases
        .verify_token(token)
        .map_err(|_| ErrorUnauthorized("invalid or expired token"))?;

    Ok(AuthenticatedConsumer {
        consommateur_id: claims.sub,
        email: claims.email,
    })
}

impl FromRequest for AuthenticatedConsumer {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(verify_consumer(req))
    }
}

/// 403 renvoyé par `ConsentedConsumer`. Le corps porte un code stable pour
/// que le frontend distingue « il manque le consentement » d'un refus
/// ordinaire et renvoie vers /consentement au lieu d'afficher une erreur.
#[derive(Debug, ThisError)]
#[error("consentement au programme bêta requis")]
struct ConsentRequired;

impl ResponseError for ConsentRequired {
    fn status_code(&self) -> StatusCode {
        StatusCode::FORBIDDEN
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::Forbidden().json(serde_json::json!({
            "error": self.to_string(),
            "code": "consentement_requis",
        }))
    }
}

/// Consommateur authentifié **et** couvert par un consentement bêta portant
/// sur la version en vigueur du texte. C'est le portier RGPD : tout endpoint
/// qui traite des données de consommateur l'utilise à la place de
/// `AuthenticatedConsumer`, de sorte qu'un nouvel endpoint est bloqué par
/// défaut plutôt que d'ouvrir un trou par oubli.
///
/// Seuls les endpoints de gestion du consentement lui-même
/// (`/consommateurs/moi/consentement`) restent sur `AuthenticatedConsumer` :
/// il faut bien pouvoir consentir, ou se rétracter, sans avoir consenti.
///
/// Coûte une requête base par appel authentifié. C'est assumé pour un bêta :
/// mettre l'état du consentement dans le JWT rendrait un retrait sans effet
/// pendant les 30 jours de vie du jeton.
#[derive(Debug, Clone)]
pub struct ConsentedConsumer {
    pub consommateur_id: Uuid,
    pub email: String,
}

impl FromRequest for ConsentedConsumer {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let consumer = verify_consumer(req);
        let state = req.app_data::<web::Data<AppState>>().cloned();

        Box::pin(async move {
            let consumer = consumer?;
            require_consent(state, ConsentSubject::Consumer(consumer.consommateur_id)).await?;
            Ok(ConsentedConsumer {
                consommateur_id: consumer.consommateur_id,
                email: consumer.email,
            })
        })
    }
}

/// Marchand authentifié **et** couvert par un consentement bêta à jour.
///
/// Le pendant de `ConsentedConsumer` côté professionnel, et il pèse plus
/// lourd : un marchand confie son nom commercial, son adresse postale et sa
/// position GPS, toutes publiées sur la carte publique. Tous les endpoints
/// marchand passent par lui, y compris `/mcp` - un compte piloté depuis un
/// client MCP ne doit pas contourner le retrait de consentement.
#[derive(Debug, Clone)]
pub struct ConsentedMerchant {
    pub marchand_id: Uuid,
    pub email: String,
}

impl FromRequest for ConsentedMerchant {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let merchant = verify_merchant(req);
        let state = req.app_data::<web::Data<AppState>>().cloned();

        Box::pin(async move {
            let merchant = merchant?;
            require_consent(state, ConsentSubject::Merchant(merchant.marchand_id)).await?;
            Ok(ConsentedMerchant {
                marchand_id: merchant.marchand_id,
                email: merchant.email,
            })
        })
    }
}

/// Cœur commun aux deux portiers : refuse l'accès tant que le sujet n'a pas
/// de consentement portant sur la version en vigueur.
async fn require_consent(
    state: Option<web::Data<AppState>>,
    subject: ConsentSubject,
) -> Result<(), Error> {
    let state = state.ok_or_else(|| ErrorUnauthorized("internal server error"))?;

    match state.consent_use_cases.has_current_consent(subject).await {
        Ok(true) => Ok(()),
        Ok(false) => Err(ConsentRequired.into()),
        Err(err) => {
            tracing::error!(?err, role = subject.role(), "consent check failed");
            Err(ErrorInternalServerError("internal error"))
        }
    }
}

/// Admin identity extracted and verified from the JWT `Authorization: Bearer`
/// header. Backoffice léger, un seul compte - voir application::use_cases::AdminUseCases.
#[derive(Debug, Clone)]
pub struct AuthenticatedAdmin {
    pub admin_id: Uuid,
}

impl FromRequest for AuthenticatedAdmin {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let app_state = match req.app_data::<web::Data<AppState>>() {
            Some(state) => state,
            None => return ready(Err(ErrorUnauthorized("internal server error"))),
        };

        let token = match bearer_token(req) {
            Ok(t) => t,
            Err(err) => return ready(Err(err)),
        };

        match app_state.admin_auth_use_cases.verify_token(token) {
            Ok(claims) => ready(Ok(AuthenticatedAdmin {
                admin_id: claims.sub,
            })),
            Err(_) => ready(Err(ErrorUnauthorized("invalid or expired token"))),
        }
    }
}
