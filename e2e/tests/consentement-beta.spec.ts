import { test, expect } from "@playwright/test";

// Cycle de vie du consentement RGPD au programme bêta, des deux côtés (voir
// docs/rgpd/registre-traitements.md).
//
// Ce que les deux parcours démontrent, bout à bout :
//   1. la case n'est jamais pré-cochée et l'inscription est impossible sans ;
//   2. une fois consenti, l'application est accessible normalement ;
//   3. l'écran « Programme bêta » atteste du consentement donné ;
//   4. le retrait anonymise le compte : reconnexion impossible avec les mêmes
//      identifiants, et le portier backend referme l'accès ;
//   5. côté marchand, le retrait retire aussi ses paniers de la carte
//      publique - laisser l'offre en ligne prolongerait le traitement.

const STAMP = Date.now();
const CONSUMER_EMAIL = `e2e-consent-conso-${STAMP}@test.com`;
const MERCHANT_EMAIL = `e2e-consent-marchand-${STAMP}@test.com`;
const PASSWORD = "password123";
const BASKET_NAME = `Panier consentement ${STAMP}`;

test.describe("consentement bêta", () => {
  test("côté client : case obligatoire, puis retrait qui anonymise", async ({ page }) => {
    page.on("dialog", (dialog) => dialog.accept());

    await test.step("la case n'est pas pré-cochée et retient l'inscription", async () => {
      await page.goto("/compte?mode=register");
      await page.getByPlaceholder("Email").fill(CONSUMER_EMAIL);
      await page.getByPlaceholder("Mot de passe").fill(PASSWORD);

      const consent = page.getByRole("checkbox");
      await expect(consent).not.toBeChecked();

      // Case décochée : la validation native retient le formulaire, on reste
      // sur /compte et aucun compte n'est créé.
      await page.getByRole("button", { name: "Créer mon compte" }).click();
      await expect(page).toHaveURL(/\/compte/);
    });

    await test.step("case cochée, l'inscription passe et l'app est accessible", async () => {
      await page.getByRole("checkbox").check();
      await page.getByRole("button", { name: "Créer mon compte" }).click();
      await page.waitForURL("**/feed");

      await page.goto("/profil");
      await expect(page.getByText("paniers sauvés")).toBeVisible();
    });

    await test.step("l'écran programme bêta atteste du consentement", async () => {
      await page.goto("/consentement");
      await expect(page.getByText("Tu participes au programme bêta")).toBeVisible();
    });

    await test.step("le retrait anonymise le compte", async () => {
      await page.getByRole("button", { name: "Retirer mon consentement" }).click();
      await page.getByRole("button", { name: "Oui, retirer et supprimer" }).click();
      await page.waitForURL(/consentement=retire/);

      // Le compte est anonymisé : le hash du mot de passe a été vidé, donc
      // l'ancien couple email/mot de passe ne vaut plus rien.
      await page.goto("/compte?mode=login");
      await page.getByPlaceholder("Email").fill(CONSUMER_EMAIL);
      await page.getByPlaceholder("Mot de passe").fill(PASSWORD);
      await page.getByRole("button", { name: "Se connecter" }).click();
      await expect(page.locator(".error")).toBeVisible();
      await expect(page).toHaveURL(/\/compte/);
    });
  });

  test("côté marchand : le retrait dépublie aussi les paniers", async ({ page }) => {
    page.on("dialog", (dialog) => dialog.accept());

    await test.step("la case n'est pas pré-cochée et retient l'inscription", async () => {
      await page.goto("/pro/login?mode=register");
      await page.getByPlaceholder("Nom du commerce").fill(`Commerce ${STAMP}`);
      await page.getByPlaceholder("Adresse").fill("Rue du Consentement 1, Bruxelles");
      await page.getByRole("combobox").selectOption("Primeur");
      await page.getByPlaceholder("Email professionnel").fill(MERCHANT_EMAIL);
      await page.getByPlaceholder("Mot de passe").fill(PASSWORD);

      const consent = page.getByRole("checkbox");
      await expect(consent).not.toBeChecked();

      await page.getByRole("button", { name: "Créer mon compte marchand" }).click();
      await expect(page).toHaveURL(/\/pro\/login/);
    });

    await test.step("case cochée, l'inscription passe et le backoffice s'ouvre", async () => {
      await page.getByRole("checkbox").check();
      await page.getByRole("button", { name: "Créer mon compte marchand" }).click();
      await page.waitForURL("**/pro/dashboard");
      await expect(page.getByRole("heading", { name: "Aujourd'hui" })).toBeVisible();
    });

    await test.step("le marchand publie un panier, visible dans le feed public", async () => {
      await page.goto("/pro/panier/nouveau");
      await page.getByPlaceholder("Panier boulanger surprise").fill(BASKET_NAME);
      await page.getByPlaceholder("Ce qu'il contient").fill("Fruits et légumes du jour");
      await page.locator('input[type="time"]').first().fill("00:01");
      await page.locator('input[type="time"]').last().fill("23:59");
      await page.getByRole("button", { name: "Publier maintenant" }).click();
      await page.waitForURL("**/pro/dashboard");

      await page.goto("/feed");
      await expect(page.getByText(BASKET_NAME)).toBeVisible({ timeout: 15_000 });
    });

    await test.step("l'écran programme bêta atteste du consentement", async () => {
      await page.goto("/pro/consentement");
      await expect(page.getByText("Votre commerce participe au programme bêta")).toBeVisible();
    });

    await test.step("le retrait anonymise le compte", async () => {
      await page.getByRole("button", { name: "Retirer mon consentement" }).click();
      await page
        .getByRole("button", { name: "Oui, retirer et supprimer" })
        .click();
      await page.waitForURL(/consentement=retire/);

      await page.goto("/pro/login?mode=login");
      await page.getByPlaceholder("Email professionnel").fill(MERCHANT_EMAIL);
      await page.getByPlaceholder("Mot de passe").fill(PASSWORD);
      await page.getByRole("button", { name: "Se connecter" }).click();
      await expect(page.locator(".error")).toBeVisible();
      await expect(page).toHaveURL(/\/pro\/login/);
    });

    await test.step("le panier a quitté la carte publique", async () => {
      // Le cœur de la démonstration : le traitement cesse vraiment, l'offre
      // ne survit pas au retrait du consentement de son commerçant.
      await page.goto("/feed");
      await expect(page.getByText(BASKET_NAME)).toHaveCount(0, { timeout: 15_000 });
    });
  });
});
