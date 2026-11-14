import { test, expect, type Page } from "@playwright/test";

// Saisie de l'adresse d'un commerce, contre le géocodeur et les suggestions
// fictifs du profil dev : toute saisie reçoit deux suggestions inventées
// (« …, 1000 Bruxelles » et « …, 4000 Liège ») ; une adresse ou un nom de
// commerce qui contient « introuvable » est inconnu du géocodeur.

const STAMP = Date.now();
const PASSWORD = "mot-de-passe-e2e-2026";

async function remplirInscription(page: Page, email: string, nom = `Commerce ${email}`) {
  await page.goto("/pro/login?mode=register");
  await page.getByPlaceholder("Nom du commerce").fill(nom);
  await page.getByRole("combobox", { name: "Pays" }).selectOption("BE");
  await page.getByRole("combobox", { name: "Catégorie" }).selectOption("Boulangerie");
  await page.getByPlaceholder("Email professionnel").fill(email);
  await page.getByPlaceholder("Mot de passe").fill(PASSWORD);
}

test("les suggestions complètent l'adresse saisie", async ({ page }) => {
  await remplirInscription(page, `e2e-suggestion-${STAMP}@test.com`);
  const adresse = page.getByPlaceholder("Adresse");
  await adresse.click();
  await adresse.pressSequentially("Rue du Marché 4");

  const suggestion = page.getByRole("option", { name: "Rue du Marché 4, 1000 Bruxelles" });
  await expect(suggestion).toBeVisible();
  await suggestion.click();
  await expect(adresse).toHaveValue("Rue du Marché 4, 1000 Bruxelles");
  await expect(page.getByRole("listbox")).toBeHidden();
});

test("commerce non trouvé : accepté seulement en ajout manuel", async ({ page }) => {
  await test.step("le commerçant est prévenu et la case d'ajout manuel apparaît", async () => {
    // Adresse connue, mais aucun commerce de ce nom : c'est l'établissement
    // qui manque, pas la rue.
    await remplirInscription(
      page,
      `e2e-introuvable-${STAMP}@test.com`,
      `Commerce introuvable ${STAMP}`,
    );
    await page.getByPlaceholder("Adresse").fill("4 Rue du Marché, 1000 Bruxelles");
    // Seule case visible à ce stade : celle du consentement bêta.
    await page.getByRole("checkbox").check();
    await page.getByRole("button", { name: "Créer mon compte marchand" }).click();

    await expect(page.getByText(/aucun commerce de ce nom/)).toBeVisible();
    await expect(page).toHaveURL(/\/pro\/login/);
    // Pas de carte : seulement une case, jamais pré-cochée.
    await expect(page.locator(".leaflet-container")).toHaveCount(0);
    await expect(
      page.getByRole("checkbox", { name: /je l'ajoute manuellement/ }),
    ).not.toBeChecked();
  });

  await test.step("sans la case cochée, l'inscription reste bloquée", async () => {
    await page.getByRole("button", { name: "Créer mon compte marchand" }).click();
    await expect(page.getByText(/cochez la case pour ajouter votre commerce manuellement/)).toBeVisible();
    await expect(page).toHaveURL(/\/pro\/login/);
  });

  await test.step("case cochée, le compte est créé", async () => {
    await page.getByRole("checkbox", { name: /je l'ajoute manuellement/ }).check();
    await page.getByRole("button", { name: "Créer mon compte marchand" }).click();
    await page.waitForURL("**/pro/dashboard");
    await expect(page.getByRole("heading", { name: "Aujourd'hui" })).toBeVisible();
  });
});
