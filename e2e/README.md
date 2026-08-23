# E2E - parcours complet

Un seul test Playwright qui rejoue le parcours de bout en bout :

1. Un marchand s'inscrit et publie un panier.
2. Un consommateur s'inscrit et réserve ce panier.
3. Le marchand valide le code de retrait (panier enlevé).
4. Un admin supprime le marchand puis le consommateur créés.

Tourne contre la stack `docker compose --profile dev` (backend Actix +
Postgres + frontend Astro dev server) - rien n'est mocké, c'est le même
contrat API que la prod.

## Lancer en local

```sh
# depuis la racine du repo
cp .env.example .env   # si pas déjà fait
docker network create ecosolva-web   # réseau Traefik partagé, requis même en dev
                                      # (compose valide tous les réseaux déclarés,
                                      # y compris ceux du profil prod)
docker compose --profile dev up -d --build

cd e2e
npm install
npx playwright install --with-deps chromium   # une fois
npm test
```

Le compte admin utilisé par le test est celui du bootstrap
(`ADMIN_EMAIL`/`ADMIN_PASSWORD` dans `.env`) ; surchargeable via
`E2E_ADMIN_EMAIL`/`E2E_ADMIN_PASSWORD` si besoin. `E2E_BASE_URL` change
la cible (par défaut `http://localhost:4322`, le frontend-dev).

Tourne aussi en CI (`.github/workflows/e2e.yml`) à chaque push/PR sur `main`.
