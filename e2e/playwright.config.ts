import { defineConfig, devices } from "@playwright/test";

// Cible la stack docker compose --profile dev (frontend Astro dev server +
// backend Actix + Postgres). Ce projet ne démarre rien lui-même : lancer
// `docker compose --profile dev up -d` depuis la racine avant `npm test`.
export default defineConfig({
  testDir: "./tests",
  fullyParallel: false,
  retries: 0,
  reporter: [["list"], ["html", { open: "never" }]],
  use: {
    baseURL: process.env.E2E_BASE_URL ?? "http://localhost:4322",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "mobile-chromium",
      use: { ...devices["Pixel 7"] },
    },
  ],
});
