use async_trait::async_trait;
use serde_json::{json, Value};

use super::message::{new_offer_email, password_reset_email, RenderedEmail};
use super::non_empty_env;
use crate::application::ports::{EmailError, EmailSender};
use crate::domain::entities::{Merchant, Product};

/// API d'envoi v3.1 de Mailjet (Mailjet SAS, infrastructure européenne).
const MAILJET_ENDPOINT: &str = "https://api.mailjet.com/v3.1/send";

/// Un envoi lent ne doit pas retarder la publication d'une démarque : le
/// fan-out aux abonnés est séquentiel, donc chaque appel est borné.
const SEND_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Adaptateur `EmailSender` sur l'API HTTP de Mailjet.
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
    /// `MAILJET_FROM_EMAIL` manque.
    pub fn from_env() -> Option<Self> {
        Some(Self {
            api_key: non_empty_env("MAILJET_API_KEY")?,
            secret_key: non_empty_env("MAILJET_SECRET_KEY")?,
            from_email: non_empty_env("MAILJET_FROM_EMAIL")?,
            from_name: non_empty_env("MAILJET_FROM_NAME")
                .unwrap_or_else(|| super::DEFAULT_FROM_NAME.to_string()),
            app_base_url: super::app_base_url(),
            endpoint: MAILJET_ENDPOINT.to_string(),
            client: reqwest::Client::builder()
                .timeout(SEND_TIMEOUT)
                .build()
                .expect("failed to build the Mailjet HTTP client"),
        })
    }

    fn new_offer_payload(&self, to_email: &str, merchant: &Merchant, product: &Product) -> Value {
        self.payload(
            to_email,
            new_offer_email(merchant, product, &self.app_base_url),
        )
    }

    /// Poste un message et traduit la réponse en `Result`.
    async fn post(&self, payload: Value) -> Result<(), EmailError> {
        let response = self
            .client
            .post(&self.endpoint)
            .basic_auth(&self.api_key, Some(&self.secret_key))
            .json(&payload)
            .send()
            .await
            .map_err(|err| EmailError::SendFailed(err.to_string()))?;

        let status = response.status();
        // Le corps détaille la cause (expéditeur non validé, clé sans droit,
        // compte suspendu...). Sans lui, un 401 est indébogable depuis les
        // journaux.
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

    fn payload(&self, to_email: &str, email: RenderedEmail) -> Value {
        json!({
            "Messages": [{
                "From": { "Email": self.from_email, "Name": self.from_name },
                "To": [{ "Email": to_email }],
                "Subject": email.subject,
                "TextPart": email.text,
                "HTMLPart": email.html,
                // Pas de pixel d'ouverture, pas de réécriture des liens.
                "TrackOpens": "disabled",
                "TrackClicks": "disabled",
            }],
        })
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
        self.post(self.new_offer_payload(to_email, merchant, product))
            .await
    }

    async fn send_password_reset(
        &self,
        to_email: &str,
        reset_url: &str,
        expires_in_minutes: i64,
    ) -> Result<(), EmailError> {
        let payload = self.payload(
            to_email,
            password_reset_email(reset_url, expires_in_minutes),
        );
        self.post(payload).await
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

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{TimeZone, Utc};
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    use crate::domain::entities::ProductStatus;

    fn sender() -> MailjetEmailSender {
        MailjetEmailSender {
            api_key: "cle-publique".into(),
            secret_key: "cle-privee".into(),
            from_email: "contact@derniere-chance.ecosolva.org".into(),
            from_name: "DernièreChance".into(),
            app_base_url: "https://derniere-chance.ecosolva.org".into(),
            endpoint: MAILJET_ENDPOINT.into(),
            client: reqwest::Client::new(),
        }
    }

    fn merchant() -> Merchant {
        Merchant {
            id: Uuid::nil(),
            nom: "Chez Léa".into(),
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

    fn product() -> Product {
        Product {
            id: Uuid::nil(),
            marchand_id: Uuid::nil(),
            nom: "Panier surprise".into(),
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
        let payload = sender().new_offer_payload("abonne@example.org", &merchant(), &product());
        let message = &payload["Messages"][0];

        assert_eq!(message["To"][0]["Email"], "abonne@example.org");
        assert_eq!(
            message["From"]["Email"],
            "contact@derniere-chance.ecosolva.org"
        );
        assert_eq!(message["Subject"], "Chez Léa : Panier surprise");
        assert!(message["TextPart"].is_string());
        assert!(message["HTMLPart"].is_string());
        assert_eq!(message["TrackOpens"], "disabled");
        assert_eq!(message["TrackClicks"], "disabled");
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
}
