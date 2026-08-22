use std::sync::Arc;

use crate::application::use_cases::{
    CatalogUseCases, ConsumerAuthUseCases, DashboardUseCases, MerchantAuthUseCases,
    ProductUseCases, ReservationUseCases, SubscriptionUseCases,
};

#[derive(Clone)]
pub struct AppState {
    pub merchant_auth_use_cases: Arc<MerchantAuthUseCases>,
    pub consumer_auth_use_cases: Arc<ConsumerAuthUseCases>,
    pub catalog_use_cases: Arc<CatalogUseCases>,
    pub product_use_cases: Arc<ProductUseCases>,
    pub subscription_use_cases: Arc<SubscriptionUseCases>,
    pub reservation_use_cases: Arc<ReservationUseCases>,
    pub dashboard_use_cases: Arc<DashboardUseCases>,
}
