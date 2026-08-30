use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::application::use_cases::ReservationError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{conflict, forbidden, internal_error, not_found};
use crate::infrastructure::web::middleware::{AuthenticatedMerchant, ConsentedConsumer};

pub async fn reserve(
    state: web::Data<AppState>,
    consumer: ConsentedConsumer,
    path: web::Path<Uuid>,
) -> HttpResponse {
    match state
        .reservation_use_cases
        .reserve(consumer.consommateur_id, path.into_inner())
        .await
    {
        Ok(confirmation) => HttpResponse::Created().json(confirmation),
        Err(ReservationError::ProductNotFound) => not_found("offer not found"),
        Err(err @ ReservationError::SoldOut) => conflict(&err.to_string()),
        Err(err) => {
            tracing::error!(?err, "reserve failed");
            internal_error()
        }
    }
}

pub async fn validate_pickup(
    state: web::Data<AppState>,
    merchant: AuthenticatedMerchant,
    path: web::Path<String>,
) -> HttpResponse {
    match state
        .reservation_use_cases
        .validate_pickup(merchant.marchand_id, &path.into_inner())
        .await
    {
        Ok(validation) => HttpResponse::Ok().json(validation),
        Err(ReservationError::ReservationNotFound) => not_found("reservation not found"),
        Err(ReservationError::Forbidden) => {
            forbidden("this reservation belongs to another marchand")
        }
        Err(err @ ReservationError::AlreadyRedeemed) => conflict(&err.to_string()),
        Err(err) => {
            tracing::error!(?err, "validate_pickup failed");
            internal_error()
        }
    }
}

pub async fn list_my_reservations(
    state: web::Data<AppState>,
    consumer: ConsentedConsumer,
) -> HttpResponse {
    match state
        .reservation_use_cases
        .list_my_reservations(consumer.consommateur_id)
        .await
    {
        Ok(reservations) => HttpResponse::Ok().json(reservations),
        Err(err) => {
            tracing::error!(?err, "list_my_reservations failed");
            internal_error()
        }
    }
}
