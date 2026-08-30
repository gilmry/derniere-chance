# Registre des traitements — DernièreChance

Registre simplifié tenu au titre de l'article 30 du RGPD. Il couvre le
**programme bêta** en cours et doit être relu à chaque évolution du produit
qui touche aux données (nouvelle finalité, nouveau sous-traitant, nouvelle
catégorie de données collectées).

- **Responsable du traitement** : Gilles Maury, indépendant (architecte
  solutions & développeur freelance), Bruxelles, Belgique — BCE/TVA
  BE0670.778.061
- **Contact** : gilmry@gmail.com
- **Délégué à la protection des données** : aucun (seuils de l'article 37 non
  atteints)
- **Autorité de contrôle chef de file** : Autorité de protection des données
  (APD), Bruxelles — guichet unique au titre de l'article 56, l'établissement
  unique du responsable étant en Belgique. La CNIL est autorité concernée pour
  les testeurs résidant en France et peut être saisie par eux ; elle transmet
  alors à l'APD.
- **Dernière mise à jour** : 30 août 2026
- **Version du texte de consentement en vigueur** : `2026-08-30`

---

## Traitement n° 1 — Programme bêta DernièreChance

### Finalités

1. **Fournir le service pendant la phase de test** : création et gestion du
   compte du testeur, abonnement à des commerçants, réservation de paniers
   invendus, génération et validation du code de retrait en boutique.
2. **Améliorer le produit** : comprendre les usages, identifier les points de
   friction et corriger les anomalies rencontrées par les testeurs.

Finalité technique associée, distincte par sa base légale (voir plus bas) :

3. **Assurer la sécurité du service** : détecter et bloquer les tentatives
   d'intrusion visant le serveur.

### Base légale

**Consentement explicite** (art. 6.1.a), recueilli par une case à cocher non
pré-cochée à l'inscription, distincte de toute autre acceptation.

Recueilli auprès des **deux principaux** : testeurs consommateurs et
commerçants partenaires. Un commerçant publie nom, adresse et position de son
commerce sur la carte publique ; exercé en personne physique, ce sont ses
données personnelles, et elles justifient le même acte explicite.

Traçabilité : chaque acte de consentement est enregistré dans la table
`consentements_beta` (`consommateur_id` **ou** `marchand_id`, exclusifs l'un
de l'autre par contrainte `consentement_un_seul_sujet` ; `version` du texte
accepté, `accepte_le`, `retire_le`). Aucune ligne n'est écrasée ; le retrait
est horodaté en place. Un changement de fond du texte produit une nouvelle version
et l'accord est redemandé avant tout accès (constante `BETA_CONSENT_VERSION`,
côté backend et frontend).

**Exception** : les journaux de sécurité (Suricata, CrowdSec, Fail2ban, AIDE,
journaux système) reposent sur l'**intérêt légitime** du responsable
(art. 6.1.f) à protéger son infrastructure. Ils sont collectés indépendamment
du consentement, ne servent qu'à cette fin, et leur conservation est bornée à
30 jours.

### Catégories de personnes concernées

- Testeurs consommateurs du programme bêta (majoritairement résidents en
  France).
- Commerçants partenaires (compte professionnel : nom du commerce, adresse,
  catégorie, email, position facultative).

### Catégories de données

| Catégorie | Données | Où |
|---|---|---|
| Identification (client) | Adresse email, empreinte bcrypt du mot de passe, date de création | `consommateurs` |
| Identification (commerçant) | Nom et catégorie du commerce, adresse postale, position GPS (facultative), logo, email, empreinte bcrypt | `marchands` |
| Consentement | Sujet (client ou commerçant), version acceptée, date d'acceptation, date de retrait | `consentements_beta` |
| Usage | Commerçants suivis | `abonnements` |
| Usage | Réservations : panier, code de retrait, statut, date | `reservations` |
| Usage | Notifications envoyées et leur statut | `notifications` |
| Usage (commerçant) | Paniers publiés, prix, fenêtres de retrait, photos | `produits` |
| Techniques | Adresses IP, dates, chemins et codes de réponse HTTP | Journaux applicatifs (conteneurs Docker) |
| Sécurité | Adresses IP, signatures d'attaque, bannissements | Suricata, CrowdSec, Fail2ban, journal système |
| Intégrité système | Empreintes de fichiers du serveur (aucune donnée personnelle) | AIDE |

**Aucune donnée sensible** au sens de l'article 9 n'est collectée. Aucun
mineur n'est visé. Aucune décision automatisée ni profilage (art. 22).

**Géolocalisation** : la position du consommateur n'est **jamais
enregistrée**. Transmise en paramètre de requête si le navigateur l'autorise,
elle sert au calcul d'une distance puis disparaît avec la requête. Seule la
position d'un commerçant est stockée (`marchands.latitude/longitude`) :
facultative, fournie par lui à l'inscription via `navigator.geolocation`, et
publiée sur la carte. Un refus n'empêche pas d'utiliser le service, le
commerce n'apparaît simplement pas sur la carte.

Il s'agit d'une position **fixe de point de vente**, saisie une fois, et non
d'un suivi de déplacement : ni surveillance systématique, ni traçage de
personnes. C'est ce qui distingue ce traitement de la « géolocalisation à
grande échelle » qui déclencherait une AIPD.

### Durées de conservation

| Donnée | Durée | Mécanisme |
|---|---|---|
| Compte et données d'usage (client et commerçant) | Durée du programme bêta, puis suppression ou anonymisation sous 1 mois | Manuel à la clôture du bêta |
| Paniers publiés | Dépubliés immédiatement au retrait du consentement du commerçant | `ProductRepository::unpublish_all_by_merchant` |
| Preuve du consentement | 3 ans après retrait ou clôture du programme | Table `consentements_beta`, sans donnée identifiante après anonymisation du compte |
| Réservations | 12 mois (statistiques commerçants), détachées de l'identité dès l'anonymisation | Anonymisation du compte |
| Journaux applicatifs | Bornés à 50 Mo par service (~quelques semaines au rythme du bêta) | `logging.max-size`/`max-file` dans `docker-compose.yml` |
| Journaux de sécurité (Suricata, CrowdSec, Fail2ban, AIDE) | 30 jours maximum | logrotate — `infra/log-retention/` |
| Journaux système (journald, auth.log, syslog) | 30 jours maximum | `MaxRetentionSec` journald + logrotate |
| Journal de déploiement (`deploy.log`) | 30 jours maximum | logrotate — `infra/log-retention/` |

Le retrait du consentement court-circuite ces durées : le compte est anonymisé
immédiatement.

### Destinataires et sous-traitants

| Destinataire | Rôle | Ce qu'il reçoit |
|---|---|---|
| Commerçant partenaire | Destinataire | Les réservations le concernant (code de retrait, panier, statut) |
| n8n auto-hébergé | Sous-traitant | Les seules informations nécessaires aux emails de notification (nouvelle réservation, retrait effectué, inscription). Pour l'événement `compte_anonymise`, uniquement l'identifiant technique et le rôle : jamais l'email ni le nom du commerce, sans quoi la donnée effacée survivrait chez le sous-traitant |
| Fournisseur du serveur (VPS) | Hébergeur d'infrastructure | Aucun accès applicatif ; données chiffrées en transit |

Auto-hébergement : application, base PostgreSQL et photos (MinIO) tournent sur
un serveur administré directement par le responsable. Pas de plateforme
d'hébergement applicatif tierce, pas de CDN, pas de régie publicitaire, pas
d'outil de mesure d'audience.

**Aucun transfert hors de l'Union européenne.**

### Mesures de sécurité

**Applicatives**

- Mots de passe stockés uniquement sous forme d'empreinte bcrypt.
- Authentification par JWT signé, avec vérification du rôle : un jeton
  consommateur ne peut pas authentifier un endpoint marchand ou admin.
- Portiers de consentement (`ConsentedConsumer`, `ConsentedMerchant`) : tout
  endpoint traitant des données d'un testeur exige un consentement à jour, de
  sorte qu'un nouvel endpoint est fermé par défaut. Le portier marchand
  couvre aussi `/mcp`, pour qu'un compte piloté depuis un client MCP ne
  contourne pas le retrait. Les portiers interrogent la base à chaque appel,
  pour qu'un retrait prenne effet immédiatement plutôt qu'à l'expiration du
  jeton.
- Base de données sur un réseau Docker interne, sans port exposé.
- Console MinIO liée à la seule interface de bouclage.

**Infrastructure**

- TLS de bout en bout via Traefik et Let's Encrypt.
- Suricata (détection d'intrusion réseau), CrowdSec (détection
  comportementale), Fail2ban (blocage des tentatives répétées).
- AIDE (contrôle quotidien d'intégrité des fichiers système).
- Mises à jour de sécurité automatiques (`unattended-upgrades`).
- Rotation et purge automatiques de tous les journaux (voir tableau ci-dessus).

**Organisationnelles**

- Notification au responsable de traitement à chaque effacement
  (événement `compte_anonymise`), pour alimenter le suivi des demandes
  d'exercice des droits. Le message ne porte aucune donnée identifiante.
- Un seul administrateur, compte backoffice unique et nominatif.
- Code source public sous licence MIT : les traitements sont auditables par
  quiconque (github.com/gilmry/derniere-chance).
- Déploiement GitOps : toute modification de production passe par un commit
  sur `main`, donc par une trace horodatée et attribuée.

### Exercice des droits

Accès, rectification, effacement, limitation, opposition et portabilité
s'exercent par email à gilmry@gmail.com, avec réponse sous un mois.

Le même écran permet de faire effacer son compte **sans avoir jamais
consenti** : un testeur inscrit avant la mise en place du consentement, ou qui
refuse une nouvelle version du texte, doit pouvoir partir plutôt que rester
coincé derrière le portier. C'est pourquoi les endpoints de consentement sont
les seuls endpoints authentifiés qui ne passent pas par les portiers.

Le **retrait du consentement** est automatisé et immédiat, sans démarche :
depuis `/consentement`, il déclenche l'anonymisation du compte
(`ConsentUseCases::withdraw`). L'email et l'empreinte du mot de passe sont
remplacés par des valeurs neutres, la reconnexion devient impossible et
l'accès applicatif est refermé.

Pour un commerçant, le retrait passe par `/pro/consentement` et produit deux
effets : ses paniers encore publiés sont dépubliés (ils quittent la carte), et
nom, adresse, position, logo et email sont remplacés par des valeurs neutres.

Le compte est **anonymisé plutôt que supprimé** : effacer la ligne
`consommateurs` ou `marchands` supprimerait par cascade la preuve du
consentement, que l'article 7 §1 impose de pouvoir produire, ainsi que les
réservations déjà honorées. Une fois anonymisée, la ligne ne contient plus
aucune donnée personnelle. Une suppression pure et simple reste possible à la
demande, par le backoffice admin.

### Analyse d'impact (AIPD)

**Non requise.** Le screening qui l'établit est documenté à part, avec les
neuf critères du CEPD passés un par un et les évolutions produit qui
obligeraient à le refaire : [`pre-analyse-aipd.md`](./pre-analyse-aipd.md).

Un point d'attention y est traité en détail, parce que c'est le seul qui
approche un déclencheur : la géolocalisation. Elle ne fait pas basculer le
traitement, la position du consommateur n'étant jamais conservée et celle du
marchand étant un point de vente fixe, non une trajectoire.

---

## Journal des révisions

| Date | Modification |
|---|---|
| 2026-08-30 | Création du registre. Mise en conformité initiale du programme bêta : consentement explicite tracé, retrait automatisé avec anonymisation, politique de confidentialité publiée, rétention des journaux bornée à 30 jours. |
| 2026-08-30 | Extension du consentement aux commerçants partenaires (nom, adresse et position publiés sur la carte). Retrait côté marchand : dépublication des paniers puis anonymisation. Portier étendu à tous les endpoints marchand, `/mcp` compris. |
| 2026-08-30 | Effacement possible sans avoir consenti (compte antérieur, ou refus d'une nouvelle version du texte). Notification `compte_anonymise` au responsable de traitement à chaque effacement, sans donnée identifiante. |
