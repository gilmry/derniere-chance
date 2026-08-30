import { test, expect } from "@playwright/test";

// Cycle de vie du consentement RGPD au programme bêta (voir
// docs/rgpd/registre-traitements.md) :
//
//   1. La case n'est jamais pré-cochée et l'inscription est impossible sans.
//   2. Une fois consenti, l'application est accessible normalement.
//   3. Le retrait anonymise le compte : l'ancien email ne permet plus de se
//      reconnecter, et l'accès à l'application est refermé.

const STAMP = Date.now();
const EMAIL = `e2e-consent-${STAMP}@test.com`;
const PASSWORD = "password123";

test("consentement bêta : case obligatoire, puis retrait qui anonymise", async ({ page }) => {
  page.on("dialog", (dialog) => dialog.accept());

  await test.step("la case n'est pas pré-cochée et bloque l'inscription", async () => {
    await page.goto("/compte?mode=register");
    await page.getByPlaceholder("Email").fill(EMAIL);
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

  await test.step("la page programme bêta affiche le consentement donné", async () => {
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
    await page.getByPlaceholder("Email").fill(EMAIL);
    await page.getByPlaceholder("Mot de passe").fill(PASSWORD);
    await page.getByRole("button", { name: "Se connecter" }).click();
    await expect(page.locator(".error")).toBeVisible();
    await expect(page).toHaveURL(/\/compte/);
  });
});
