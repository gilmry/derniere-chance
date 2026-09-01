# DernièreChance — Vision produit (MVP)

> Révision : périmètre élargi suite au design PWA (écrans app cliente + backoffice marchand). Le produit passe d'un simple "notifie-moi par email" à un vrai flux de réservation géolocalisée avec code de retrait — voir §6 pour le détail des changements.

## 1. Contexte & problème

Chaque jour, des commerçants locaux (épiceries, boulangeries, primeurs, restaurants) jettent des produits encore consommables faute d'avoir pu les écouler à temps : invendus proches de la date de péremption, surproduction, fins de série. Ils n'ont souvent ni le temps ni les outils pour communiquer une démarque de dernière minute à leur clientèle.

Côté consommateurs, il existe une vraie appétence pour l'anti-gaspillage et les bonnes affaires locales, mais pas de canal simple pour repérer et récupérer, près de chez eux, les invendus qu'un commerçant apprécié brade avant fermeture.

DernièreChance connecte les deux : un commerçant publie un panier en démarque en quelques secondes, les consommateurs à proximité (ou qui suivent ce commerçant) le réservent avec un code à présenter en boutique.

## 2. Vision

Faire du réflexe "je démarque plutôt que je jette" le geste le plus simple possible pour un commerçant local, et faire gagner du temps de recherche aux consommateurs anti-gaspi en leur apportant l'info directement, sans qu'ils aient à chercher.

## 3. Utilisateurs cibles

**Le marchand** — commerçant indépendant ou petite enseigne locale (alimentaire en priorité), peu équipé numériquement, veut un outil rapide à utiliser sans formation, sur mobile ou desktop.

**Le consommateur anti-gaspi** — habitant du quartier/de la ville, sensible au gaspillage et au commerce local, veut repérer rapidement ce qui est disponible près de lui et réserver un panier avant qu'il ne soit épuisé.

## 4. Proposition de valeur

- **Pour le marchand** : écouler des invendus, fidéliser une clientèle sensible à l'anti-gaspi, sans coût ni friction (pas de commission prévue au stade MVP).
- **Pour le consommateur** : trouver et réserver en quelques secondes un panier à prix réduit près de chez lui ou chez un commerçant qu'il suit, avec la garantie de récupérer ce qu'il a réservé (code de retrait) — et être notifié par email des nouveautés des commerçants suivis sans avoir à ouvrir l'app tous les jours.

## 5. Parcours utilisateur

**Marchand**
1. S'inscrit et crée son compte (back-office).
2. Encode sa fiche commerçant (nom, adresse, catégorie).
3. Crée un produit en démarque : nom, description, prix initial, prix démarqué, quantité disponible, fenêtre de retrait.
4. Publie → les consommateurs abonnés à ce marchand sont notifiés.
5. Peut dépublier/marquer comme écoulé.

**Consommateur**
1. Arrive sur la PWA, crée un compte (email + mot de passe) ou se connecte.
2. Parcourt les commerçants et paniers à proximité, en carte ou en liste, filtrable par catégorie.
3. Ouvre le détail d'un panier (photo, prix barré/promo, fenêtre de retrait, quantité restante) et le réserve.
4. Reçoit un code de retrait à présenter en boutique dans la fenêtre indiquée.
5. Peut suivre des marchands ("marchand ami") depuis leur fiche pour recevoir un email à chaque nouvelle démarque, et gérer ses abonnements depuis son profil (avec ses stats : paniers sauvés, montant économisé).

## 6. Périmètre du MVP

**Changements vs. la première version de ce document** : le design de l'app (9 écrans — voir `frontend/`) a fait entrer dans le MVP trois choses qui en étaient explicitement exclues au départ :
- **Géolocalisation** — feed "autour de toi" en carte ou liste, distance affichée par panier/commerçant.
- **Réservation** (pas de paiement en ligne) — le consommateur réserve un panier, reçoit un code, le présente en boutique. Ce n'était pas prévu ; le parcours d'origine s'arrêtait à la notification email.
- **Compte consommateur complet** (email + mot de passe) au lieu d'un simple email sans mot de passe.

Et une chose à mi-chemin : la fiche marchand affiche une **note** (⭐ 4.8). Elle reste affichée comme information (IN), mais **le dépôt d'avis par les consommateurs reste hors MVP** (OUT) — la note est pour l'instant un attribut du marchand, pas le résultat d'un système de review.

**In**
- Back-office marchand : inscription, authentification, CRUD produits en démarque.
- Compte consommateur (email + mot de passe), connexion/inscription.
- Feed géolocalisé (carte + liste) des paniers disponibles, filtrable par catégorie.
- Fiche marchand : infos, note, offres actives, suivi/désabonnement.
- Réservation d'un panier → code de retrait à présenter en boutique, décrément de la quantité disponible.
- Profil consommateur : stats (paniers sauvés, montant économisé), commerçants suivis.
- Suivi/désabonnement d'un marchand.
- Notification email à la publication d'une démarque, envoyée aux abonnés du marchand concerné.
- PWA installable (manifest + service worker) pour un accès rapide depuis mobile.

**Out (roadmap v2+)**
- Paiement en ligne (la réservation est gratuite, le paiement reste en boutique).
- Dépôt d'avis / notation par les consommateurs (seul l'affichage d'une note existe en MVP).
- Notifications push PWA (en plus de l'email).
- Multi-langue.
- Statistiques avancées pour le marchand (impact CO2, etc.).

## 7. Modèle de données (haut niveau)

- **Marchand** : id, nom, adresse, catégorie, note, email de connexion, mot de passe (hash).
- **Produit** : id, marchand_id, nom, description, prix initial, prix démarqué, quantité, fenêtre de retrait, statut (publié/écoulé/expiré).
- **Consommateur** : id, email, mot de passe (hash).
- **Réservation** : id, produit_id, consommateur_id, code de retrait, statut (réservée/récupérée/expirée), créée à.
- **Abonnement** : consommateur_id, marchand_id, date de création.
- **Notification** (log d'envoi) : produit_id, consommateur_id, date d'envoi, statut.

## 8. Stack technique

Reprise de la stack Elevia (`projects/elevia`), qui a fait ses preuves en prod chez Gilles :

- **Backend** : Rust + Actix-web, architecture hexagonale (`domain/` / `application/` / `infrastructure/`), PostgreSQL via SQLx (migrations auto au démarrage), auth JWT + bcrypt.
- **Frontend** : Astro + Svelte, PWA (manifest.json + service-worker.js), offline-friendly comme Elevia.
- **Déploiement** : Docker Compose (profils dev/prod), Traefik + Let's Encrypt en prod, déploiement GitOps par cron (comme Elevia/KoproGo/OpenMajor). Pas encore mis en place pour DernièreChance (`backend/` tourne pour l'instant en local uniquement).

Le backend (`backend/`) est implémenté : entités de domaine, ports (repositories + `EmailSender`), use cases et adaptateurs Postgres, testé de bout en bout (inscription, publication avec notification des abonnés, réservation avec décrément atomique du stock, validation du code de retrait, dashboards). Statut des briques identifiées :
- **Notification email** : fait — le flux de publication appelle `EmailSender` pour chaque abonné. Deux transporteurs sont câblés derrière ce port : `SmtpEmailSender` (relais SMTP quelconque, via `lettre` ; c'est le chemin en service, chez Proton) et `MailjetEmailSender` (API v3.1, voie de secours inactive). Le corps est rendu par un module commun, donc changer de fournisseur ne change rien à ce que le destinataire lit. Sans identifiants, `main` retombe sur `LoggingEmailSender`, ce qui laisse le dev, la CI et les e2e tourner sans rien envoyer. Voir `docs/emails.md`.
- **Réservation** : fait — décrément de quantité atomique en SQL (`UPDATE ... WHERE statut='publie' AND quantite>0`), empêche la survente si deux consommateurs réservent en même temps.
- **Géolocalisation** : toujours pas implémentée. Le frontend prototype affiche des distances statiques (mock) et le backend ne stocke aucune coordonnée ; le calcul réel (position du navigateur + distance au commerçant) reste à faire des deux côtés.

## 9. Hors scope / Roadmap v2

Pistes à explorer une fois le MVP validé par l'usage : paiement en ligne, notifications push, avis/notation déposés par les consommateurs, multi-langue, tableau de bord d'impact (CO2 évité, kilos sauvés).

## 10. Métriques de succès du MVP

- Nombre de marchands actifs (au moins une démarque publiée dans les 30 derniers jours).
- Nombre de produits publiés en démarque.
- Nombre de réservations, et taux de retrait effectif (réservations honorées / réservations créées).
- Nombre d'abonnements consommateur ↔ marchand.
- Part des notifications de démarque effectivement remises au fournisseur (table `notifications`, statut `envoyee` contre `echouee`). Le taux d'ouverture et le taux de clic ne sont pas mesurés : le suivi d'ouverture et la réécriture des liens de Mailjet sont désactivés, ce sont des traceurs que le registre des traitements exclut.
- Nombre de produits marqués "écoulé" suite à une démarque (proxy du gaspillage évité).

## 11. Licence

MIT, comme Elevia — choix délibéré pour favoriser l'adoption par des commerçants sans friction de licence.
