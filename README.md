# DernièreChance

Plateforme anti-gaspillage qui connecte les commerçants locaux et les
consommateurs autour des invendus : un marchand publie un panier en démarque
en quelques secondes, les consommateurs à proximité (ou abonnés à ce
commerçant) le réservent et reçoivent un code à présenter en boutique.

Voir [`VISION.md`](./VISION.md) pour le produit en détail.

## Stack

- **Backend** : Rust / Actix-web, architecture hexagonale, PostgreSQL (sqlx),
  auth JWT, photos sur MinIO/S3.
- **Frontend** : Astro (sortie statique) + Svelte, PWA mobile-first.
- **e2e** : Playwright ([`e2e/`](./e2e)), parcours complet marchand →
  consommateur → admin, contre la stack Docker réelle.
- **Infra** : Docker Compose (profils `dev`/`prod`), images construites en CI
  (GitHub Actions → GHCR), déploiement GitOps par cron sur le serveur cible
  (voir [`deploy.sh`](./deploy.sh)). Le déploiement est **épinglé au hash du
  commit** : il attend que toutes les images `sha-<court>` soient publiées
  avant de démarrer, et revient à la version précédente si le démarrage
  échoue.
- **Emails** : alertes « nouvelle démarque » aux abonnés d'un marchand et
  liens de réinitialisation de mot de passe, par relais SMTP (`lettre`) ; sans
  identifiants, le backend se rabat sur un adaptateur qui journalise seulement
  et le dit au démarrage.
- **Géocodage** : la position d'un commerce sur la carte est calculée depuis
  l'adresse et le pays saisis (32 pays européens : UE, Suisse, Royaume-Uni,
  Norvège, Islande, Liechtenstein), à l'inscription et à chaque changement
  d'adresse, par Nominatim (OpenStreetMap). Pendant la saisie, des
  suggestions d'adresses complètes viennent de Photon (OpenStreetMap aussi,
  via le backend). Le backend vérifie ensuite que le commerce existe : un
  lieu de ce nom doit être connu d'OpenStreetMap près de l'adresse (300 m,
  puis 1,5 km). Sinon, le commerçant est prévenu, avec un message qui dit si
  c'est l'adresse ou le commerce qui n'a pas été trouvé, et peut l'ajouter
  manuellement en cochant une case : la date de
  cet ajout est gardée, le commerce est placé à son adresse (ou dans sa ville
  si la rue est inconnue) et le backoffice le signale « commerce non
  vérifié ». Le profil dev utilise
  un géocodeur fictif (`GEOCODEUR_DEV`), voir `.env.example`. Détails, avec
  la réservation de plusieurs paniers sous un seul code :
  [`docs/geocodage-et-reservations.md`](./docs/geocodage-et-reservations.md).
- **RGPD** : consentement explicite tracé pour le programme bêta, retrait en
  un clic avec anonymisation du compte, rétention des journaux bornée à
  30 jours (voir [`docs/rgpd/`](./docs/rgpd/)).

## Démarrer en local

```sh
cp .env.example .env
docker network create ecosolva-web   # réseau Traefik partagé, requis même en dev
docker compose --profile dev up -d --build
```

- Frontend : http://localhost:4322
- Backend : http://localhost:8080
- MinIO console : http://localhost:9001

## Tests

```sh
cd e2e
npm install
npx playwright install --with-deps chromium
npm test
```

Tourne aussi en CI sur chaque push/PR ([`.github/workflows/e2e.yml`](./.github/workflows/e2e.yml)) ;
le rapport (vidéos incluses) est publié sur GitHub Pages.

## Documentation complémentaire

- [`docs/reinitialisation-mot-de-passe.md`](./docs/reinitialisation-mot-de-passe.md) —
  parcours « mot de passe oublié » pour les clients et les commerçants : ce qui
  est tenu (pas d'oracle d'existence de compte, lien à usage unique) et ce qui
  ne l'est pas.
- [`docs/emails.md`](./docs/emails.md) — emails envoyés aux testeurs : mise en
  service d'un relais SMTP (jeton, SPF/DKIM) et décisions inscrites dans le
  code.
- [`docs/n8n-notifications-workflow.md`](./docs/n8n-notifications-workflow.md) —
  workflow n8n qui envoie un email pour chaque événement notable
  (inscription marchand, réservation, retrait).
- [`docs/mcp-oauth.md`](./docs/mcp-oauth.md) — serveur MCP (`/mcp`) avec
  OAuth 2.1/PKCE maison, pour connecter un client MCP (Claude Code, Claude
  Desktop, claude.ai) au compte d'un marchand en lecture/écriture.
- [`docs/rgpd/registre-traitements.md`](./docs/rgpd/registre-traitements.md) —
  registre des traitements (art. 30) du programme bêta : finalités, base
  légale, durées de conservation, sous-traitants, mesures de sécurité.
- [`infra/log-retention/`](./infra/log-retention) — politique de rétention
  des journaux (30 jours), appliquée par `deploy.sh`.

## Licence

MIT — voir [`LICENSE`](./LICENSE).
