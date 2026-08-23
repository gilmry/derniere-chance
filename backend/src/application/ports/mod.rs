mod admin_repository;
mod consumer_repository;
mod email_sender;
mod error;
mod event_notifier;
mod merchant_repository;
mod notification_repository;
mod oauth_repository;
mod product_repository;
mod reservation_repository;
mod subscription_repository;

pub use admin_repository::AdminRepository;
pub use consumer_repository::{ConsumerRepository, NewConsumer};
pub use email_sender::{EmailError, EmailSender};
pub use error::RepoError;
pub use event_notifier::EventNotifier;
pub use merchant_repository::{MerchantRepository, MerchantUpdate, NewMerchant};
pub use notification_repository::{NewNotification, NotificationRepository};
pub use oauth_repository::{
    AuthorizationCodeRepository, NewAuthorizationCode, NewOAuthClient, NewRefreshToken,
    OAuthClientRepository, RefreshTokenRepository,
};
pub use product_repository::{NewProduct, ProductRepository, ProductUpdate, ProductWithMerchant};
pub use reservation_repository::{
    ConsumerStats, MerchantDailyStats, NewReservation, ReservationRepository, ReservationSummary,
};
pub use subscription_repository::{SubscriberContact, SubscriptionRepository};
