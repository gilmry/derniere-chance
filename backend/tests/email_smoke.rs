//! Envoi réel d'une alerte « nouvelle démarque », par le transporteur
//! effectivement configuré.
//!
//! Ignoré par défaut : ce test sort sur le réseau et écrit dans une vraie
//! boîte mail. Il ne doit jamais partir en CI, mais il reste ici parce que
//! c'est le seul moyen de vérifier ce que les tests unitaires ne peuvent pas
//! voir : identifiants valides, expéditeur autorisé par le relais, SPF/DKIM
//! posés. À rejouer après toute rotation de jeton ou changement de
//! fournisseur.
//!
//! ```sh
//! cd backend
//! EMAIL_TEST_TO=vous@example.org cargo test --test email_smoke -- --ignored --nocapture
//! ```
//!
//! Il passe par `sender_from_env`, donc il éprouve exactement l'adaptateur qui
//! tournera en production, et son nom est affiché avant l'envoi.

use chrono::{Duration, Utc};
use rust_decimal_macros::dec;
use uuid::Uuid;

use derniere_chance_api::domain::entities::{Merchant, Product, ProductStatus};
use derniere_chance_api::infrastructure::email::sender_from_env;

#[tokio::test]
#[ignore = "envoie un vrai email : lancer avec --ignored et EMAIL_TEST_TO"]
async fn envoie_une_vraie_alerte_nouvelle_demarque() {
    // Chemin explicite : `dotenvy::dotenv()` remonte depuis le répertoire
    // courant, mais s'arrête à la première ligne qu'il ne sait pas lire et
    // rend la main en silence si on avale l'erreur.
    let env_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env");
    if let Err(err) = dotenvy::from_path(&env_file) {
        eprintln!(
            "{} non chargé ({err}) : on se rabat sur l'environnement",
            env_file.display()
        );
    }

    let destinataire = std::env::var("EMAIL_TEST_TO")
        .expect("EMAIL_TEST_TO doit contenir une adresse qui vous appartient");

    let (sender, provider) = sender_from_env();
    assert_ne!(
        provider, "journalisation seule",
        "aucun transporteur configuré : ce test n'enverrait rien et passerait pour un succès"
    );
    println!("transporteur : {provider}");

    let now = Utc::now();
    let merchant = Merchant {
        id: Uuid::new_v4(),
        nom: "Boulangerie du Marché".into(),
        adresse: "Rue du Marché 4, 1000 Bruxelles".into(),
        categorie: "boulangerie".into(),
        note: None,
        email: "pro@example.org".into(),
        password_hash: String::new(),
        latitude: None,
        longitude: None,
        logo_url: None,
        created_at: now,
        anonymise_le: None,
    };
    let product = Product {
        id: Uuid::new_v4(),
        marchand_id: merchant.id,
        nom: "Panier boulanger surprise".into(),
        description: "Pains & viennoiseries du jour, choisis par le boulanger.".into(),
        prix_initial: dec!(8.00),
        prix_demarque: dec!(3.20),
        quantite: 5,
        retrait_debut: now + Duration::hours(3),
        retrait_fin: now + Duration::hours(5),
        statut: ProductStatus::Publie,
        photo_url: None,
        created_at: now,
    };

    sender
        .send_new_offer_notification(&destinataire, &merchant, &product)
        .await
        .expect("le transporteur a refusé l'envoi");

    println!("email envoyé à {destinataire}");
}
