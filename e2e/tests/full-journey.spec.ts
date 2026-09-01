import { test, expect } from "@playwright/test";

// Parcours complet de bout en bout, exécuté dans un seul navigateur : les
// tokens marchand/consommateur/admin vivent sous des clés localStorage
// distinctes (voir frontend/src/lib/auth.ts), donc une seule page peut
// enchaîner les trois rôles sans changer de contexte.
//
//   1. Un marchand s'inscrit et publie un panier.
//   2. Un consommateur s'inscrit et réserve ce panier.
//   3. Le marchand valide le code de retrait (panier enlevé).
//   4. Un admin supprime le marchand puis le consommateur créés (cascade
//      paniers/réservations/abonnements côté DB).

const STAMP = Date.now();
const MERCHANT_EMAIL = `e2e-marchand-${STAMP}@test.com`;
const CONSUMER_EMAIL = `e2e-conso-${STAMP}@test.com`;
// Au moins MIN_PASSWORD_LENGTH caractères (backend/src/domain/services/password.rs) :
// l'inscription refuse plus court depuis que la règle est alignée sur celle de
// la réinitialisation.
const PASSWORD = "mot-de-passe-e2e-2026";
const BASKET_NAME = `Panier e2e ${STAMP}`;
const ADMIN_EMAIL = process.env.E2E_ADMIN_EMAIL ?? "admin@example.com";
const ADMIN_PASSWORD = process.env.E2E_ADMIN_PASSWORD ?? "change-me-immediately";

test("marchand publie, consommateur réserve, marchand valide, admin nettoie", async ({ page }) => {
  page.on("dialog", (dialog) => dialog.accept());

  await test.step("le marchand s'inscrit", async () => {
    await page.goto("/pro/login?mode=register");
    await page.getByPlaceholder("Nom du commerce").fill("Boulangerie e2e");
    await page.getByPlaceholder("Adresse").fill("Rue du Test 1, Bruxelles");
    await page.getByRole("combobox").selectOption("Boulangerie");
    await page.getByPlaceholder("Email professionnel").fill(MERCHANT_EMAIL);
    await page.getByPlaceholder("Mot de passe").fill(PASSWORD);
    // Consentement bêta marchand : case obligatoire et jamais pré-cochée. Un
    // commerçant publie nom, adresse et position sur la carte, d'où le même
    // acte explicite que côté client (docs/rgpd/registre-traitements.md).
    const merchantConsent = page.getByRole("checkbox");
    await expect(merchantConsent).not.toBeChecked();
    await merchantConsent.check();
    await page.getByRole("button", { name: "Créer mon compte marchand" }).click();
    await page.waitForURL("**/pro/dashboard");
    await expect(page.getByRole("heading", { name: "Aujourd'hui" })).toBeVisible();
  });

  await test.step("le marchand publie un panier", async () => {
    await page.getByRole("link", { name: "+ Ajouter" }).click();
    await page.waitForURL("**/pro/panier/nouveau");
    await page.getByPlaceholder("Panier boulanger surprise").fill(BASKET_NAME);
    await page.getByPlaceholder("Ce qu'il contient").fill("Pains et viennoiseries du jour");
    await page.locator('input[type="time"]').first().fill("00:01");
    await page.locator('input[type="time"]').last().fill("23:59");
    await page.getByRole("button", { name: "Publier maintenant" }).click();
    await page.waitForURL("**/pro/dashboard");
    await expect(page.getByText(BASKET_NAME)).toBeVisible();
  });

  await test.step("le consommateur s'inscrit", async () => {
    await page.goto("/compte");
    await page.getByPlaceholder("Email").fill(CONSUMER_EMAIL);
    await page.getByPlaceholder("Mot de passe").fill(PASSWORD);
    // Consentement bêta : case obligatoire et jamais pré-cochée, sans quoi
    // le backend refuse l'inscription (voir docs/rgpd/registre-traitements.md).
    const consent = page.getByRole("checkbox");
    await expect(consent).not.toBeChecked();
    await consent.check();
    await page.getByRole("button", { name: "Créer mon compte" }).click();
    await page.waitForURL("**/feed");
  });

  await test.step("le consommateur suit le commerçant", async () => {
    await expect(page.getByText(BASKET_NAME)).toBeVisible({ timeout: 15_000 });
    await page.getByText(BASKET_NAME).click();
    await page.waitForURL("**/offre?id=*");
    await page.getByRole("link", { name: "Boulangerie e2e" }).click();
    await page.waitForURL("**/marchand?id=*");
    await page.getByRole("button", { name: "+ S'abonner" }).click();
    await expect(page.getByRole("button", { name: "✓ Abonné" })).toBeVisible();
  });

  let pickupCode = "";
  await test.step("le consommateur réserve le panier", async () => {
    await page.goBack();
    await page.waitForURL("**/offre?id=*");
    await page.getByRole("button", { name: "Réserver ce panier" }).click();
    await page.waitForURL("**/reservation");
    pickupCode = (await page.locator(".code").innerText()).trim();
    expect(pickupCode).toMatch(/^DC-\d+$/);
  });

  await test.step("le profil du consommateur affiche tout, sans erreur", async () => {
    // Régression : lister les commerçants suivis plantait côté backend avec
    // "internal error" dès qu'il y en avait au moins un (colonnes manquantes
    // dans la requête SQL) - ce step couvre ce chemin.
    await page.goto("/profil");
    await expect(page.getByText(BASKET_NAME)).toBeVisible();
    await expect(page.getByText(pickupCode)).toBeVisible();
    await expect(page.getByRole("link", { name: "Boulangerie e2e" })).toBeVisible();
    await expect(page.locator(".state.error")).toHaveCount(0);
  });

  await test.step("le marchand valide le code et marque le panier enlevé", async () => {
    await page.goto("/pro/dashboard");
    await page.getByPlaceholder("Code du client (ex. DC-4821)").fill(pickupCode);
    await page.getByRole("button", { name: "Valider" }).click();
    await expect(page.getByText(/remis/)).toBeVisible();
  });

  await test.step("l'admin supprime le marchand et le consommateur créés", async () => {
    await page.goto("/admin/login");
    await page.getByPlaceholder("Email").fill(ADMIN_EMAIL);
    await page.getByPlaceholder("Mot de passe").fill(ADMIN_PASSWORD);
    await page.getByRole("button", { name: "Se connecter" }).click();
    await page.waitForURL("**/admin");

    await page.getByRole("button", { name: /^Marchands/ }).click();
    const merchantRow = page.locator(".row", { hasText: MERCHANT_EMAIL });
    await expect(merchantRow).toBeVisible();
    await merchantRow.getByRole("button", { name: "Supprimer" }).click();
    await expect(merchantRow).toHaveCount(0);

    await page.getByRole("button", { name: /^Consommateurs/ }).click();
    const consumerRow = page.locator(".row", { hasText: CONSUMER_EMAIL });
    await expect(consumerRow).toBeVisible();
    await consumerRow.getByRole("button", { name: "Supprimer" }).click();
    await expect(consumerRow).toHaveCount(0);
  });
});
