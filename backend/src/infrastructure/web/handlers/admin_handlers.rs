use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::application::use_cases::AdminError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{internal_error, not_found};
use crate::infrastructure::web::middleware::AuthenticatedAdmin;

fn map_err(err: AdminError) -> HttpResponse {
    match err {
        AdminError::NotFound => not_found("resource not found"),
        err => {
            tracing::error!(?err, "admin action failed");
            internal_error()
        }
    }
}

pub async fn list_merchants(state: web::Data<AppState>, _admin: AuthenticatedAdmin) -> HttpResponse {
    match state.admin_use_cases.list_merchants().await {
        Ok(merchants) => HttpResponse::Ok().json(merchants),
        Err(err) => map_err(err),
    }
}

pub async fn delete_merchant(
    state: web::Data<AppState>,
    _admin: AuthenticatedAdmin,
    path: web::Path<Uuid>,
) -> HttpResponse {
    match state.admin_use_cases.delete_merchant(path.into_inner()).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(err) => map_err(err),
    }
}

pub async fn list_consumers(state: web::Data<AppState>, _admin: AuthenticatedAdmin) -> HttpResponse {
    match state.admin_use_cases.list_consumers().await {
        Ok(consumers) => HttpResponse::Ok().json(consumers),
        Err(err) => map_err(err),
    }
}

pub async fn delete_consumer(
    state: web::Data<AppState>,
    _admin: AuthenticatedAdmin,
    path: web::Path<Uuid>,
) -> HttpResponse {
    match state.admin_use_cases.delete_consumer(path.into_inner()).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(err) => map_err(err),
    }
}

pub async fn list_products(state: web::Data<AppState>, _admin: AuthenticatedAdmin) -> HttpResponse {
    match state.admin_use_cases.list_products().await {
        Ok(products) => HttpResponse::Ok().json(products),
        Err(err) => map_err(err),
    }
}

pub async fn delete_product(
    state: web::Data<AppState>,
    _admin: AuthenticatedAdmin,
    path: web::Path<Uuid>,
) -> HttpResponse {
    match state.admin_use_cases.delete_product(path.into_inner()).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(err) => map_err(err),
    }
}

pub async fn unpublish_product(
    state: web::Data<AppState>,
    _admin: AuthenticatedAdmin,
    path: web::Path<Uuid>,
) -> HttpResponse {
    match state.admin_use_cases.unpublish_product(path.into_inner()).await {
        Ok(()) => HttpResponse::NoContent().finish(),
        Err(err) => map_err(err),
    }
}

pub async fn stats(state: web::Data<AppState>, _admin: AuthenticatedAdmin) -> HttpResponse {
    match state.admin_use_cases.stats().await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => map_err(err),
    }
}
