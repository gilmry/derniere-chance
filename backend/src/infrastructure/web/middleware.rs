use actix_web::{dev::Payload, error::ErrorUnauthorized, web, Error, FromRequest, HttpRequest};
use std::future::{ready, Ready};
use uuid::Uuid;

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

impl FromRequest for AuthenticatedMerchant {
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

        match app_state.merchant_auth_use_cases.verify_token(token) {
            Ok(claims) => ready(Ok(AuthenticatedMerchant {
                marchand_id: claims.sub,
                email: claims.email,
            })),
            Err(_) => ready(Err(ErrorUnauthorized("invalid or expired token"))),
        }
    }
}

/// Consommateur identity extracted and verified from the JWT `Authorization:
/// Bearer` header.
#[derive(Debug, Clone)]
pub struct AuthenticatedConsumer {
    pub consommateur_id: Uuid,
    pub email: String,
}

impl FromRequest for AuthenticatedConsumer {
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

        match app_state.consumer_auth_use_cases.verify_token(token) {
            Ok(claims) => ready(Ok(AuthenticatedConsumer {
                consommateur_id: claims.sub,
                email: claims.email,
            })),
            Err(_) => ready(Err(ErrorUnauthorized("invalid or expired token"))),
        }
    }
}
