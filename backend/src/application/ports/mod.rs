mod admin_repository;
mod consumer_repository;
mod email_sender;
mod error;
mod merchant_repository;
mod notification_repository;
mod product_repository;
mod reservation_repository;
mod subscription_repository;

pub use admin_repository::AdminRepository;
pub use consumer_repository::{ConsumerRepository, NewConsumer};
pub use email_sender::{EmailError, EmailSender};
pub use error::RepoError;
pub use merchant_repository::{MerchantRepository, NewMerchant};
pub use notification_repository::{NewNotification, NotificationRepository};
pub use product_repository::{NewProduct, ProductRepository, ProductWithMerchant};
pub use reservation_repository::{
    ConsumerStats, MerchantDailyStats, NewReservation, ReservationRepository,
};
pub use subscription_repository::{SubscriberContact, SubscriptionRepository};
