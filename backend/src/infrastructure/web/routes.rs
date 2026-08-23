use actix_web::web;

use crate::infrastructure::web::{handlers, mcp, oauth};

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(handlers::health))
        // MCP (marchand only, read+write - see infrastructure::web::mcp) +
        // its OAuth 2.1/PKCE authorization server (infrastructure::web::oauth)
        .service(
            web::scope("/mcp")
                .app_data(web::JsonConfig::default().error_handler(mcp::json_error_handler))
                .route("", web::post().to(mcp::handle)),
        )
        .route(
            "/.well-known/oauth-authorization-server",
            web::get().to(oauth::metadata),
        )
        .route("/oauth/register", web::post().to(oauth::register))
        .route("/oauth/authorize", web::get().to(oauth::authorize_get))
        .route("/oauth/authorize", web::post().to(oauth::authorize_post))
        .route("/oauth/token", web::post().to(oauth::token))
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
        .route("/marchands/moi", web::get().to(handlers::merchant_auth::me))
        .route(
            "/marchands/moi",
            web::patch().to(handlers::merchant_auth::update_me),
        )
        .route(
            "/marchands/moi/produits",
            web::post().to(handlers::publish),
        )
        .route(
            "/marchands/moi/produits/photo",
            web::post().to(handlers::upload_photo),
        )
        .route(
            "/marchands/moi/logo",
            web::post().to(handlers::upload_logo),
        )
        .route(
            "/marchands/moi/produits",
            web::get().to(handlers::list_mine),
        )
        .route(
            "/marchands/moi/produits/{id}",
            web::patch().to(handlers::update_product),
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
        .route(
            "/consommateurs/moi/reservations",
            web::get().to(handlers::list_my_reservations),
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
        )
        // Backoffice admin (auth: admin)
        .route("/admin/connexion", web::post().to(handlers::admin_auth::login))
        .route("/admin/marchands", web::get().to(handlers::list_merchants))
        .route(
            "/admin/marchands/{id}",
            web::delete().to(handlers::delete_merchant),
        )
        .route("/admin/consommateurs", web::get().to(handlers::list_consumers))
        .route(
            "/admin/consommateurs/{id}",
            web::delete().to(handlers::delete_consumer),
        )
        .route("/admin/produits", web::get().to(handlers::list_products))
        .route(
            "/admin/produits/{id}",
            web::delete().to(handlers::delete_product),
        )
        .route(
            "/admin/produits/{id}/depublier",
            web::patch().to(handlers::unpublish_product),
        )
        .route("/admin/stats", web::get().to(handlers::admin_stats));
}
