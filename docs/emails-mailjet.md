# Emails de notification aux abonnés (Mailjet)

Quand un marchand publie une démarque, le backend prévient par email chaque
consommateur qui suit ce marchand. Le flux vit dans
`ProductUseCases::notify_subscribers` (`backend/src/application/use_cases/product_use_cases.rs`),
derrière le port `EmailSender`. Deux adaptateurs l'implémentent :

- `MailjetEmailSender` : envoi réel via l'API v3.1 de Mailjet. Retenu par
  `main` dès que `MAILJET_API_KEY`, `MAILJET_SECRET_KEY` et
  `MAILJET_FROM_EMAIL` sont renseignés.
- `LoggingEmailSender` : repli qui journalise seulement. Actif quand ces
  variables manquent, pour qu'un poste de dev, la CI et les e2e tournent sans
  clé et sans écrire à de vraies personnes. Le démarrage le signale par un
  `warn` dans les journaux.

Ne pas confondre avec `docs/n8n-notifications-workflow.md` : celui-là couvre
les alertes internes envoyées au responsable (inscription, réservation,
retrait). Ici, ce sont des emails envoyés aux testeurs eux-mêmes.

## Pourquoi Mailjet

Mailjet SAS est une société française dont les serveurs sont dans l'Union
européenne. C'est le seul critère qui a tranché : le registre des traitements
annonce l'absence de transfert hors UE, et les fournisseurs américains du
marché (Resend, SendGrid, Postmark) auraient imposé d'encadrer un transfert
au titre du chapitre V du RGPD, avec la pré-analyse AIPD à rouvrir. Voir
`docs/rgpd/registre-traitements.md`, section « Destinataires et
sous-traitants ».

## Mise en service

1. **Créer les clés** : Mailjet > Paramètres du compte > Gestion des clés API.
   Deux valeurs, une publique (`MAILJET_API_KEY`) et une secrète
   (`MAILJET_SECRET_KEY`).
2. **Valider l'adresse d'expédition** : Mailjet > Expéditeurs et domaines.
   Tant que l'adresse n'est pas validée, l'API refuse chaque envoi avec un
   message explicite, que le backend recopie dans ses journaux.
3. **Authentifier le domaine (SPF + DKIM)** : ajouter les enregistrements DNS
   que Mailjet affiche pour le domaine d'envoi. Sans eux, les alertes partent
   mais finissent en indésirables, ce qui est le mode d'échec le plus coûteux
   ici : silencieux côté serveur, invisible côté testeur.
4. **Renseigner le `.env` du serveur** puis redéployer :

   ```sh
   MAILJET_API_KEY=...
   MAILJET_SECRET_KEY=...
   MAILJET_FROM_EMAIL=alertes@derniere-chance.ecosolva.org
   MAILJET_FROM_NAME=DernièreChance
   ```

   `docker-compose.yml` ne passe ces variables qu'au service `backend`
   (profil `prod`). Le `backend-dev` en est privé volontairement : le jeu de
   données de dev contient des adresses fictives, et les leur envoyer pour de
   vrai ferait rebondir des messages au nom du domaine de production.

5. **Vérifier** : publier une démarque depuis un compte marchand suivi par un
   compte de test dont vous possédez l'adresse. Journaux attendus côté
   backend, puis l'email. En cas d'échec, la notification est enregistrée
   `echouee` en base (table `notifications`) et la cause exacte renvoyée par
   Mailjet apparaît dans les journaux.

## Décisions inscrites dans l'adaptateur

- **Pas de traceur.** `TrackOpens` et `TrackClicks` sont forcés à `disabled` à
  chaque envoi : pas de pixel d'ouverture, pas de réécriture des liens. C'est
  ce qui permet à la politique de confidentialité d'annoncer des emails sans
  pistage, au prix du taux d'ouverture comme métrique (voir `VISION.md` §10).
- **Un 200 ne suffit pas.** Mailjet peut répondre 200 tout en refusant le
  message ; le verdict par destinataire est dans `Messages[].Status`. Sans
  cette vérification, un refus serait journalisé comme `Envoyee`.
- **Envoi borné à 10 s.** Le fan-out aux abonnés est séquentiel : un
  fournisseur lent retarderait la publication d'une démarque.
- **Heure de Bruxelles dans le corps.** Le créneau de retrait est stocké en
  UTC ; l'email affiche une heure de boutique. Le décalage est calculé depuis
  la règle européenne (dernier dimanche de mars et d'octobre à 01:00 UTC)
  plutôt qu'en embarquant une base de fuseaux pour un seul pays.
- **Échappement HTML.** Nom du commerce, nom et description du panier sont
  saisis par les marchands et interpolés dans le corps HTML.
- **L'échec ne casse rien.** `notify_subscribers` est en meilleur effort : un
  envoi raté est tracé `echouee` et n'empêche ni la publication de la
  démarque, ni les notifications aux autres abonnés.

## Contenu de l'email

Objet : `<nom du commerce> : <nom du panier>`. Corps en texte et en HTML, avec
le prix démarqué, le prix initial et la remise, la quantité, le créneau de
retrait, l'adresse du commerce, un lien vers la fiche de l'offre
(`APP_BASE_URL/offre?id=...`) et, en pied, le rappel de la raison de l'envoi
avec le lien vers le profil (`APP_BASE_URL/profil`) où le testeur retire le
marchand de ses suivis. Le rendu est couvert par les tests unitaires de
`backend/src/infrastructure/email/mailjet_email_sender.rs`.
