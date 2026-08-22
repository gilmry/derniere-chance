use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::application::use_cases::SubscriptionError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{internal_error, not_found};
use crate::infrastructure::web::middleware::AuthenticatedConsumer;

pub async fn follow(
    state: web::Data<AppState>,
    consumer: AuthenticatedConsumer,
    path: web::Path<Uuid>,
) -> HttpResponse {
    match state
        .subscription_use_cases
        .follow(consumer.consommateur_id, path.into_inner())
        .await
    {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(SubscriptionError::MerchantNotFound) => not_found("merchant not found"),
        Err(err) => {
            tracing::error!(?err, "follow failed");
            internal_error()
        }
    }
}

pub async fn unfollow(
    state: web::Data<AppState>,
    consumer: AuthenticatedConsumer,
    path: web::Path<Uuid>,
) -> HttpResponse {
    match state
        .subscription_use_cases
        .unfollow(consumer.consommateur_id, path.into_inner())
        .await
    {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(err) => {
            tracing::error!(?err, "unfollow failed");
            internal_error()
        }
    }
}

pub async fn list_followed(
    state: web::Data<AppState>,
    consumer: AuthenticatedConsumer,
) -> HttpResponse {
    match state
        .subscription_use_cases
        .list_followed(consumer.consommateur_id)
        .await
    {
        Ok(merchants) => HttpResponse::Ok().json(merchants),
        Err(err) => {
            tracing::error!(?err, "list_followed failed");
            internal_error()
        }
    }
}
