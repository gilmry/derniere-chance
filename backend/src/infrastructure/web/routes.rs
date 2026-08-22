use actix_web::web;

use crate::infrastructure::web::handlers;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(handlers::health))
        // Auth
        .route(
            "/marchands/inscription",
            web::post().to(handlers::merchant_auth::register),
        )
        .route(
            "/marchands/connexion",
            web::post().to(handlers::merchant_auth::login),
        )
        .route(
            "/consommateurs/inscription",
            web::post().to(handlers::consumer_auth::register),
        )
        .route(
            "/consommateurs/connexion",
            web::post().to(handlers::consumer_auth::login),
        )
        // Marchand backoffice (auth: marchand)
        .route(
            "/marchands/moi/produits",
            web::post().to(handlers::publish),
        )
        .route(
            "/marchands/moi/produits",
            web::get().to(handlers::list_mine),
        )
        .route(
            "/marchands/moi/produits/{id}/ecoule",
            web::patch().to(handlers::mark_ecoule),
        )
        .route(
            "/marchands/moi/dashboard",
            web::get().to(handlers::merchant_today),
        )
        .route(
            "/marchands/moi/reservations/{code}/valider",
            web::post().to(handlers::validate_pickup),
        )
        // Consommateur account (auth: consommateur)
        .route(
            "/consommateurs/moi/abonnements",
            web::get().to(handlers::list_followed),
        )
        .route(
            "/consommateurs/moi/profil",
            web::get().to(handlers::consumer_profile),
        )
        // Public catalogue
        .route("/offres", web::get().to(handlers::list_offers))
        .route("/offres/{id}", web::get().to(handlers::get_offer))
        .route("/marchands/{id}", web::get().to(handlers::get_merchant))
        // Consumer actions on a specific offer/marchand (auth: consommateur)
        .route(
            "/offres/{id}/reservation",
            web::post().to(handlers::reserve),
        )
        .route(
            "/marchands/{id}/abonnement",
            web::post().to(handlers::follow),
        )
        .route(
            "/marchands/{id}/abonnement",
            web::delete().to(handlers::unfollow),
        );
}
