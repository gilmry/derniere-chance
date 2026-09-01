# Emails de notification aux abonnés

Quand un marchand publie une démarque, le backend prévient par email chaque
consommateur qui suit ce marchand. Le flux vit dans
`ProductUseCases::notify_subscribers`
(`backend/src/application/use_cases/product_use_cases.rs`), derrière le port
`EmailSender`.

Ne pas confondre avec [`n8n-notifications-workflow.md`](./n8n-notifications-workflow.md) :
celui-là couvre les alertes internes envoyées au responsable (inscription,
réservation, retrait). Ici, ce sont des emails envoyés aux testeurs eux-mêmes.
Destinataires, obligations et risques diffèrent, les deux canaux restent
séparés.

## Les adaptateurs

| Adaptateur | Transport | Retenu quand |
|---|---|---|
| `SmtpEmailSender` | relais SMTP quelconque, via `lettre` | `SMTP_SERVEUR`, `SMTP_USER` et `SMTP_TOKEN` sont renseignés |
| `LoggingEmailSender` | aucun, journalise | à défaut |

`infrastructure::email::sender_from_env` tranche et renvoie le nom du
transporteur, que le démarrage journalise. Sans cette ligne, une variable
oubliée se traduirait par un silence indiscernable d'un envoi réussi.

Le repli sur la journalisation n'est pas une commodité : il garantit qu'un
poste de dev, la CI et les e2e tournent sans identifiants et sans écrire à de
vraies personnes depuis un jeu de données de test. C'est aussi pourquoi
`docker-compose.yml` ne passe les variables d'envoi qu'au service `backend` du
profil prod, jamais à `backend-dev`.

Le corps des messages est rendu par `infrastructure::email::message`, séparé
du transport : changer de fournisseur ne change rien à ce que le destinataire
lit, et le gabarit reste testable sans réseau. Un adaptateur HTTP sur l'API
d'un fournisseur se brancherait au même endroit, mais SMTP suffit et n'enferme
dans aucune plateforme.

## Le transport en service : Proton

Proton AG est suisse. La Suisse n'est pas dans l'Union, mais la Commission
européenne lui reconnaît un niveau de protection adéquat, donc le transfert
est licite au titre de l'article 45 du RGPD sans garanties supplémentaires.
C'est la seule sortie de données hors de l'Union dans tout le produit, et elle
est déclarée comme telle dans
[`rgpd/registre-traitements.md`](./rgpd/registre-traitements.md) et dans la
politique de confidentialité publique.

## Mise en service d'un relais SMTP

1. **Obtenir un jeton d'envoi** chez le fournisseur, lié à l'adresse
   d'expédition. Chez Proton, cela suppose un plan payant avec domaine
   personnalisé, et le jeton n'autorise que l'adresse pour laquelle il a été
   créé.
2. **Authentifier le domaine (SPF + DKIM)** en posant les enregistrements DNS
   que le fournisseur affiche. Sans eux, les alertes partent mais finissent en
   indésirables, ce qui est le mode d'échec le plus coûteux ici : silencieux
   côté serveur, invisible côté testeur.
3. **Renseigner le `.env` du serveur** puis redéployer :

   ```sh
   SMTP_SERVEUR=smtp.protonmail.ch
   SMTP_PORT=587
   SMTP_PROTOCOL=TLS/SSL
   SMTP_USER=contact@derniere-chance.ecosolva.org
   SMTP_TOKEN=...
   ```

4. **Vérifier** avec le test de fumée, qui affiche le transporteur retenu
   avant d'envoyer :

   ```sh
   cd backend
   EMAIL_TEST_TO=vous@example.org cargo test --test email_smoke -- --ignored --nocapture
   ```

   Il est marqué `#[ignore]` pour ne jamais partir en CI. C'est le seul moyen
   de vérifier ce que les tests unitaires ne voient pas : identifiants
   valides, expéditeur autorisé par le relais, SPF/DKIM posés. À rejouer après
   toute rotation de jeton ou changement de fournisseur.

En cas d'échec, la notification est enregistrée `echouee` en base (table
`notifications`) et le message exact du relais apparaît dans les journaux.

### `SMTP_PROTOCOL` et le port

Les fournisseurs étiquettent ce réglage de façon incohérente : Proton annonce
« TLS/SSL » sur le port 587, qui est pourtant du STARTTLS. L'adaptateur ne
suit donc `SMTP_PROTOCOL` que lorsqu'il est explicite (`STARTTLS` ou `SMTPS`)
et se fie au port sinon : 465 pour du TLS dès l'ouverture, STARTTLS partout
ailleurs. Le clair n'est jamais utilisé.

## Décisions inscrites dans le code

- **Pas de traceur.** Aucun pixel d'ouverture, aucune réécriture de liens : un
  envoi SMTP ne pose rien de tel, contrairement aux plateformes qui l'activent
  par défaut. C'est ce qui permet à la politique de confidentialité d'annoncer
  des emails sans pistage, au prix du taux d'ouverture comme métrique (voir
  `VISION.md` §10).
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

## Les deux messages

**Alerte « nouvelle démarque »**, envoyée aux abonnés d'un marchand qui
publie.

Objet : `<nom du commerce> : <nom du panier>`. Corps en texte et en HTML, avec
le prix démarqué, le prix initial et la remise, la quantité, le créneau de
retrait, l'adresse du commerce, un lien vers la fiche de l'offre
(`APP_BASE_URL/offre?id=...`) et, en pied, le rappel de la raison de l'envoi
avec le lien vers le profil (`APP_BASE_URL/profil`) où le testeur retire le
marchand de ses suivis. Le rendu est couvert par les tests unitaires de
`backend/src/infrastructure/email/message.rs`.

**Lien de réinitialisation de mot de passe**, envoyé à qui le demande depuis
`/mot-de-passe-oublie`. Objet : « Réinitialiser votre mot de passe
DernièreChance ». Le message ne nomme ni le compte, ni la personne, ni même
l'adresse visée : n'importe qui peut saisir une adresse dans le formulaire,
donc il peut atterrir chez quelqu'un qui n'a rien demandé. Il dit quoi faire,
la durée de validité du lien, et surtout quoi faire si on n'a rien demandé.
Un test le vérifie explicitement. Le parcours complet est décrit dans
[`reinitialisation-mot-de-passe.md`](./reinitialisation-mot-de-passe.md).
