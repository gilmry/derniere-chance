use std::sync::Arc;

use thiserror::Error;

use crate::application::dto::ConsentStatusDto;
use crate::application::ports::{
    ConsentRepository, ConsumerRepository, EventNotifier, MerchantRepository, ProductRepository,
    RepoError,
};
use crate::domain::entities::ConsentSubject;

/// Version du texte de consentement au programme bêta actuellement en
/// vigueur. C'est cette chaîne qui est stockée en base pour dire *à quoi* la
/// personne a consenti, donc elle doit être incrémentée dès que le texte de
/// /confidentialite change sur le fond (nouvelle finalité, nouvelle
/// catégorie de données, nouveau sous-traitant).
///
/// Trois endroits à modifier ensemble, sinon les inscriptions sont refusées
/// tant que le frontend n'est pas redéployé (les deux images sortent du même
/// commit, donc le décalage ne dure que le temps d'un déploiement) :
///   1. cette constante,
///   2. `BETA_CONSENT_VERSION` dans `frontend/src/lib/consent.ts`,
///   3. la date de mise à jour affichée sur `/confidentialite`.
pub const BETA_CONSENT_VERSION: &str = "2026-08-30";

#[derive(Debug, Error)]
pub enum ConsentError {
    #[error("consent version is no longer current")]
    StaleVersion,
    #[error("internal error: {0}")]
    Internal(#[from] RepoError),
}

pub struct ConsentUseCases {
    consent_repo: Arc<dyn ConsentRepository>,
    consumer_repo: Arc<dyn ConsumerRepository>,
    merchant_repo: Arc<dyn MerchantRepository>,
    product_repo: Arc<dyn ProductRepository>,
    event_notifier: Arc<dyn EventNotifier>,
}

impl ConsentUseCases {
    pub fn new(
        consent_repo: Arc<dyn ConsentRepository>,
        consumer_repo: Arc<dyn ConsumerRepository>,
        merchant_repo: Arc<dyn MerchantRepository>,
        product_repo: Arc<dyn ProductRepository>,
        event_notifier: Arc<dyn EventNotifier>,
    ) -> Self {
        Self {
            consent_repo,
            consumer_repo,
            merchant_repo,
            product_repo,
            event_notifier,
        }
    }

    pub async fn status(&self, subject: ConsentSubject) -> Result<ConsentStatusDto, ConsentError> {
        let active = self.consent_repo.find_active(subject).await?;
        let consenti = active
            .as_ref()
            .is_some_and(|consent| consent.version == BETA_CONSENT_VERSION);

        Ok(ConsentStatusDto {
            consenti,
            version_acceptee: active.as_ref().map(|consent| consent.version.clone()),
            accepte_le: active.as_ref().map(|consent| consent.accepte_le),
            version_courante: BETA_CONSENT_VERSION.to_string(),
        })
    }

    pub async fn grant(
        &self,
        subject: ConsentSubject,
        version: &str,
    ) -> Result<ConsentStatusDto, ConsentError> {
        if version != BETA_CONSENT_VERSION {
            return Err(ConsentError::StaleVersion);
        }

        // Une version périmée encore active (texte modifié depuis) doit être
        // retirée avant d'enregistrer la nouvelle : l'index partiel unique
        // n'autorise qu'un consentement actif à la fois, et on veut garder la
        // trace de l'ancien plutôt que de l'écraser.
        if let Some(active) = self.consent_repo.find_active(subject).await? {
            if active.version != BETA_CONSENT_VERSION {
                self.consent_repo.withdraw(subject).await?;
            }
        }

        self.consent_repo
            .grant(subject, BETA_CONSENT_VERSION)
            .await?;
        self.status(subject).await
    }

    /// Retrait du consentement. La base légale du traitement disparaissant,
    /// le compte est anonymisé dans la foulée : c'est le seul moyen de
    /// cesser le traitement sans effacer la preuve du consentement passé ni
    /// les réservations déjà honorées.
    ///
    /// Pour un marchand, l'anonymisation ne suffit pas : ses paniers encore
    /// publiés doivent quitter la carte, sinon le service continuerait à
    /// exposer une offre rattachée à un commerce qui a quitté le programme.
    ///
    /// **Fonctionne aussi sans consentement à retirer**, et c'est voulu : un
    /// compte créé avant la mise en place du consentement, ou dont le
    /// détenteur refuse la nouvelle version du texte, doit pouvoir s'effacer
    /// au lieu de rester coincé derrière le portier. Le retrait ne touche
    /// alors aucune ligne, l'anonymisation s'exécute quand même. C'est aussi
    /// pour cela que les endpoints de consentement restent sur
    /// `Authenticated*` et non sur les portiers.
    pub async fn withdraw(&self, subject: ConsentSubject) -> Result<(), ConsentError> {
        self.consent_repo.withdraw(subject).await?;

        let detail = match subject {
            ConsentSubject::Consumer(id) => {
                self.consumer_repo.anonymize(id).await?;
                String::new()
            }
            ConsentSubject::Merchant(id) => {
                let retires = self.product_repo.unpublish_all_by_merchant(id).await?;
                self.merchant_repo.anonymize(id).await?;
                tracing::info!(marchand_id = %id, retires, "paniers dépubliés au retrait du consentement");
                format!(" {retires} panier(s) dépublié(s).")
            }
        };

        // Le responsable de traitement doit savoir qu'un effacement a eu lieu,
        // pour pouvoir le porter au suivi des demandes d'exercice des droits.
        //
        // Le message ne porte QUE l'identifiant technique et le rôle : y
        // mettre l'email ou le nom du commerce ferait survivre la donnée
        // qu'on vient d'effacer dans une boîte mail et chez le sous-traitant
        // qui achemine la notification. Ce serait exactement le contraire du
        // droit qu'on est en train d'honorer.
        //
        // Émis après l'anonymisation : n'annoncer un effacement qu'une fois
        // qu'il a réellement eu lieu. `notify` est fire-and-forget, un échec
        // d'acheminement ne remet pas en cause l'effacement.
        self.event_notifier
            .notify(
                "compte_anonymise",
                format!(
                    "Retrait de consentement RGPD : compte {} anonymisé \
                     (identifiant technique {}).{detail}",
                    subject.role(),
                    subject.id(),
                ),
            )
            .await;

        Ok(())
    }

    /// Portier utilisé par les extracteurs `ConsentedConsumer` et
    /// `ConsentedMerchant` : vrai seulement si le consentement porte sur la
    /// version en vigueur.
    pub async fn has_current_consent(&self, subject: ConsentSubject) -> Result<bool, ConsentError> {
        Ok(self
            .consent_repo
            .find_active(subject)
            .await?
            .is_some_and(|consent| consent.version == BETA_CONSENT_VERSION))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Mutex;

    use chrono::Utc;
    use uuid::Uuid;

    use crate::application::ports::{
        MerchantUpdate, NewConsumer, NewMerchant, NewProduct, ProductUpdate, ProductWithMerchant,
    };
    use crate::domain::entities::{BetaConsent, Consumer, Merchant, Product, ProductStatus};

    /// Dépôt de consentements en mémoire, fidèle aux invariants tenus par la
    /// base : au plus un consentement actif par sujet, et l'historique des
    /// consentements retirés est conservé.
    #[derive(Default)]
    struct FakeConsentRepo {
        rows: Mutex<Vec<BetaConsent>>,
    }

    impl FakeConsentRepo {
        fn matches(row: &BetaConsent, subject: ConsentSubject) -> bool {
            match subject {
                ConsentSubject::Consumer(id) => row.consommateur_id == Some(id),
                ConsentSubject::Merchant(id) => row.marchand_id == Some(id),
            }
        }

        fn history_len(&self) -> usize {
            self.rows.lock().unwrap().len()
        }
    }

    #[async_trait::async_trait]
    impl ConsentRepository for FakeConsentRepo {
        async fn grant(
            &self,
            subject: ConsentSubject,
            version: &str,
        ) -> Result<BetaConsent, RepoError> {
            let mut rows = self.rows.lock().unwrap();
            if let Some(existing) = rows
                .iter()
                .find(|r| Self::matches(r, subject) && r.retire_le.is_none())
            {
                return Ok(existing.clone());
            }
            let (consommateur_id, marchand_id) = match subject {
                ConsentSubject::Consumer(id) => (Some(id), None),
                ConsentSubject::Merchant(id) => (None, Some(id)),
            };
            let row = BetaConsent {
                id: Uuid::new_v4(),
                consommateur_id,
                marchand_id,
                version: version.to_string(),
                accepte_le: Utc::now(),
                retire_le: None,
            };
            rows.push(row.clone());
            Ok(row)
        }

        async fn find_active(
            &self,
            subject: ConsentSubject,
        ) -> Result<Option<BetaConsent>, RepoError> {
            Ok(self
                .rows
                .lock()
                .unwrap()
                .iter()
                .find(|r| Self::matches(r, subject) && r.retire_le.is_none())
                .cloned())
        }

        async fn withdraw(&self, subject: ConsentSubject) -> Result<u64, RepoError> {
            let mut touched = 0;
            for row in self.rows.lock().unwrap().iter_mut() {
                if Self::matches(row, subject) && row.retire_le.is_none() {
                    row.retire_le = Some(Utc::now());
                    touched += 1;
                }
            }
            Ok(touched)
        }
    }

    /// Ne retient que ce que les tests observent : qui a été anonymisé.
    #[derive(Default)]
    struct FakeConsumerRepo {
        anonymized: Mutex<Vec<Uuid>>,
    }

    #[async_trait::async_trait]
    impl ConsumerRepository for FakeConsumerRepo {
        async fn update_password(&self, _id: Uuid, _password_hash: &str) -> Result<(), RepoError> {
            unimplemented!("hors du périmètre du consentement")
        }
        async fn create(&self, _new: NewConsumer) -> Result<Consumer, RepoError> {
            unimplemented!("hors du périmètre du consentement")
        }
        async fn find_by_email(&self, _email: &str) -> Result<Option<Consumer>, RepoError> {
            Ok(None)
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
        async fn anonymize(&self, id: Uuid) -> Result<(), RepoError> {
            self.anonymized.lock().unwrap().push(id);
            Ok(())
        }
        async fn count(&self) -> Result<i64, RepoError> {
            Ok(0)
        }
    }

    #[derive(Default)]
    struct FakeMerchantRepo {
        anonymized: Mutex<Vec<Uuid>>,
    }

    #[async_trait::async_trait]
    impl MerchantRepository for FakeMerchantRepo {
        async fn update_password(&self, _id: Uuid, _password_hash: &str) -> Result<(), RepoError> {
            unimplemented!("hors du périmètre du consentement")
        }
        async fn create(&self, _new: NewMerchant) -> Result<Merchant, RepoError> {
            unimplemented!("hors du périmètre du consentement")
        }
        async fn find_by_email(&self, _email: &str) -> Result<Option<Merchant>, RepoError> {
            Ok(None)
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
        async fn anonymize(&self, id: Uuid) -> Result<(), RepoError> {
            self.anonymized.lock().unwrap().push(id);
            Ok(())
        }
        async fn count(&self) -> Result<i64, RepoError> {
            Ok(0)
        }
        async fn update_logo(&self, _id: Uuid, _logo_url: &str) -> Result<Merchant, RepoError> {
            unimplemented!("hors du périmètre du consentement")
        }
        async fn update(&self, _id: Uuid, _changes: MerchantUpdate) -> Result<Merchant, RepoError> {
            unimplemented!("hors du périmètre du consentement")
        }
    }

    #[derive(Default)]
    struct FakeProductRepo {
        unpublished: Mutex<Vec<Uuid>>,
    }

    #[async_trait::async_trait]
    impl ProductRepository for FakeProductRepo {
        async fn create(&self, _new: NewProduct) -> Result<Product, RepoError> {
            unimplemented!("hors du périmètre du consentement")
        }
        async fn find_by_id(&self, _id: Uuid) -> Result<Option<Product>, RepoError> {
            Ok(None)
        }
        async fn find_with_merchant(
            &self,
            _id: Uuid,
        ) -> Result<Option<ProductWithMerchant>, RepoError> {
            Ok(None)
        }
        async fn list_active(
            &self,
            _categorie: Option<&str>,
        ) -> Result<Vec<ProductWithMerchant>, RepoError> {
            Ok(vec![])
        }
        async fn list_active_by_merchant(
            &self,
            _marchand_id: Uuid,
        ) -> Result<Vec<Product>, RepoError> {
            Ok(vec![])
        }
        async fn list_by_merchant(&self, _marchand_id: Uuid) -> Result<Vec<Product>, RepoError> {
            Ok(vec![])
        }
        async fn list_all(&self) -> Result<Vec<ProductWithMerchant>, RepoError> {
            Ok(vec![])
        }
        async fn delete(&self, _id: Uuid) -> Result<(), RepoError> {
            Ok(())
        }
        async fn unpublish_all_by_merchant(&self, marchand_id: Uuid) -> Result<u64, RepoError> {
            self.unpublished.lock().unwrap().push(marchand_id);
            Ok(1)
        }
        async fn count_active(&self) -> Result<i64, RepoError> {
            Ok(0)
        }
        async fn update_status(
            &self,
            _id: Uuid,
            _statut: ProductStatus,
        ) -> Result<Product, RepoError> {
            unimplemented!("hors du périmètre du consentement")
        }
        async fn update(&self, _id: Uuid, _changes: ProductUpdate) -> Result<Product, RepoError> {
            unimplemented!("hors du périmètre du consentement")
        }
        async fn reserve_unit(&self, _id: Uuid) -> Result<Product, RepoError> {
            unimplemented!("hors du périmètre du consentement")
        }
    }

    /// Enregistre les notifications émises, pour vérifier ce qu'elles
    /// contiennent - et surtout ce qu'elles ne contiennent pas.
    #[derive(Default)]
    struct FakeNotifier {
        events: Mutex<Vec<(String, String)>>,
    }

    #[async_trait::async_trait]
    impl EventNotifier for FakeNotifier {
        async fn notify(&self, event: &str, message: String) {
            self.events
                .lock()
                .unwrap()
                .push((event.to_string(), message));
        }
    }

    struct Harness {
        use_cases: ConsentUseCases,
        consents: Arc<FakeConsentRepo>,
        consumers: Arc<FakeConsumerRepo>,
        merchants: Arc<FakeMerchantRepo>,
        products: Arc<FakeProductRepo>,
        notifier: Arc<FakeNotifier>,
    }

    fn harness() -> Harness {
        let consents = Arc::new(FakeConsentRepo::default());
        let consumers = Arc::new(FakeConsumerRepo::default());
        let merchants = Arc::new(FakeMerchantRepo::default());
        let products = Arc::new(FakeProductRepo::default());
        let notifier = Arc::new(FakeNotifier::default());
        Harness {
            use_cases: ConsentUseCases::new(
                consents.clone(),
                consumers.clone(),
                merchants.clone(),
                products.clone(),
                notifier.clone(),
            ),
            consents,
            consumers,
            merchants,
            products,
            notifier,
        }
    }

    // --- Portier ---

    #[tokio::test]
    async fn sans_consentement_le_portier_refuse() {
        let h = harness();
        let subject = ConsentSubject::Consumer(Uuid::new_v4());
        assert!(!h.use_cases.has_current_consent(subject).await.unwrap());
    }

    #[tokio::test]
    async fn apres_consentement_le_portier_laisse_passer() {
        let h = harness();
        let subject = ConsentSubject::Consumer(Uuid::new_v4());
        h.use_cases
            .grant(subject, BETA_CONSENT_VERSION)
            .await
            .unwrap();
        assert!(h.use_cases.has_current_consent(subject).await.unwrap());
    }

    #[tokio::test]
    async fn le_consentement_d_un_marchand_n_ouvre_pas_le_compte_client_de_meme_id() {
        // Garde-fou contre une confusion de sujets dans le dépôt : les deux
        // principaux vivent dans des tables distinctes et pourraient porter
        // le même UUID.
        let h = harness();
        let id = Uuid::new_v4();
        h.use_cases
            .grant(ConsentSubject::Merchant(id), BETA_CONSENT_VERSION)
            .await
            .unwrap();

        assert!(h
            .use_cases
            .has_current_consent(ConsentSubject::Merchant(id))
            .await
            .unwrap());
        assert!(!h
            .use_cases
            .has_current_consent(ConsentSubject::Consumer(id))
            .await
            .unwrap());
    }

    // --- Version du texte ---

    #[tokio::test]
    async fn accepter_une_version_perimee_est_refuse() {
        let h = harness();
        let subject = ConsentSubject::Consumer(Uuid::new_v4());
        let result = h.use_cases.grant(subject, "1970-01-01").await;
        assert!(matches!(result, Err(ConsentError::StaleVersion)));
        // Rien n'a été enregistré : un refus ne doit pas laisser de trace.
        assert_eq!(h.consents.history_len(), 0);
    }

    #[tokio::test]
    async fn un_accord_sur_une_version_perimee_ne_vaut_plus() {
        // Simule une montée de version du texte : l'accord existant porte sur
        // l'ancienne, le portier doit se refermer et redemander l'accord.
        let h = harness();
        let subject = ConsentSubject::Consumer(Uuid::new_v4());
        h.consents.grant(subject, "2020-01-01").await.unwrap();

        assert!(!h.use_cases.has_current_consent(subject).await.unwrap());
        let status = h.use_cases.status(subject).await.unwrap();
        assert!(!status.consenti);
        assert_eq!(status.version_acceptee.as_deref(), Some("2020-01-01"));
        assert_eq!(status.version_courante, BETA_CONSENT_VERSION);
    }

    #[tokio::test]
    async fn re_accepter_apres_montee_de_version_conserve_l_ancien_accord() {
        let h = harness();
        let subject = ConsentSubject::Consumer(Uuid::new_v4());
        h.consents.grant(subject, "2020-01-01").await.unwrap();

        h.use_cases
            .grant(subject, BETA_CONSENT_VERSION)
            .await
            .unwrap();

        assert!(h.use_cases.has_current_consent(subject).await.unwrap());
        // Deux lignes : l'ancien accord retiré, le nouveau actif. La preuve
        // de ce qui a été accepté par le passé ne doit jamais être écrasée.
        assert_eq!(h.consents.history_len(), 2);
    }

    #[tokio::test]
    async fn consentir_deux_fois_ne_cree_pas_de_doublon() {
        let h = harness();
        let subject = ConsentSubject::Consumer(Uuid::new_v4());
        h.use_cases
            .grant(subject, BETA_CONSENT_VERSION)
            .await
            .unwrap();
        let premier = h.use_cases.status(subject).await.unwrap().accepte_le;

        h.use_cases
            .grant(subject, BETA_CONSENT_VERSION)
            .await
            .unwrap();

        assert_eq!(h.consents.history_len(), 1);
        // La date d'origine fait foi, un second clic ne la repousse pas.
        assert_eq!(
            h.use_cases.status(subject).await.unwrap().accepte_le,
            premier
        );
    }

    // --- Retrait ---

    #[tokio::test]
    async fn le_retrait_client_referme_le_portier_et_anonymise() {
        let h = harness();
        let id = Uuid::new_v4();
        let subject = ConsentSubject::Consumer(id);
        h.use_cases
            .grant(subject, BETA_CONSENT_VERSION)
            .await
            .unwrap();

        h.use_cases.withdraw(subject).await.unwrap();

        assert!(!h.use_cases.has_current_consent(subject).await.unwrap());
        assert_eq!(*h.consumers.anonymized.lock().unwrap(), vec![id]);
        // Le compte est anonymisé, jamais le mauvais principal.
        assert!(h.merchants.anonymized.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn le_retrait_marchand_depublie_ses_paniers_puis_anonymise() {
        let h = harness();
        let id = Uuid::new_v4();
        let subject = ConsentSubject::Merchant(id);
        h.use_cases
            .grant(subject, BETA_CONSENT_VERSION)
            .await
            .unwrap();

        h.use_cases.withdraw(subject).await.unwrap();

        assert!(!h.use_cases.has_current_consent(subject).await.unwrap());
        // Les paniers quittent la carte : laisser en ligne l'offre d'un
        // commerce qui a quitté le programme continuerait le traitement.
        assert_eq!(*h.products.unpublished.lock().unwrap(), vec![id]);
        assert_eq!(*h.merchants.anonymized.lock().unwrap(), vec![id]);
        assert!(h.consumers.anonymized.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn le_retrait_conserve_la_preuve_du_consentement_passe() {
        // L'art. 7 §1 impose de pouvoir démontrer le consentement : le
        // retrait l'horodate, il ne l'efface pas.
        let h = harness();
        let subject = ConsentSubject::Consumer(Uuid::new_v4());
        h.use_cases
            .grant(subject, BETA_CONSENT_VERSION)
            .await
            .unwrap();

        h.use_cases.withdraw(subject).await.unwrap();

        assert_eq!(h.consents.history_len(), 1);
        let rows = h.consents.rows.lock().unwrap();
        assert!(rows[0].retire_le.is_some());
        assert_eq!(rows[0].version, BETA_CONSENT_VERSION);
    }

    #[tokio::test]
    async fn s_effacer_sans_avoir_jamais_consenti_fonctionne() {
        // Un compte créé avant la mise en place du consentement doit pouvoir
        // partir, pas rester coincé derrière le portier. Le retrait ne touche
        // aucune ligne, l'anonymisation s'exécute quand même.
        let h = harness();
        let id = Uuid::new_v4();
        let subject = ConsentSubject::Consumer(id);
        assert!(!h.use_cases.has_current_consent(subject).await.unwrap());

        h.use_cases.withdraw(subject).await.unwrap();

        assert_eq!(*h.consumers.anonymized.lock().unwrap(), vec![id]);
        assert_eq!(h.consents.history_len(), 0);
    }

    #[tokio::test]
    async fn un_marchand_qui_refuse_une_nouvelle_version_peut_s_effacer() {
        // Variante marchand : l'accord porte sur une version périmée, le
        // portier est donc fermé. S'effacer doit rester possible, et emporter
        // les paniers encore en ligne.
        let h = harness();
        let id = Uuid::new_v4();
        let subject = ConsentSubject::Merchant(id);
        h.consents.grant(subject, "2020-01-01").await.unwrap();
        assert!(!h.use_cases.has_current_consent(subject).await.unwrap());

        h.use_cases.withdraw(subject).await.unwrap();

        assert_eq!(*h.products.unpublished.lock().unwrap(), vec![id]);
        assert_eq!(*h.merchants.anonymized.lock().unwrap(), vec![id]);
        // L'accord périmé est horodaté comme retiré, pas effacé.
        assert_eq!(h.consents.history_len(), 1);
        assert!(h.consents.rows.lock().unwrap()[0].retire_le.is_some());
    }

    // --- Notification d'effacement ---

    #[tokio::test]
    async fn l_effacement_est_notifie_au_responsable_de_traitement() {
        let h = harness();
        let id = Uuid::new_v4();
        h.use_cases
            .withdraw(ConsentSubject::Consumer(id))
            .await
            .unwrap();

        let events = h.notifier.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].0, "compte_anonymise");
        assert!(events[0].1.contains("consommateur"));
        assert!(events[0].1.contains(&id.to_string()));
    }

    #[tokio::test]
    async fn la_notification_marchand_indique_les_paniers_depublies() {
        let h = harness();
        let id = Uuid::new_v4();
        h.use_cases
            .withdraw(ConsentSubject::Merchant(id))
            .await
            .unwrap();

        let events = h.notifier.events.lock().unwrap();
        assert_eq!(events[0].0, "compte_anonymise");
        assert!(events[0].1.contains("marchand"));
        assert!(events[0].1.contains("1 panier(s) dépublié(s)"));
    }

    #[tokio::test]
    async fn la_notification_ne_transporte_aucune_donnee_personnelle() {
        // Le garde-fou qui compte : mettre l'email ou le nom du commerce dans
        // la notification ferait survivre, dans une boîte mail et chez le
        // sous-traitant qui l'achemine, la donnée qu'on vient d'effacer.
        let h = harness();
        let id = Uuid::new_v4();
        h.use_cases
            .withdraw(ConsentSubject::Merchant(id))
            .await
            .unwrap();

        let events = h.notifier.events.lock().unwrap();
        let message = &events[0].1;
        assert!(!message.contains('@'), "aucune adresse email attendue");
        // Seul l'identifiant technique, déjà opaque, identifie la ligne.
        assert!(message.contains(&id.to_string()));
    }

    #[tokio::test]
    async fn retirer_deux_fois_est_sans_effet_supplementaire() {
        let h = harness();
        let id = Uuid::new_v4();
        let subject = ConsentSubject::Consumer(id);
        h.use_cases
            .grant(subject, BETA_CONSENT_VERSION)
            .await
            .unwrap();

        h.use_cases.withdraw(subject).await.unwrap();
        h.use_cases.withdraw(subject).await.unwrap();

        // Anonymiser est idempotent côté base ; l'appel se répète sans casse.
        assert_eq!(h.consents.history_len(), 1);
        assert!(!h.use_cases.has_current_consent(subject).await.unwrap());
    }
}
