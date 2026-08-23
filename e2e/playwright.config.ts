import { defineConfig, devices } from "@playwright/test";

// Cible la stack docker compose --profile dev (frontend Astro dev server +
// backend Actix + Postgres). Ce projet ne démarre rien lui-même : lancer
// `docker compose --profile dev up -d` depuis la racine avant `npm test`.
export default defineConfig({
  testDir: "./tests",
  fullyParallel: false,
  retries: 0,
  // slowMo (1s entre chaque action, voir use.launchOptions) allonge le
  // parcours bien au-delà du timeout par défaut de 30s.
  timeout: 180_000,
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    baseURL: process.env.E2E_BASE_URL ?? "http://localhost:4322",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
    video: "on",
    // Marque un temps d'1s entre chaque action Playwright (clic, saisie,
    // navigation...) pour que la vidéo du parcours reste lisible à l'oeil.
    launchOptions: { slowMo: 1000 },
  },
  projects: [
    {
      name: "mobile-chromium",
      use: { ...devices["Pixel 7"] },
    },
  ],
});
