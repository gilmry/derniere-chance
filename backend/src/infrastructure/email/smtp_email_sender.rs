use async_trait::async_trait;
use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use super::message::{new_offer_email, password_reset_email, RenderedEmail};
use super::non_empty_env;
use crate::application::ports::{EmailError, EmailSender};
use crate::domain::entities::{Merchant, Product};

/// Un envoi lent ne doit pas retarder la publication d'une démarque : le
/// fan-out aux abonnés est séquentiel, donc chaque appel est borné.
const SEND_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

/// Adaptateur `EmailSender` sur un relais SMTP quelconque (Proton, OVH,
/// Scaleway, Mailjet en mode SMTP...).
///
/// Il existe à côté de l'adaptateur Mailjet pour que la plateforme ne dépende
/// pas d'un fournisseur unique : le corps de l'email est rendu par
/// `super::message`, donc changer de transporteur ne change rien à ce que le
/// destinataire lit.
pub struct SmtpEmailSender {
    from: Mailbox,
    app_base_url: String,
    transport: AsyncSmtpTransport<Tokio1Executor>,
}

/// Comment la connexion est chiffrée. Le clair n'est pas une option : ces
/// messages portent l'adresse d'un testeur et le jeton du relais.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Encryption {
    /// Connexion en clair puis `STARTTLS`, le cas courant sur le port 587.
    Starttls,
    /// TLS dès l'ouverture de la connexion, le cas courant sur le port 465.
    Implicit,
}

impl SmtpEmailSender {
    /// `None` si `SMTP_SERVEUR`, `SMTP_USER` ou `SMTP_TOKEN` manque.
    ///
    /// `SMTP_FROM_EMAIL` permet d'envoyer depuis une adresse différente du
    /// compte d'authentification ; par défaut l'expéditeur est `SMTP_USER`,
    /// ce que la plupart des relais imposent de toute façon.
    pub fn from_env() -> Option<Self> {
        let server = non_empty_env("SMTP_SERVEUR").or_else(|| non_empty_env("SMTP_SERVER"))?;
        let user = non_empty_env("SMTP_USER")?;
        let token = non_empty_env("SMTP_TOKEN").or_else(|| non_empty_env("SMTP_PASSWORD"))?;

        let port = non_empty_env("SMTP_PORT")
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(587);
        let encryption = encryption(non_empty_env("SMTP_PROTOCOL").as_deref(), port);

        let from_email = non_empty_env("SMTP_FROM_EMAIL").unwrap_or_else(|| user.clone());
        let from_name =
            non_empty_env("SMTP_FROM_NAME").unwrap_or_else(|| super::DEFAULT_FROM_NAME.to_string());
        let from = match format!("{from_name} <{from_email}>").parse::<Mailbox>() {
            Ok(mailbox) => mailbox,
            Err(err) => {
                tracing::error!(?err, %from_email, "adresse d'expédition SMTP invalide");
                return None;
            }
        };

        let builder = match encryption {
            Encryption::Starttls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&server),
            Encryption::Implicit => AsyncSmtpTransport::<Tokio1Executor>::relay(&server),
        };
        let builder = match builder {
            Ok(builder) => builder,
            Err(err) => {
                tracing::error!(?err, %server, "relais SMTP inutilisable");
                return None;
            }
        };

        tracing::info!(%server, port, ?encryption, "relais SMTP configuré");

        Some(Self {
            from,
            app_base_url: super::app_base_url(),
            transport: builder
                .port(port)
                .credentials(Credentials::new(user, token))
                .timeout(Some(SEND_TIMEOUT))
                .build(),
        })
    }
}

impl SmtpEmailSender {
    async fn send(&self, to_email: &str, rendered: RenderedEmail) -> Result<(), EmailError> {
        let to: Mailbox = to_email
            .parse()
            .map_err(|err| EmailError::SendFailed(format!("adresse invalide {to_email}: {err}")))?;

        let message = Message::builder()
            .from(self.from.clone())
            .to(to)
            .subject(rendered.subject)
            .multipart(MultiPart::alternative_plain_html(
                rendered.text,
                rendered.html,
            ))
            .map_err(|err| EmailError::SendFailed(format!("message mal formé: {err}")))?;

        // Le message d'erreur du relais est recopié tel quel : c'est lui qui
        // distingue un jeton refusé d'un destinataire rejeté ou d'un quota
        // dépassé, et sans lui la panne est indébogable depuis les journaux.
        self.transport
            .send(message)
            .await
            .map_err(|err| EmailError::SendFailed(format!("smtp: {err}")))?;

        Ok(())
    }
}

#[async_trait]
impl EmailSender for SmtpEmailSender {
    async fn send_new_offer_notification(
        &self,
        to_email: &str,
        merchant: &Merchant,
        product: &Product,
    ) -> Result<(), EmailError> {
        self.send(
            to_email,
            new_offer_email(merchant, product, &self.app_base_url),
        )
        .await
    }

    async fn send_password_reset(
        &self,
        to_email: &str,
        reset_url: &str,
        expires_in_minutes: i64,
    ) -> Result<(), EmailError> {
        self.send(
            to_email,
            password_reset_email(reset_url, expires_in_minutes),
        )
        .await
    }
}

/// Les fournisseurs étiquettent ce réglage de façon incohérente : Proton
/// annonce « TLS/SSL » sur le port 587, qui est pourtant du STARTTLS. On ne
/// tranche donc sur `SMTP_PROTOCOL` que lorsqu'il est explicite, et on se fie
/// au port sinon, seul indicateur fiable en pratique.
fn encryption(protocol: Option<&str>, port: u16) -> Encryption {
    let declared = protocol.map(str::to_ascii_lowercase);
    match declared.as_deref() {
        Some(value) if value.contains("starttls") => Encryption::Starttls,
        Some(value) if value.contains("smtps") || value.contains("implicit") => {
            Encryption::Implicit
        }
        _ if port == 465 => Encryption::Implicit,
        _ => Encryption::Starttls,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_explicit_protocol_wins() {
        assert_eq!(encryption(Some("STARTTLS"), 465), Encryption::Starttls);
        assert_eq!(encryption(Some("smtps"), 587), Encryption::Implicit);
    }

    /// Le cas Proton : le port 587 étiqueté « TLS/SSL » est du STARTTLS.
    /// Suivre l'étiquette ouvrirait une connexion TLS implicite sur un port
    /// qui attend du clair, et chaque envoi échouerait.
    #[test]
    fn an_ambiguous_label_falls_back_to_the_port() {
        assert_eq!(encryption(Some("TLS/SSL"), 587), Encryption::Starttls);
        assert_eq!(encryption(Some("TLS/SSL"), 465), Encryption::Implicit);
        assert_eq!(encryption(Some("SSL"), 465), Encryption::Implicit);
    }

    #[test]
    fn without_a_protocol_the_port_decides() {
        assert_eq!(encryption(None, 587), Encryption::Starttls);
        assert_eq!(encryption(None, 465), Encryption::Implicit);
        assert_eq!(encryption(None, 2525), Encryption::Starttls);
    }
}
