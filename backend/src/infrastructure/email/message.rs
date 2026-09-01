//! Rendu du corps des emails, partagé par tous les adaptateurs.
//!
//! Le gabarit vit ici et non dans un adaptateur : le contenu envoyé à un
//! testeur ne doit pas dépendre du fournisseur qui l'achemine. Changer de
//! transporteur ne doit rien changer à ce que la personne lit, et un seul jeu
//! de tests couvre les deux chemins.

use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, TimeZone, Utc, Weekday};
use rust_decimal::Decimal;

use crate::domain::entities::{Merchant, Product};
use crate::domain::services::pricing::discount_percent;

/// Un email prêt à poster, indépendant du transport.
pub(super) struct RenderedEmail {
    pub subject: String,
    pub text: String,
    pub html: String,
}

/// Alerte « nouvelle démarque » envoyée aux abonnés d'un marchand.
pub(super) fn new_offer_email(
    merchant: &Merchant,
    product: &Product,
    app_base_url: &str,
) -> RenderedEmail {
    let offer_url = format!("{app_base_url}/offre?id={}", product.id);
    let profile_url = format!("{app_base_url}/profil");

    let subject = format!("{} : {}", merchant.nom, product.nom);

    let text = format!(
        "Bonjour,\n\n\
         {marchand} vient de publier une démarque sur DernièreChance.\n\n\
         {produit}\n\
         {description}\n\n\
         Prix : {demarque} au lieu de {initial} (-{remise} %)\n\
         Quantité : {quantite}\n\
         Retrait : {retrait}\n\
         Adresse : {adresse}\n\n\
         Réserver : {lien}\n\n\
         --\n\
         Vous recevez cet email parce que vous suivez {marchand} sur \
         DernièreChance. Pour ne plus recevoir ces alertes, retirez ce \
         commerçant de vos marchands suivis : {profil}\n",
        marchand = merchant.nom,
        produit = product.nom,
        description = product.description,
        demarque = euros(product.prix_demarque),
        initial = euros(product.prix_initial),
        remise = discount_percent(product.prix_initial, product.prix_demarque),
        quantite = product.quantite,
        retrait = pickup_window(product.retrait_debut, product.retrait_fin),
        adresse = merchant.adresse,
        lien = offer_url,
        profil = profile_url,
    );

    let html = format!(
        "<!DOCTYPE html><html lang=\"fr\"><body \
         style=\"font-family:system-ui,sans-serif;color:#1a1a1a;line-height:1.5\">\
         <p>Bonjour,</p>\
         <p><strong>{marchand}</strong> vient de publier une démarque sur DernièreChance.</p>\
         <h2 style=\"margin-bottom:4px\">{produit}</h2>\
         <p style=\"margin-top:0\">{description}</p>\
         <ul>\
         <li><strong>{demarque}</strong> au lieu de {initial} (-{remise} %)</li>\
         <li>Quantité : {quantite}</li>\
         <li>Retrait : {retrait}</li>\
         <li>Adresse : {adresse}</li>\
         </ul>\
         <p><a href=\"{lien}\">Réserver ce panier</a></p>\
         <hr>\
         <p style=\"font-size:12px;color:#666\">Vous recevez cet email parce que vous \
         suivez {marchand} sur DernièreChance. Pour ne plus recevoir ces alertes, \
         retirez ce commerçant de vos marchands suivis depuis \
         <a href=\"{profil}\">votre profil</a>.</p>\
         </body></html>",
        marchand = escape_html(&merchant.nom),
        produit = escape_html(&product.nom),
        description = escape_html(&product.description),
        demarque = euros(product.prix_demarque),
        initial = euros(product.prix_initial),
        remise = discount_percent(product.prix_initial, product.prix_demarque),
        quantite = product.quantite,
        retrait = pickup_window(product.retrait_debut, product.retrait_fin),
        adresse = escape_html(&merchant.adresse),
        lien = escape_html(&offer_url),
        profil = escape_html(&profile_url),
    );

    RenderedEmail {
        subject,
        text,
        html,
    }
}

/// Lien de réinitialisation de mot de passe.
///
/// Le message ne nomme ni le compte ni la personne : il peut atterrir dans la
/// boîte de quelqu'un qui n'a rien demandé, parce que n'importe qui peut
/// saisir une adresse dans le formulaire. Il dit donc quoi faire, et surtout
/// quoi faire si on n'a rien demandé.
pub(super) fn password_reset_email(reset_url: &str, expires_in_minutes: i64) -> RenderedEmail {
    let subject = "Réinitialiser votre mot de passe DernièreChance".to_string();

    let text = format!(
        "Bonjour,\n\n\
         Une réinitialisation de mot de passe a été demandée pour ce compte \
         DernièreChance. Pour choisir un nouveau mot de passe, ouvrez ce lien \
         dans les {minutes} minutes :\n\n\
         {lien}\n\n\
         Passé ce délai, le lien ne fonctionnera plus et il faudra en demander \
         un autre. Il ne fonctionne qu'une seule fois.\n\n\
         Si vous n'êtes pas à l'origine de cette demande, ignorez cet email : \
         votre mot de passe actuel reste valable et personne n'a eu accès à \
         votre compte.\n",
        minutes = expires_in_minutes,
        lien = reset_url,
    );

    let html = format!(
        "<!DOCTYPE html><html lang=\"fr\"><body \
         style=\"font-family:system-ui,sans-serif;color:#1a1a1a;line-height:1.5\">\
         <p>Bonjour,</p>\
         <p>Une réinitialisation de mot de passe a été demandée pour ce compte \
         DernièreChance. Pour choisir un nouveau mot de passe, ouvrez ce lien dans \
         les {minutes} minutes :</p>\
         <p><a href=\"{lien}\">Choisir un nouveau mot de passe</a></p>\
         <p>Passé ce délai, le lien ne fonctionnera plus et il faudra en demander un \
         autre. Il ne fonctionne qu'une seule fois.</p>\
         <hr>\
         <p style=\"font-size:12px;color:#666\">Si vous n'êtes pas à l'origine de cette \
         demande, ignorez cet email : votre mot de passe actuel reste valable et \
         personne n'a eu accès à votre compte.</p>\
         </body></html>",
        minutes = expires_in_minutes,
        lien = escape_html(reset_url),
    );

    RenderedEmail {
        subject,
        text,
        html,
    }
}

/// « 3,20 € » : montant à deux décimales, virgule décimale française.
fn euros(amount: Decimal) -> String {
    format!("{amount:.2} €").replace('.', ",")
}

/// « le 02/09 entre 17:30 et 19:00 », en heure de Bruxelles : le destinataire
/// lit une heure de boutique, pas de l'UTC.
fn pickup_window(debut: DateTime<Utc>, fin: DateTime<Utc>) -> String {
    let debut = to_brussels(debut);
    let fin = to_brussels(fin);

    if debut.date_naive() == fin.date_naive() {
        format!(
            "le {} entre {} et {}",
            debut.format("%d/%m"),
            debut.format("%H:%M"),
            fin.format("%H:%M")
        )
    } else {
        format!(
            "du {} au {}",
            debut.format("%d/%m à %H:%M"),
            fin.format("%d/%m à %H:%M")
        )
    }
}

/// Heure locale belge sans embarquer une base de fuseaux (chrono-tz) pour un
/// seul pays : l'Union européenne fixe le passage à l'heure d'été au dernier
/// dimanche de mars à 01:00 UTC et le retour au dernier dimanche d'octobre à
/// la même heure, règle inchangée depuis la directive 2000/84/CE.
fn to_brussels(instant: DateTime<Utc>) -> DateTime<FixedOffset> {
    instant.with_timezone(&brussels_offset(instant))
}

fn brussels_offset(instant: DateTime<Utc>) -> FixedOffset {
    let year = instant.year();
    let summer_starts = last_sunday_at_01_utc(year, 3);
    let summer_ends = last_sunday_at_01_utc(year, 10);
    let hours = if instant >= summer_starts && instant < summer_ends {
        2
    } else {
        1
    };
    FixedOffset::east_opt(hours * 3600).expect("offset horaire belge valide")
}

/// Mars et octobre comptent 31 jours, d'où le point de départ.
fn last_sunday_at_01_utc(year: i32, month: u32) -> DateTime<Utc> {
    for day in (25..=31).rev() {
        let date = NaiveDate::from_ymd_opt(year, month, day).expect("jour valide en mars/octobre");
        if date.weekday() == Weekday::Sun {
            return Utc.from_utc_datetime(&date.and_hms_opt(1, 0, 0).expect("01:00 valide"));
        }
    }
    unreachable!("une semaine contient toujours un dimanche")
}

/// Les noms de commerce, de panier et les descriptions sont saisis par les
/// marchands : ils sont interpolés dans le corps HTML, donc échappés.
fn escape_html(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal_macros::dec;
    use uuid::Uuid;

    use crate::domain::entities::ProductStatus;

    const BASE: &str = "https://derniere-chance.ecosolva.org";

    fn merchant(nom: &str) -> Merchant {
        Merchant {
            id: Uuid::nil(),
            nom: nom.into(),
            adresse: "Rue du Marché 4, 1000 Bruxelles".into(),
            categorie: "boulangerie".into(),
            note: None,
            email: "pro@example.org".into(),
            password_hash: String::new(),
            latitude: None,
            longitude: None,
            logo_url: None,
            created_at: Utc::now(),
            anonymise_le: None,
        }
    }

    fn product(nom: &str) -> Product {
        Product {
            id: Uuid::nil(),
            marchand_id: Uuid::nil(),
            nom: nom.into(),
            description: "Pains & viennoiseries du jour".into(),
            prix_initial: dec!(8.00),
            prix_demarque: dec!(3.20),
            quantite: 5,
            retrait_debut: Utc.with_ymd_and_hms(2026, 9, 2, 15, 30, 0).unwrap(),
            retrait_fin: Utc.with_ymd_and_hms(2026, 9, 2, 17, 0, 0).unwrap(),
            statut: ProductStatus::Publie,
            photo_url: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn subject_names_the_merchant_then_the_basket() {
        let email = new_offer_email(&merchant("Chez Léa"), &product("Panier surprise"), BASE);
        assert_eq!(email.subject, "Chez Léa : Panier surprise");
    }

    #[test]
    fn text_body_carries_price_discount_and_links() {
        let email = new_offer_email(&merchant("Chez Léa"), &product("Panier surprise"), BASE);

        assert!(
            email.text.contains("3,20 € au lieu de 8,00 € (-60 %)"),
            "{}",
            email.text
        );
        assert!(
            email.text.contains(
                "https://derniere-chance.ecosolva.org/offre?id=00000000-0000-0000-0000-000000000000"
            ),
            "{}",
            email.text
        );
        assert!(
            email
                .text
                .contains("https://derniere-chance.ecosolva.org/profil"),
            "{}",
            email.text
        );
    }

    #[test]
    fn merchant_input_is_escaped_in_the_html_body() {
        let email = new_offer_email(
            &merchant("<script>alert('x')</script>"),
            &product("Panier & thé"),
            BASE,
        );

        assert!(!email.html.contains("<script>"), "{}", email.html);
        assert!(email.html.contains("&lt;script&gt;"), "{}", email.html);
        assert!(email.html.contains("Panier &amp; thé"), "{}", email.html);
    }

    /// Une base d'URL avec barre oblique finale ne doit pas produire `//offre`.
    #[test]
    fn trailing_slash_in_the_base_url_never_doubles_the_separator() {
        let email = new_offer_email(
            &merchant("Chez Léa"),
            &product("Panier surprise"),
            "https://derniere-chance.ecosolva.org",
        );
        assert!(!email.text.contains("org//offre"), "{}", email.text);
    }

    /// Le message part parfois vers quelqu'un qui n'a rien demandé : il ne
    /// doit donc rien révéler du compte, pas même l'adresse visée.
    #[test]
    fn le_message_de_reinitialisation_ne_nomme_pas_le_compte() {
        let email = password_reset_email(
            "https://derniere-chance.ecosolva.org/mot-de-passe?token=abc",
            60,
        );

        assert!(email.text.contains("60 minutes"), "{}", email.text);
        assert!(
            email
                .text
                .contains("https://derniere-chance.ecosolva.org/mot-de-passe?token=abc"),
            "{}",
            email.text
        );
        assert!(
            email.text.contains("ignorez cet email"),
            "aucune consigne pour un destinataire qui n'a rien demandé : {}",
            email.text
        );
        assert!(
            !email.text.contains('@'),
            "adresse divulguée : {}",
            email.text
        );
    }

    #[test]
    fn pickup_window_is_shown_in_brussels_time() {
        // 15:30 UTC en septembre = 17:30 à Bruxelles (heure d'été).
        let summer = pickup_window(
            Utc.with_ymd_and_hms(2026, 9, 2, 15, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 9, 2, 17, 0, 0).unwrap(),
        );
        assert_eq!(summer, "le 02/09 entre 17:30 et 19:00");

        // 15:30 UTC en décembre = 16:30 à Bruxelles (heure d'hiver).
        let winter = pickup_window(
            Utc.with_ymd_and_hms(2026, 12, 2, 15, 30, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 12, 2, 17, 0, 0).unwrap(),
        );
        assert_eq!(winter, "le 02/12 entre 16:30 et 18:00");
    }

    #[test]
    fn pickup_window_spanning_midnight_shows_both_dates() {
        let window = pickup_window(
            Utc.with_ymd_and_hms(2026, 9, 2, 20, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2026, 9, 3, 6, 0, 0).unwrap(),
        );
        assert_eq!(window, "du 02/09 à 22:00 au 03/09 à 08:00");
    }

    /// Le basculement européen a lieu au dernier dimanche de mars à 01:00 UTC
    /// (29 mars 2026) et au dernier dimanche d'octobre (25 octobre 2026).
    #[test]
    fn daylight_saving_switches_on_the_european_dates() {
        let before = Utc.with_ymd_and_hms(2026, 3, 29, 0, 59, 0).unwrap();
        let after = Utc.with_ymd_and_hms(2026, 3, 29, 1, 0, 0).unwrap();
        assert_eq!(brussels_offset(before).local_minus_utc(), 3600);
        assert_eq!(brussels_offset(after).local_minus_utc(), 7200);

        let last_summer_instant = Utc.with_ymd_and_hms(2026, 10, 25, 0, 59, 0).unwrap();
        let first_winter_instant = Utc.with_ymd_and_hms(2026, 10, 25, 1, 0, 0).unwrap();
        assert_eq!(brussels_offset(last_summer_instant).local_minus_utc(), 7200);
        assert_eq!(
            brussels_offset(first_winter_instant).local_minus_utc(),
            3600
        );
    }
}
