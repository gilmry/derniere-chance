mod auth;
mod catalog;
mod dashboard;
mod merchant;
mod product;
mod reservation;

pub use auth::{
    AuthResponse, Claims, LoginRequest, RegisterConsumerRequest, RegisterMerchantRequest,
};
pub use catalog::{MerchantProfileDto, OfferDto};
pub use dashboard::{ConsumerProfileDto, MerchantDashboardDto};
pub use merchant::MerchantResponseDto;
pub use product::{CreateProductDto, ProductResponseDto};
pub use reservation::ReservationConfirmationDto;
