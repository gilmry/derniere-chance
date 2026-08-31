use async_trait::async_trait;
use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, TimeZone, Utc, Weekday};
use rust_decimal::Decimal;
use serde_json::{json, Value};

use crate::application::ports::{EmailError, EmailSender};
use crate::domain::entities::{Merchant, Product};
use crate::domain::services::pricing::discount_percent;

/// API d'envoi v3.1 de Mailjet (Mailjet SAS, infrastructure européenne : pas
/// de transfert hors UE à déclarer au registre des traitements).
const MAILJET_ENDPOINT: &str = "https://api.mailjet.com/v3.1/send";

/// Un envoi lent ne doit pas retarder la publication d'une démarque : le
/// fan-out aux abonnés est séquentiel, donc chaque appel est borné.
const SEND_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Adaptateur `EmailSender` réel, sur Mailjet.
///
/// Construit par [`MailjetEmailSender::from_env`], qui renvoie `None` quand la
/// configuration est absente : `main` retombe alors sur
/// [`super::LoggingEmailSender`], pour qu'un poste de dev, la CI et les e2e
/// tournent sans clé et sans écrire à de vraies personnes.
///
/// Le suivi d'ouverture et la réécriture des liens sont explicitement
/// désactivés : ce sont des traceurs, et le registre des traitements annonce
/// l'absence d'outil de mesure d'audience.
pub struct MailjetEmailSender {
    api_key: String,
    secret_key: String,
    from_email: String,
    from_name: String,
    app_base_url: String,
    endpoint: String,
    client: reqwest::Client,
}

impl MailjetEmailSender {
    /// `None` si `MAILJET_API_KEY`, `MAILJET_SECRET_KEY` ou
    /// `MAILJET_FROM_EMAIL` manque. Les valeurs vides comptent comme absentes :
    /// docker-compose passe `${MAILJET_API_KEY:-}`, donc une variable non
    /// renseignée arrive comme chaîne vide plutôt qu'absente.
    pub fn from_env() -> Option<Self> {
        let api_key = non_empty_env("MAILJET_API_KEY")?;
        let secret_key = non_empty_env("MAILJET_SECRET_KEY")?;
        let from_email = non_empty_env("MAILJET_FROM_EMAIL")?;

        Some(Self {
            api_key,
            secret_key,
            from_email,
            from_name: non_empty_env("MAILJET_FROM_NAME")
                .unwrap_or_else(|| "DernièreChance".to_string()),
            app_base_url: non_empty_env("APP_BASE_URL")
                .unwrap_or_else(|| "https://derniere-chance.ecosolva.org".to_string())
                .trim_end_matches('/')
                .to_string(),
            endpoint: MAILJET_ENDPOINT.to_string(),
            client: reqwest::Client::builder()
                .timeout(SEND_TIMEOUT)
                .build()
                .expect("failed to build the Mailjet HTTP client"),
        })
    }

    fn offer_url(&self, product: &Product) -> String {
        format!("{}/offre?id={}", self.app_base_url, product.id)
    }

    fn profile_url(&self) -> String {
        format!("{}/profil", self.app_base_url)
    }

    fn new_offer_payload(&self, to_email: &str, merchant: &Merchant, product: &Product) -> Value {
        json!({
            "Messages": [{
                "From": { "Email": self.from_email, "Name": self.from_name },
                "To": [{ "Email": to_email }],
                "Subject": format!("{} : {}", merchant.nom, product.nom),
                "TextPart": self.new_offer_text(merchant, product),
                "HTMLPart": self.new_offer_html(merchant, product),
                // Pas de pixel d'ouverture, pas de réécriture des liens.
                "TrackOpens": "disabled",
                "TrackClicks": "disabled",
            }],
        })
    }

    fn new_offer_text(&self, merchant: &Merchant, product: &Product) -> String {
        format!(
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
            lien = self.offer_url(product),
            profil = self.profile_url(),
        )
    }

    fn new_offer_html(&self, merchant: &Merchant, product: &Product) -> String {
        format!(
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
            lien = escape_html(&self.offer_url(product)),
            profil = escape_html(&self.profile_url()),
        )
    }
}

#[async_trait]
impl EmailSender for MailjetEmailSender {
    async fn send_new_offer_notification(
        &self,
        to_email: &str,
        merchant: &Merchant,
        product: &Product,
    ) -> Result<(), EmailError> {
        let response = self
            .client
            .post(&self.endpoint)
            .basic_auth(&self.api_key, Some(&self.secret_key))
            .json(&self.new_offer_payload(to_email, merchant, product))
            .send()
            .await
            .map_err(|err| EmailError::SendFailed(err.to_string()))?;

        let status = response.status();
        // Le corps détaille la cause (expéditeur non validé, clé sans droit,
        // adresse invalide...). Sans lui, un 401 ou un refus par message est
        // indébogable depuis les journaux.
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(EmailError::SendFailed(format!(
                "mailjet returned {status}: {body}"
            )));
        }

        // Mailjet v3.1 peut répondre 200 tout en refusant le message : le
        // verdict par destinataire est dans `Messages[].Status`, pas dans le
        // code HTTP. Sans cette vérification, un envoi refusé serait journalisé
        // comme `Envoyee`.
        ensure_all_messages_succeeded(&body)
    }
}

fn ensure_all_messages_succeeded(body: &str) -> Result<(), EmailError> {
    let parsed: Value = serde_json::from_str(body)
        .map_err(|err| EmailError::SendFailed(format!("réponse mailjet illisible: {err}")))?;

    let Some(messages) = parsed.get("Messages").and_then(Value::as_array) else {
        return Err(EmailError::SendFailed(format!(
            "réponse mailjet sans champ Messages: {body}"
        )));
    };

    if messages
        .iter()
        .all(|message| message.get("Status").and_then(Value::as_str) == Some("success"))
    {
        return Ok(());
    }

    Err(EmailError::SendFailed(format!(
        "mailjet a refusé le message: {body}"
    )))
}

fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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

    fn sender() -> MailjetEmailSender {
        MailjetEmailSender {
            api_key: "cle-publique".into(),
            secret_key: "cle-privee".into(),
            from_email: "alertes@derniere-chance.ecosolva.org".into(),
            from_name: "DernièreChance".into(),
            app_base_url: "https://derniere-chance.ecosolva.org".into(),
            endpoint: MAILJET_ENDPOINT.into(),
            client: reqwest::Client::new(),
        }
    }

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
    fn payload_targets_the_subscriber_and_disables_tracking() {
        let payload = sender().new_offer_payload(
            "abonne@example.org",
            &merchant("Chez Léa"),
            &product("Panier surprise"),
        );
        let message = &payload["Messages"][0];

        assert_eq!(message["To"][0]["Email"], "abonne@example.org");
        assert_eq!(
            message["From"]["Email"],
            "alertes@derniere-chance.ecosolva.org"
        );
        assert_eq!(message["Subject"], "Chez Léa : Panier surprise");
        assert!(message["TextPart"].is_string());
        assert!(message["HTMLPart"].is_string());
        assert_eq!(message["TrackOpens"], "disabled");
        assert_eq!(message["TrackClicks"], "disabled");
    }

    #[test]
    fn text_body_carries_price_discount_and_links() {
        let body = sender().new_offer_text(&merchant("Chez Léa"), &product("Panier surprise"));

        assert!(body.contains("3,20 € au lieu de 8,00 € (-60 %)"), "{body}");
        assert!(
            body.contains("https://derniere-chance.ecosolva.org/offre?id=00000000-0000-0000-0000-000000000000"),
            "{body}"
        );
        assert!(
            body.contains("https://derniere-chance.ecosolva.org/profil"),
            "{body}"
        );
    }

    #[test]
    fn merchant_input_is_escaped_in_the_html_body() {
        let html = sender().new_offer_html(
            &merchant("<script>alert('x')</script>"),
            &product("Panier & thé"),
        );

        assert!(!html.contains("<script>"), "{html}");
        assert!(html.contains("&lt;script&gt;"), "{html}");
        assert!(html.contains("Panier &amp; thé"), "{html}");
    }

    /// Mailjet répond 200 même quand il refuse un destinataire : le verdict
    /// est par message, sans quoi un refus serait journalisé comme `Envoyee`.
    #[test]
    fn a_refused_message_is_an_error_even_on_http_200() {
        let refused = r#"{"Messages":[{"Status":"error","Errors":[
            {"ErrorMessage":"\"alertes@exemple.org\" is an invalid email address."}]}]}"#;
        assert!(ensure_all_messages_succeeded(refused).is_err());

        let accepted = r#"{"Messages":[{"Status":"success","To":[
            {"Email":"abonne@example.org","MessageID":1}]}]}"#;
        assert!(ensure_all_messages_succeeded(accepted).is_ok());
    }

    #[test]
    fn an_unparsable_response_is_an_error() {
        assert!(ensure_all_messages_succeeded("<html>502 Bad Gateway</html>").is_err());
        assert!(ensure_all_messages_succeeded(r#"{"ErrorMessage":"API key invalid"}"#).is_err());
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
