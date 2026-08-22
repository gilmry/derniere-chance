mod catalog_handlers;
mod consumer_auth_handlers;
mod dashboard_handlers;
mod merchant_auth_handlers;
mod product_handlers;
mod reservation_handlers;
mod responses;
mod subscription_handlers;

use actix_web::HttpResponse;

pub use catalog_handlers::{get_merchant, get_offer, list_offers};
pub use dashboard_handlers::{consumer_profile, merchant_today};
pub use product_handlers::{list_mine, mark_ecoule, publish};
pub use reservation_handlers::{reserve, validate_pickup};
pub use subscription_handlers::{follow, list_followed, unfollow};

pub mod consumer_auth {
    pub use super::consumer_auth_handlers::{login, register};
}

pub mod merchant_auth {
    pub use super::merchant_auth_handlers::{login, register};
}

/// Liveness/readiness probe.
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({ "status": "ok" }))
}
