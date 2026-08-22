use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::application::use_cases::CatalogError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{internal_error, not_found};

#[derive(Debug, Deserialize)]
pub struct ListOffersQuery {
    pub categorie: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct GeoQuery {
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

impl GeoQuery {
    fn coords(&self) -> Option<(f64, f64)> {
        self.lat.zip(self.lon)
    }
}

/// Public feed: every active démarque, optionally filtered by marchand
/// categorie (`?categorie=Boulangerie`) and sorted by proximity when the
/// consommateur shares their position (`?lat=&lon=`).
pub async fn list_offers(
    state: web::Data<AppState>,
    query: web::Query<ListOffersQuery>,
) -> HttpResponse {
    let query = query.into_inner();
    let coords = query.lat.zip(query.lon);
    match state
        .catalog_use_cases
        .list_active_offers(query.categorie, coords)
        .await
    {
        Ok(offers) => HttpResponse::Ok().json(offers),
        Err(err) => {
            tracing::error!(?err, "list_offers failed");
            internal_error()
        }
    }
}

pub async fn get_offer(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<GeoQuery>,
) -> HttpResponse {
    match state
        .catalog_use_cases
        .get_offer(path.into_inner(), query.coords())
        .await
    {
        Ok(offer) => HttpResponse::Ok().json(offer),
        Err(CatalogError::NotFound) => not_found("offer not found"),
        Err(err) => {
            tracing::error!(?err, "get_offer failed");
            internal_error()
        }
    }
}

pub async fn get_merchant(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    query: web::Query<GeoQuery>,
) -> HttpResponse {
    match state
        .catalog_use_cases
        .get_merchant_profile(path.into_inner(), query.coords())
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
