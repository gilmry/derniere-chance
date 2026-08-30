use actix_web::{web, HttpResponse};

use crate::infrastructure::web::app_state::AppState;
use crate::infrastructure::web::handlers::responses::internal_error;
use crate::infrastructure::web::middleware::{AuthenticatedMerchant, ConsentedConsumer};

pub async fn merchant_today(
    state: web::Data<AppState>,
    merchant: AuthenticatedMerchant,
) -> HttpResponse {
    match state.dashboard_use_cases.merchant_today(merchant.marchand_id).await {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => {
            tracing::error!(?err, "merchant_today failed");
            internal_error()
        }
    }
}

pub async fn consumer_profile(
    state: web::Data<AppState>,
    consumer: ConsentedConsumer,
) -> HttpResponse {
    match state
        .dashboard_use_cases
        .consumer_profile(consumer.consommateur_id)
        .await
    {
        Ok(stats) => HttpResponse::Ok().json(stats),
        Err(err) => {
            tracing::error!(?err, "consumer_profile failed");
            internal_error()
        }
    }
}
