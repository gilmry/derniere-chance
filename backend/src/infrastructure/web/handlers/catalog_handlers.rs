use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::application::use_cases::CatalogError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{internal_error, not_found};

#[derive(Debug, Deserialize)]
pub struct ListOffersQuery {
    pub categorie: Option<String>,
}

/// Public feed: every active démarque, optionally filtered by marchand
/// categorie (`?categorie=Boulangerie`).
pub async fn list_offers(
    state: web::Data<AppState>,
    query: web::Query<ListOffersQuery>,
) -> HttpResponse {
    match state
        .catalog_use_cases
        .list_active_offers(query.into_inner().categorie)
        .await
    {
        Ok(offers) => HttpResponse::Ok().json(offers),
        Err(err) => {
            tracing::error!(?err, "list_offers failed");
            internal_error()
        }
    }
}

pub async fn get_offer(state: web::Data<AppState>, path: web::Path<Uuid>) -> HttpResponse {
    match state.catalog_use_cases.get_offer(path.into_inner()).await {
        Ok(offer) => HttpResponse::Ok().json(offer),
        Err(CatalogError::NotFound) => not_found("offer not found"),
        Err(err) => {
            tracing::error!(?err, "get_offer failed");
            internal_error()
        }
    }
}

pub async fn get_merchant(state: web::Data<AppState>, path: web::Path<Uuid>) -> HttpResponse {
    match state
        .catalog_use_cases
        .get_merchant_profile(path.into_inner())
        .await
    {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(CatalogError::NotFound) => not_found("merchant not found"),
        Err(err) => {
            tracing::error!(?err, "get_merchant failed");
            internal_error()
        }
    }
}
