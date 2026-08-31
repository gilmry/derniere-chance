//! Envoi réel d'une alerte « nouvelle démarque » via Mailjet.
//!
//! Ignoré par défaut : ce test sort sur le réseau et écrit dans une vraie
//! boîte mail. Il ne doit jamais partir en CI, mais il reste ici parce que
//! c'est le seul moyen de vérifier ce que les tests unitaires ne peuvent pas
//! voir : clés valides, adresse d'expédition validée chez Mailjet, SPF/DKIM
//! posés. À rejouer après toute rotation de clés ou changement d'expéditeur.
//!
//! ```sh
//! cd backend
//! MAILJET_TEST_TO=vous@example.org cargo test --test mailjet_smoke -- --ignored --nocapture
//! ```
//!
//! Les identifiants sont lus dans le `.env` du dépôt (MAILJET_API_KEY,
//! MAILJET_SECRET_KEY, MAILJET_FROM_EMAIL).

use chrono::{Duration, Utc};
use rust_decimal_macros::dec;
use uuid::Uuid;

use derniere_chance_api::application::ports::EmailSender;
use derniere_chance_api::domain::entities::{Merchant, Product, ProductStatus};
use derniere_chance_api::infrastructure::email::MailjetEmailSender;

#[tokio::test]
#[ignore = "envoie un vrai email : lancer avec --ignored et MAILJET_TEST_TO"]
async fn envoie_une_vraie_alerte_nouvelle_demarque() {
    // Chemin explicite : `dotenvy::dotenv()` remonte depuis le répertoire
    // courant, mais s'arrête à la première ligne qu'il ne sait pas lire et
    // rend la main en silence si on avale l'erreur.
    let env_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env");
    if let Err(err) = dotenvy::from_path(&env_file) {
        eprintln!("{} non chargé ({err}) : on se rabat sur l'environnement", env_file.display());
    }

    let destinataire = std::env::var("MAILJET_TEST_TO")
        .expect("MAILJET_TEST_TO doit contenir une adresse qui vous appartient");

    let sender = MailjetEmailSender::from_env().expect(
        "MAILJET_API_KEY, MAILJET_SECRET_KEY et MAILJET_FROM_EMAIL doivent être renseignés",
    );

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
        .expect("Mailjet a refusé l'envoi");

    println!("email envoyé à {destinataire}");
}
