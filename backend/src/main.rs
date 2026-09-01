use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

use derniere_chance_api::application::ports::EventNotifier;
use derniere_chance_api::application::use_cases::{
    AdminAuthUseCases, AdminUseCases, CatalogUseCases, ConsentUseCases, ConsumerAuthUseCases,
    DashboardUseCases, MerchantAuthUseCases, OAuthUseCases, ProductUseCases, ReservationUseCases,
    SubscriptionUseCases,
};
use derniere_chance_api::infrastructure::bootstrap::bootstrap_admin;
use derniere_chance_api::infrastructure::database::create_pool;
use derniere_chance_api::infrastructure::database::repositories::{
    PostgresAdminRepository, PostgresAuthorizationCodeRepository, PostgresConsentRepository,
    PostgresConsumerRepository, PostgresMerchantRepository, PostgresNotificationRepository,
    PostgresOAuthClientRepository, PostgresProductRepository, PostgresRefreshTokenRepository,
    PostgresReservationRepository, PostgresSubscriptionRepository,
};
use derniere_chance_api::infrastructure::email::sender_from_env;
use derniere_chance_api::infrastructure::notifications::WebhookNotifier;
use derniere_chance_api::infrastructure::storage::{PhotoStorage, PhotoStorageConfig};
use derniere_chance_api::infrastructure::web::{configure_routes, AppState};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("SERVER_PORT must be a valid port number");

    let db = create_pool(&database_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("failed to run database migrations");

    bootstrap_admin(&db).await;

    let merchant_repo = Arc::new(PostgresMerchantRepository::new(db.clone()));
    let consumer_repo = Arc::new(PostgresConsumerRepository::new(db.clone()));
    let product_repo = Arc::new(PostgresProductRepository::new(db.clone()));
    let subscription_repo = Arc::new(PostgresSubscriptionRepository::new(db.clone()));
    let reservation_repo = Arc::new(PostgresReservationRepository::new(db.clone()));
    let notification_repo = Arc::new(PostgresNotificationRepository::new(db.clone()));
    let admin_repo = Arc::new(PostgresAdminRepository::new(db.clone()));
    let consent_repo = Arc::new(PostgresConsentRepository::new(db.clone()));
    let oauth_client_repo = Arc::new(PostgresOAuthClientRepository::new(db.clone()));
    let oauth_code_repo = Arc::new(PostgresAuthorizationCodeRepository::new(db.clone()));
    let oauth_refresh_repo = Arc::new(PostgresRefreshTokenRepository::new(db.clone()));

    // SMTP, Mailjet ou simple journalisation selon ce qui est configuré (voir
    // `infrastructure::email::sender_from_env`). Le nom est journalisé au
    // démarrage : sans lui, une variable oubliée se traduirait par un silence
    // indiscernable d'un envoi réussi.
    let (email_sender, email_provider) = sender_from_env();
    if email_provider == "journalisation seule" {
        tracing::warn!(
            "aucun transporteur d'email configuré - les notifications de démarque seront \
             seulement journalisées, aucun email ne partira"
        );
    } else {
        tracing::info!(%email_provider, "transporteur d'email en service");
    }

    let photo_storage = Arc::new(PhotoStorage::from_config(PhotoStorageConfig::from_env()).await);

    // n8n webhook -> email for noteworthy events (nouveau marchand, nouveau
    // client, nouvelle réservation, panier récupéré). No-op if
    // WEBHOOK_NOTIFY_URL is unset.
    let event_notifier: Arc<dyn EventNotifier> = Arc::new(WebhookNotifier::from_env());

    let admin_use_cases = Arc::new(AdminUseCases::new(
        merchant_repo.clone(),
        consumer_repo.clone(),
        product_repo.clone(),
        reservation_repo.clone(),
    ));

    let merchant_auth_use_cases = Arc::new(MerchantAuthUseCases::new(
        merchant_repo.clone(),
        consent_repo.clone(),
        jwt_secret.clone(),
        event_notifier.clone(),
    ));

    let state = web::Data::new(AppState {
        merchant_auth_use_cases: merchant_auth_use_cases.clone(),
        consumer_auth_use_cases: Arc::new(ConsumerAuthUseCases::new(
            consumer_repo.clone(),
            consent_repo.clone(),
            jwt_secret.clone(),
            event_notifier.clone(),
        )),
        admin_auth_use_cases: Arc::new(AdminAuthUseCases::new(admin_repo, jwt_secret)),
        consent_use_cases: Arc::new(ConsentUseCases::new(
            consent_repo,
            consumer_repo.clone(),
            merchant_repo.clone(),
            product_repo.clone(),
            event_notifier.clone(),
        )),
        admin_use_cases,
        catalog_use_cases: Arc::new(CatalogUseCases::new(
            product_repo.clone(),
            merchant_repo.clone(),
        )),
        product_use_cases: Arc::new(ProductUseCases::new(
            product_repo.clone(),
            merchant_repo.clone(),
            subscription_repo.clone(),
            notification_repo,
            email_sender,
        )),
        subscription_use_cases: Arc::new(SubscriptionUseCases::new(
            subscription_repo,
            merchant_repo.clone(),
        )),
        reservation_use_cases: Arc::new(ReservationUseCases::new(
            reservation_repo.clone(),
            product_repo,
            event_notifier,
        )),
        dashboard_use_cases: Arc::new(DashboardUseCases::new(reservation_repo)),
        photo_storage,
        oauth_use_cases: Arc::new(OAuthUseCases::new(
            oauth_client_repo,
            oauth_code_repo,
            oauth_refresh_repo,
            merchant_repo,
            merchant_auth_use_cases,
        )),
    });

    tracing::info!("derniere-chance-api listening on {host}:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Cors::permissive())
            .wrap(tracing_actix_web::TracingLogger::default())
            .configure(configure_routes)
    })
    .bind((host, port))?
    .run()
    .await
}
