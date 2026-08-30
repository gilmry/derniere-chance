use actix_web::{web, HttpResponse};
use uuid::Uuid;

use crate::application::dto::CreateProductDto;
use crate::application::use_cases::ProductError;
use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::{
    bad_request, forbidden, internal_error, not_found,
};
use crate::infrastructure::web::middleware::ConsentedMerchant;

pub async fn publish(
    state: web::Data<AppState>,
    merchant: ConsentedMerchant,
    dto: web::Json<CreateProductDto>,
) -> HttpResponse {
    match state
        .product_use_cases
        .publish(merchant.marchand_id, dto.into_inner())
        .await
    {
        Ok(product) => HttpResponse::Created().json(product),
        Err(ProductError::InvalidInput(msg)) => bad_request(&msg),
        Err(ProductError::NotFound) => not_found("merchant not found"),
        Err(err) => {
            tracing::error!(?err, "publish product failed");
            internal_error()
        }
    }
}

pub async fn update_product(
    state: web::Data<AppState>,
    merchant: ConsentedMerchant,
    path: web::Path<Uuid>,
    dto: web::Json<CreateProductDto>,
) -> HttpResponse {
    match state
        .product_use_cases
        .update(merchant.marchand_id, path.into_inner(), dto.into_inner())
        .await
    {
        Ok(product) => HttpResponse::Ok().json(product),
        Err(ProductError::InvalidInput(msg)) => bad_request(&msg),
        Err(ProductError::NotFound) => not_found("product not found"),
        Err(ProductError::Forbidden) => forbidden("this produit belongs to another marchand"),
        Err(err) => {
            tracing::error!(?err, "update product failed");
            internal_error()
        }
    }
}

pub async fn mark_ecoule(
    state: web::Data<AppState>,
    merchant: ConsentedMerchant,
    path: web::Path<Uuid>,
) -> HttpResponse {
    match state
        .product_use_cases
        .mark_ecoule(merchant.marchand_id, path.into_inner())
        .await
    {
        Ok(product) => HttpResponse::Ok().json(product),
        Err(ProductError::NotFound) => not_found("product not found"),
        Err(ProductError::Forbidden) => forbidden("this produit belongs to another marchand"),
        Err(err) => {
            tracing::error!(?err, "mark_ecoule failed");
            internal_error()
        }
    }
}

pub async fn list_mine(state: web::Data<AppState>, merchant: ConsentedMerchant) -> HttpResponse {
    match state.product_use_cases.list_mine(merchant.marchand_id).await {
        Ok(products) => HttpResponse::Ok().json(products),
        Err(err) => {
            tracing::error!(?err, "list_mine failed");
            internal_error()
        }
    }
}
