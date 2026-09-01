use std::sync::Arc;

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::application::ports::{
    ConsumerRepository, EmailSender, MerchantRepository, NewPasswordReset, PasswordResetRepository,
    RepoError,
};
use crate::domain::entities::AccountSubject;
use crate::domain::services::password;

#[derive(Debug, Error)]
pub enum PasswordResetError {
    #[error("invalid or expired token")]
    InvalidToken,
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("password hashing failed")]
    HashingFailed,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

/// Durée de validité d'un lien. Assez court pour qu'un email oublié dans une
/// boîte cesse vite d'être une clé, assez long pour laisser le temps d'aller
/// chercher le message.
const LINK_LIFETIME_MINUTES: i64 = 60;

/// Sous ce délai, une nouvelle demande ne réexpédie pas d'email.
///
/// Sans ce garde-fou, n'importe qui pourrait faire pleuvoir des emails sur
/// l'adresse d'un tiers en rejouant le formulaire, et l'application n'a pas
/// de limitation de débit ailleurs.
const RESEND_COOLDOWN_MINUTES: i64 = 2;

/// Réinitialisation de mot de passe par email, pour les deux principaux
/// auto-inscrits.
///
/// Un compte anonymisé après retrait du consentement reste hors de portée :
/// son adresse a été remplacée par une valeur dérivée de son identifiant,
/// donc aucune recherche par email ne le retrouve, et `update_password`
/// refuse de le toucher en base.
pub struct PasswordResetUseCases {
    consumer_repo: Arc<dyn ConsumerRepository>,
    merchant_repo: Arc<dyn MerchantRepository>,
    reset_repo: Arc<dyn PasswordResetRepository>,
    email_sender: Arc<dyn EmailSender>,
    app_base_url: String,
}

impl PasswordResetUseCases {
    pub fn new(
        consumer_repo: Arc<dyn ConsumerRepository>,
        merchant_repo: Arc<dyn MerchantRepository>,
        reset_repo: Arc<dyn PasswordResetRepository>,
        email_sender: Arc<dyn EmailSender>,
        app_base_url: String,
    ) -> Self {
        Self {
            consumer_repo,
            merchant_repo,
            reset_repo,
            email_sender,
            app_base_url: app_base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Demande d'un lien de réinitialisation.
    ///
    /// Répond `Ok` quoi qu'il arrive, y compris pour une adresse inconnue :
    /// distinguer les deux cas transformerait ce formulaire en oracle
    /// permettant de savoir qui est inscrit. Les échecs réels partent dans
    /// les journaux.
    pub async fn request(&self, email: &str) -> Result<(), PasswordResetError> {
        let email = email.trim();

        let Some(subject) = self.find_account(email).await? else {
            tracing::info!("demande de réinitialisation pour une adresse inconnue");
            return Ok(());
        };

        if let Some(last) = self.reset_repo.last_request_at(subject).await? {
            if Utc::now() - last < Duration::minutes(RESEND_COOLDOWN_MINUTES) {
                tracing::info!(
                    role = subject.role(),
                    "demande de réinitialisation ignorée, un lien vient d'être envoyé"
                );
                return Ok(());
            }
        }

        let token = generate_token();
        self.reset_repo
            .create(NewPasswordReset {
                subject,
                token_hash: hash_token(&token),
                expires_at: Utc::now() + Duration::minutes(LINK_LIFETIME_MINUTES),
            })
            .await?;

        let reset_url = format!("{}/mot-de-passe?token={}", self.app_base_url, token);
        if let Err(err) = self
            .email_sender
            .send_password_reset(email, &reset_url, LINK_LIFETIME_MINUTES)
            .await
        {
            // Journalisé sans être remonté : renvoyer une erreur ici et un
            // succès pour une adresse inconnue rendrait les deux cas
            // distinguables, ce que tout le reste de cette méthode cherche à
            // éviter.
            tracing::error!(
                ?err,
                role = subject.role(),
                "envoi du lien de réinitialisation échoué"
            );
        }

        Ok(())
    }

    /// Consomme le lien et pose le nouveau mot de passe.
    pub async fn confirm(&self, token: &str, new_password: &str) -> Result<(), PasswordResetError> {
        // Le mot de passe est validé avant de consommer le jeton : sinon un
        // mot de passe trop court brûlerait le lien, et la personne devrait
        // tout recommencer pour une faute de saisie.
        password::validate(new_password).map_err(PasswordResetError::InvalidInput)?;

        let subject = self
            .reset_repo
            .consume(&hash_token(token))
            .await?
            .ok_or(PasswordResetError::InvalidToken)?;

        let password_hash = bcrypt::hash(new_password, bcrypt::DEFAULT_COST)
            .map_err(|_| PasswordResetError::HashingFailed)?;

        match subject {
            AccountSubject::Consumer(id) => {
                self.consumer_repo
                    .update_password(id, &password_hash)
                    .await?
            }
            AccountSubject::Merchant(id) => {
                self.merchant_repo
                    .update_password(id, &password_hash)
                    .await?
            }
        }

        // Les autres liens en cours tombent : celui d'une demande faite par
        // un tiers ne doit pas rester ouvert derrière un changement réussi.
        self.reset_repo.invalidate_all(subject).await?;

        tracing::info!(role = subject.role(), "mot de passe réinitialisé");
        Ok(())
    }

    /// Consommateur d'abord, marchand ensuite. Les deux tables ont chacune une
    /// contrainte d'unicité sur l'email, et rien n'interdit qu'une même
    /// personne ait les deux comptes ; le compte client passe en premier
    /// parce que c'est le cas de loin le plus fréquent.
    async fn find_account(
        &self,
        email: &str,
    ) -> Result<Option<AccountSubject>, PasswordResetError> {
        if let Some(consumer) = self.consumer_repo.find_by_email(email).await? {
            return Ok(Some(AccountSubject::Consumer(consumer.id)));
        }
        if let Some(merchant) = self.merchant_repo.find_by_email(email).await? {
            return Ok(Some(AccountSubject::Merchant(merchant.id)));
        }
        Ok(None)
    }
}

/// 256 bits d'aléa tirés de deux UUID v4, concaténés en hexadécimal puis
/// encodés pour l'URL : même procédé que les jetons OAuth, ce qui évite
/// d'ajouter une dépendance à `rand` pour ce seul usage.
fn generate_token() -> String {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(Uuid::new_v4().as_bytes());
    bytes.extend_from_slice(Uuid::new_v4().as_bytes());
    URL_SAFE_NO_PAD.encode(bytes)
}

/// La base ne garde que cette empreinte : une fuite de la table ne doit pas
/// suffire à ouvrir des comptes.
fn hash_token(token: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Mutex;

    use chrono::DateTime;

    use crate::application::ports::{
        EmailError, MerchantUpdate, NewConsumer, NewMerchant, RepoError,
    };
    use crate::domain::entities::{Consumer, Merchant, Product};

    fn consumer(email: &str) -> Consumer {
        Consumer {
            id: Uuid::new_v4(),
            email: email.into(),
            password_hash: bcrypt::hash("ancien mot de passe", 4).unwrap(),
            created_at: Utc::now(),
            anonymise_le: None,
        }
    }

    fn merchant(email: &str) -> Merchant {
        Merchant {
            id: Uuid::new_v4(),
            nom: "Chez Léa".into(),
            adresse: "Rue du Marché 4, 1000 Bruxelles".into(),
            categorie: "boulangerie".into(),
            note: None,
            email: email.into(),
            password_hash: bcrypt::hash("ancien mot de passe", 4).unwrap(),
            latitude: None,
            longitude: None,
            logo_url: None,
            created_at: Utc::now(),
            anonymise_le: None,
        }
    }

    #[derive(Default)]
    struct FakeConsumerRepo {
        rows: Mutex<Vec<Consumer>>,
        passwords: Mutex<Vec<(Uuid, String)>>,
    }

    #[async_trait::async_trait]
    impl ConsumerRepository for FakeConsumerRepo {
        async fn create(&self, _new: NewConsumer) -> Result<Consumer, RepoError> {
            unimplemented!("hors du périmètre de la réinitialisation")
        }
        async fn find_by_email(&self, email: &str) -> Result<Option<Consumer>, RepoError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|row| row.email == email)
                .cloned())
        }
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<Consumer>, RepoError> {
            Ok(None)
        }
        async fn list_all(&self) -> Result<Vec<Consumer>, RepoError> {
            Ok(vec![])
        }
        async fn delete(&self, _id: Uuid) -> Result<(), RepoError> {
            Ok(())
        }
        async fn anonymize(&self, _id: Uuid) -> Result<(), RepoError> {
            Ok(())
        }
        async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<(), RepoError> {
            self.passwords
                .lock()
                .unwrap()
                .push((id, password_hash.to_string()));
            Ok(())
        }
        async fn count(&self) -> Result<i64, RepoError> {
            Ok(0)
        }
    }

    #[derive(Default)]
    struct FakeMerchantRepo {
        rows: Mutex<Vec<Merchant>>,
        passwords: Mutex<Vec<(Uuid, String)>>,
    }

    #[async_trait::async_trait]
    impl MerchantRepository for FakeMerchantRepo {
        async fn create(&self, _new: NewMerchant) -> Result<Merchant, RepoError> {
            unimplemented!("hors du périmètre de la réinitialisation")
        }
        async fn find_by_email(&self, email: &str) -> Result<Option<Merchant>, RepoError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|row| row.email == email)
                .cloned())
        }
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<Merchant>, RepoError> {
            Ok(None)
        }
        async fn list_all(&self) -> Result<Vec<Merchant>, RepoError> {
            Ok(vec![])
        }
        async fn delete(&self, _id: Uuid) -> Result<(), RepoError> {
            Ok(())
        }
        async fn anonymize(&self, _id: Uuid) -> Result<(), RepoError> {
            Ok(())
        }
        async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<(), RepoError> {
            self.passwords
                .lock()
                .unwrap()
                .push((id, password_hash.to_string()));
            Ok(())
        }
        async fn count(&self) -> Result<i64, RepoError> {
            Ok(0)
        }
        async fn update_logo(&self, _id: Uuid, _logo_url: &str) -> Result<Merchant, RepoError> {
            unimplemented!("hors du périmètre de la réinitialisation")
        }
        async fn update(&self, _id: Uuid, _changes: MerchantUpdate) -> Result<Merchant, RepoError> {
            unimplemented!("hors du périmètre de la réinitialisation")
        }
    }

    struct StoredReset {
        token_hash: String,
        subject: AccountSubject,
        expires_at: DateTime<Utc>,
        used: bool,
        created_at: DateTime<Utc>,
    }

    /// Reproduit les invariants tenus par la base : usage unique, expiration,
    /// et invalidation en masse par compte.
    #[derive(Default)]
    struct FakeResetRepo {
        rows: Mutex<Vec<StoredReset>>,
        /// Décalage appliqué à `created_at` des lignes insérées, pour simuler
        /// une demande plus ancienne sans faire attendre le test.
        backdate_minutes: Mutex<i64>,
    }

    impl FakeResetRepo {
        fn active_count(&self) -> usize {
            self.rows.lock().unwrap().iter().filter(|r| !r.used).count()
        }

        fn expire_all(&self) {
            for row in self.rows.lock().unwrap().iter_mut() {
                row.expires_at = Utc::now() - Duration::minutes(1);
            }
        }
    }

    #[async_trait::async_trait]
    impl PasswordResetRepository for FakeResetRepo {
        async fn create(&self, new: NewPasswordReset) -> Result<(), RepoError> {
            let backdate = *self.backdate_minutes.lock().unwrap();
            self.rows.lock().unwrap().push(StoredReset {
                token_hash: new.token_hash,
                subject: new.subject,
                expires_at: new.expires_at,
                used: false,
                created_at: Utc::now() - Duration::minutes(backdate),
            });
            Ok(())
        }

        async fn consume(&self, token_hash: &str) -> Result<Option<AccountSubject>, RepoError> {
            let mut rows = self.rows.lock().unwrap();
            let Some(row) = rows.iter_mut().find(|row| {
                row.token_hash == token_hash && !row.used && row.expires_at > Utc::now()
            }) else {
                return Ok(None);
            };
            row.used = true;
            Ok(Some(row.subject))
        }

        async fn invalidate_all(&self, subject: AccountSubject) -> Result<u64, RepoError> {
            let mut touched = 0;
            for row in self.rows.lock().unwrap().iter_mut() {
                if row.subject == subject && !row.used {
                    row.used = true;
                    touched += 1;
                }
            }
            Ok(touched)
        }

        async fn last_request_at(
            &self,
            subject: AccountSubject,
        ) -> Result<Option<DateTime<Utc>>, RepoError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .filter(|row| row.subject == subject && !row.used && row.expires_at > Utc::now())
                .map(|row| row.created_at)
                .max())
        }
    }

    #[derive(Default)]
    struct FakeEmailSender {
        sent: Mutex<Vec<(String, String)>>,
        fail: bool,
    }

    #[async_trait::async_trait]
    impl EmailSender for FakeEmailSender {
        async fn send_new_offer_notification(
            &self,
            _to_email: &str,
            _merchant: &Merchant,
            _product: &Product,
        ) -> Result<(), EmailError> {
            unimplemented!("hors du périmètre de la réinitialisation")
        }

        async fn send_password_reset(
            &self,
            to_email: &str,
            reset_url: &str,
            _expires_in_minutes: i64,
        ) -> Result<(), EmailError> {
            if self.fail {
                return Err(EmailError::SendFailed("relais injoignable".into()));
            }
            self.sent
                .lock()
                .unwrap()
                .push((to_email.to_string(), reset_url.to_string()));
            Ok(())
        }
    }

    struct Harness {
        use_cases: PasswordResetUseCases,
        consumers: Arc<FakeConsumerRepo>,
        merchants: Arc<FakeMerchantRepo>,
        resets: Arc<FakeResetRepo>,
        emails: Arc<FakeEmailSender>,
    }

    fn harness_with(email_fails: bool) -> Harness {
        let consumers = Arc::new(FakeConsumerRepo::default());
        let merchants = Arc::new(FakeMerchantRepo::default());
        let resets = Arc::new(FakeResetRepo::default());
        let emails = Arc::new(FakeEmailSender {
            sent: Mutex::new(vec![]),
            fail: email_fails,
        });

        Harness {
            use_cases: PasswordResetUseCases::new(
                consumers.clone(),
                merchants.clone(),
                resets.clone(),
                emails.clone(),
                "https://derniere-chance.ecosolva.org/".into(),
            ),
            consumers,
            merchants,
            resets,
            emails,
        }
    }

    fn harness() -> Harness {
        harness_with(false)
    }

    /// Extrait le jeton du lien envoyé, comme le ferait le navigateur.
    fn token_from_last_email(h: &Harness) -> String {
        let sent = h.emails.sent.lock().unwrap();
        let (_, url) = sent.last().expect("aucun email envoyé");
        url.rsplit_once("token=")
            .expect("lien sans jeton")
            .1
            .to_string()
    }

    // --- Demande ---

    /// Le formulaire ne doit pas dire qui est inscrit : une adresse inconnue
    /// répond comme une adresse connue, sans rien écrire ni envoyer.
    #[tokio::test]
    async fn une_adresse_inconnue_repond_comme_une_adresse_connue() {
        let h = harness();

        h.use_cases.request("inconnu@example.org").await.unwrap();

        assert_eq!(h.resets.rows.lock().unwrap().len(), 0);
        assert_eq!(h.emails.sent.lock().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn un_compte_client_recoit_un_lien_portant_un_jeton() {
        let h = harness();
        h.consumers
            .rows
            .lock()
            .unwrap()
            .push(consumer("abonne@example.org"));

        h.use_cases.request("abonne@example.org").await.unwrap();

        let sent = h.emails.sent.lock().unwrap();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].0, "abonne@example.org");
        assert!(
            sent[0]
                .1
                .starts_with("https://derniere-chance.ecosolva.org/mot-de-passe?token="),
            "{}",
            sent[0].1
        );
        assert_eq!(h.resets.active_count(), 1);
    }

    /// Le jeton en clair ne doit exister que dans l'email : la base n'en
    /// garde qu'une empreinte.
    #[tokio::test]
    async fn le_jeton_en_clair_n_est_jamais_stocke() {
        let h = harness();
        h.consumers
            .rows
            .lock()
            .unwrap()
            .push(consumer("abonne@example.org"));
        h.use_cases.request("abonne@example.org").await.unwrap();

        let token = token_from_last_email(&h);
        let rows = h.resets.rows.lock().unwrap();
        assert_ne!(rows[0].token_hash, token);
        assert_eq!(rows[0].token_hash, hash_token(&token));
    }

    #[tokio::test]
    async fn un_compte_marchand_est_trouve_aussi() {
        let h = harness();
        h.merchants
            .rows
            .lock()
            .unwrap()
            .push(merchant("pro@example.org"));

        h.use_cases.request("pro@example.org").await.unwrap();

        assert_eq!(h.emails.sent.lock().unwrap().len(), 1);
        assert!(matches!(
            h.resets.rows.lock().unwrap()[0].subject,
            AccountSubject::Merchant(_)
        ));
    }

    /// Sans ce garde-fou, rejouer le formulaire ferait pleuvoir des emails sur
    /// l'adresse d'un tiers.
    #[tokio::test]
    async fn deux_demandes_rapprochees_n_envoient_qu_un_email() {
        let h = harness();
        h.consumers
            .rows
            .lock()
            .unwrap()
            .push(consumer("abonne@example.org"));

        h.use_cases.request("abonne@example.org").await.unwrap();
        h.use_cases.request("abonne@example.org").await.unwrap();

        assert_eq!(h.emails.sent.lock().unwrap().len(), 1);
        assert_eq!(h.resets.rows.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn passe_le_delai_une_nouvelle_demande_renvoie_un_lien() {
        let h = harness();
        h.consumers
            .rows
            .lock()
            .unwrap()
            .push(consumer("abonne@example.org"));

        *h.resets.backdate_minutes.lock().unwrap() = RESEND_COOLDOWN_MINUTES + 1;
        h.use_cases.request("abonne@example.org").await.unwrap();
        *h.resets.backdate_minutes.lock().unwrap() = 0;
        h.use_cases.request("abonne@example.org").await.unwrap();

        assert_eq!(h.emails.sent.lock().unwrap().len(), 2);
    }

    /// Remonter l'échec d'envoi rendrait à nouveau distinguables l'adresse
    /// connue et l'inconnue.
    #[tokio::test]
    async fn un_envoi_qui_echoue_ne_remonte_pas_l_erreur() {
        let h = harness_with(true);
        h.consumers
            .rows
            .lock()
            .unwrap()
            .push(consumer("abonne@example.org"));

        assert!(h.use_cases.request("abonne@example.org").await.is_ok());
    }

    // --- Confirmation ---

    #[tokio::test]
    async fn le_lien_pose_le_nouveau_mot_de_passe() {
        let h = harness();
        let compte = consumer("abonne@example.org");
        let id = compte.id;
        h.consumers.rows.lock().unwrap().push(compte);
        h.use_cases.request("abonne@example.org").await.unwrap();
        let token = token_from_last_email(&h);

        h.use_cases
            .confirm(&token, "correct cheval batterie agrafe")
            .await
            .unwrap();

        let passwords = h.consumers.passwords.lock().unwrap();
        assert_eq!(passwords.len(), 1);
        assert_eq!(passwords[0].0, id);
        assert!(
            bcrypt::verify("correct cheval batterie agrafe", &passwords[0].1).unwrap(),
            "l'empreinte enregistrée ne correspond pas au mot de passe choisi"
        );
    }

    /// Un email qui traîne dans une boîte ne doit pas rester une clé du
    /// compte.
    #[tokio::test]
    async fn un_lien_ne_sert_qu_une_fois() {
        let h = harness();
        h.consumers
            .rows
            .lock()
            .unwrap()
            .push(consumer("abonne@example.org"));
        h.use_cases.request("abonne@example.org").await.unwrap();
        let token = token_from_last_email(&h);

        h.use_cases
            .confirm(&token, "correct cheval batterie agrafe")
            .await
            .unwrap();
        let rejeu = h.use_cases.confirm(&token, "un autre mot de passe").await;

        assert!(matches!(rejeu, Err(PasswordResetError::InvalidToken)));
        assert_eq!(h.consumers.passwords.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn un_lien_expire_ne_vaut_plus_rien() {
        let h = harness();
        h.consumers
            .rows
            .lock()
            .unwrap()
            .push(consumer("abonne@example.org"));
        h.use_cases.request("abonne@example.org").await.unwrap();
        let token = token_from_last_email(&h);
        h.resets.expire_all();

        let resultat = h
            .use_cases
            .confirm(&token, "correct cheval batterie agrafe")
            .await;

        assert!(matches!(resultat, Err(PasswordResetError::InvalidToken)));
    }

    #[tokio::test]
    async fn un_jeton_inventé_est_refusé() {
        let h = harness();
        let resultat = h
            .use_cases
            .confirm("jeton-inventé", "correct cheval batterie agrafe")
            .await;

        assert!(matches!(resultat, Err(PasswordResetError::InvalidToken)));
    }

    /// Une faute de saisie ne doit pas brûler le lien : sinon il faut
    /// redemander un email pour un mot de passe trop court.
    #[tokio::test]
    async fn un_mot_de_passe_refusé_ne_consomme_pas_le_lien() {
        let h = harness();
        h.consumers
            .rows
            .lock()
            .unwrap()
            .push(consumer("abonne@example.org"));
        h.use_cases.request("abonne@example.org").await.unwrap();
        let token = token_from_last_email(&h);

        let resultat = h.use_cases.confirm(&token, "court").await;
        assert!(matches!(resultat, Err(PasswordResetError::InvalidInput(_))));

        h.use_cases
            .confirm(&token, "correct cheval batterie agrafe")
            .await
            .expect("le lien devait rester utilisable");
    }

    /// Un tiers qui aurait demandé un lien ne doit pas garder une porte
    /// ouverte derrière un changement réussi.
    #[tokio::test]
    async fn un_changement_reussi_invalide_les_autres_liens() {
        let h = harness();
        h.consumers
            .rows
            .lock()
            .unwrap()
            .push(consumer("abonne@example.org"));

        h.use_cases.request("abonne@example.org").await.unwrap();
        let premier = token_from_last_email(&h);
        // Deuxième lien émis hors du délai de garde.
        *h.resets.backdate_minutes.lock().unwrap() = 0;
        for row in h.resets.rows.lock().unwrap().iter_mut() {
            row.created_at = Utc::now() - Duration::minutes(RESEND_COOLDOWN_MINUTES + 1);
        }
        h.use_cases.request("abonne@example.org").await.unwrap();
        let second = token_from_last_email(&h);

        h.use_cases
            .confirm(&second, "correct cheval batterie agrafe")
            .await
            .unwrap();

        assert!(matches!(
            h.use_cases
                .confirm(&premier, "encore un autre mot de passe")
                .await,
            Err(PasswordResetError::InvalidToken)
        ));
    }

    #[tokio::test]
    async fn le_marchand_change_son_mot_de_passe_par_le_meme_circuit() {
        let h = harness();
        let compte = merchant("pro@example.org");
        let id = compte.id;
        h.merchants.rows.lock().unwrap().push(compte);
        h.use_cases.request("pro@example.org").await.unwrap();
        let token = token_from_last_email(&h);

        h.use_cases
            .confirm(&token, "correct cheval batterie agrafe")
            .await
            .unwrap();

        let passwords = h.merchants.passwords.lock().unwrap();
        assert_eq!(passwords.len(), 1);
        assert_eq!(passwords[0].0, id);
        assert!(h.consumers.passwords.lock().unwrap().is_empty());
    }
}
