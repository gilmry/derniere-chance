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

Traçabilité : chaque acte de consentement est enregistré dans la table
`consentements_beta` (`consommateur_id`, `version` du texte accepté,
`accepte_le`, `retire_le`). Aucune ligne n'est écrasée ; le retrait est
horodaté en place. Un changement de fond du texte produit une nouvelle version
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
| Identification | Adresse email, empreinte bcrypt du mot de passe, date de création | `consommateurs` |
| Consentement | Version acceptée, date d'acceptation, date de retrait | `consentements_beta` |
| Usage | Commerçants suivis | `abonnements` |
| Usage | Réservations : panier, code de retrait, statut, date | `reservations` |
| Usage | Notifications envoyées et leur statut | `notifications` |
| Techniques | Adresses IP, dates, chemins et codes de réponse HTTP | Journaux applicatifs (conteneurs Docker) |
| Sécurité | Adresses IP, signatures d'attaque, bannissements | Suricata, CrowdSec, Fail2ban, journal système |
| Intégrité système | Empreintes de fichiers du serveur (aucune donnée personnelle) | AIDE |

**Aucune donnée sensible** au sens de l'article 9 n'est collectée. Aucun
mineur n'est visé. Aucune décision automatisée ni profilage (art. 22).

**Géolocalisation** : la position du consommateur n'est **jamais enregistrée**.
Transmise en paramètre de requête si le navigateur l'autorise, elle sert au
calcul d'une distance puis disparaît avec la requête. Seule la position d'un
commerçant, facultative et fournie par lui, est stockée (`marchands`).

### Durées de conservation

| Donnée | Durée | Mécanisme |
|---|---|---|
| Compte et données d'usage | Durée du programme bêta, puis suppression ou anonymisation sous 1 mois | Manuel à la clôture du bêta |
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
| n8n auto-hébergé | Sous-traitant | Les seules informations nécessaires aux emails de notification (nouvelle réservation, retrait effectué) |
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
- Portier de consentement (`ConsentedConsumer`) : tout endpoint traitant des
  données de consommateur exige un consentement à jour, de sorte qu'un nouvel
  endpoint est fermé par défaut. Le portier interroge la base à chaque appel,
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

- Un seul administrateur, compte backoffice unique et nominatif.
- Code source public sous licence MIT : les traitements sont auditables par
  quiconque (github.com/gilmry/derniere-chance).
- Déploiement GitOps : toute modification de production passe par un commit
  sur `main`, donc par une trace horodatée et attribuée.

### Exercice des droits

Accès, rectification, effacement, limitation, opposition et portabilité
s'exercent par email à gilmry@gmail.com, avec réponse sous un mois.

Le **retrait du consentement** est automatisé et immédiat, sans démarche :
depuis `/consentement`, il déclenche l'anonymisation du compte
(`ConsentUseCases::withdraw`). L'email et l'empreinte du mot de passe sont
remplacés par des valeurs neutres, la reconnexion devient impossible et
l'accès applicatif est refermé.

Le compte est **anonymisé plutôt que supprimé** : effacer la ligne
`consommateurs` supprimerait par cascade la preuve du consentement, que
l'article 7 §1 impose de pouvoir produire, ainsi que les réservations déjà
honorées par les commerçants. Une fois anonymisée, la ligne ne contient plus
aucune donnée personnelle. Une suppression pure et simple reste possible à la
demande, par le backoffice admin.

### Analyse d'impact (AIPD)

Non requise : le traitement ne figure dans aucune des catégories de la liste
de l'APD imposant une AIPD, il ne porte ni sur des données sensibles, ni sur
une surveillance systématique à grande échelle, ni sur des personnes
vulnérables, et le nombre de testeurs reste réduit. À réévaluer avant toute
ouverture au public.

---

## Journal des révisions

| Date | Modification |
|---|---|
| 2026-08-30 | Création du registre. Mise en conformité initiale du programme bêta : consentement explicite tracé, retrait automatisé avec anonymisation, politique de confidentialité publiée, rétention des journaux bornée à 30 jours. |
