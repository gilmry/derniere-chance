<script lang="ts">
  import { onMount } from "svelte";
  import Photo from "./Photo.svelte";
  import {
    merchantDashboard,
    listMyProducts,
    markEcoule,
    validatePickup,
    getMyMerchantProfile,
    updateMerchantProfile,
    uploadMerchantLogo,
    formatPrice,
    ApiError,
    MERCHANT_CATEGORIES,
    type MerchantDashboard,
    type Product,
    type Merchant,
  } from "../lib/api";
  import { getMerchantToken, clearMerchantToken } from "../lib/auth";

  function logout() {
    clearMerchantToken();
    window.location.href = "/pro/login";
  }

  let stats: MerchantDashboard | null = null;
  let products: Product[] = [];
  let merchant: Merchant | null = null;
  let loading = true;
  let loadError = "";

  let logoFileInput: HTMLInputElement;
  let logoPreview: string | null = null;
  let uploadingLogo = false;
  let logoError = "";

  async function onLogoChange(event: Event) {
    const file = (event.target as HTMLInputElement).files?.[0];
    if (!file) return;
    const token = getMerchantToken();
    if (!token) return;

    logoPreview = URL.createObjectURL(file);
    logoError = "";
    uploadingLogo = true;
    try {
      const logo_url = await uploadMerchantLogo(file, token);
      if (merchant) merchant = { ...merchant, logo_url };
    } catch (err) {
      logoError = err instanceof ApiError ? err.message : "L'envoi du logo a échoué.";
    } finally {
      uploadingLogo = false;
    }
  }

  let editingProfile = false;
  let profileNom = "";
  let profileAdresse = "";
  let profileCategorie = "";
  let profileSaving = false;
  let profileError = "";

  function startEditProfile() {
    if (!merchant) return;
    profileNom = merchant.nom;
    profileAdresse = merchant.adresse;
    profileCategorie = merchant.categorie;
    profileError = "";
    editingProfile = true;
  }

  async function saveProfile(event: SubmitEvent) {
    event.preventDefault();
    const token = getMerchantToken();
    if (!token) return;
    profileSaving = true;
    profileError = "";
    try {
      merchant = await updateMerchantProfile(
        { nom: profileNom, adresse: profileAdresse, categorie: profileCategorie },
        token,
      );
      editingProfile = false;
    } catch (err) {
      profileError = err instanceof ApiError ? err.message : "La mise à jour a échoué.";
    } finally {
      profileSaving = false;
    }
  }

  let pickupCode = "";
  let pickupChecking = false;
  let pickupResult = "";
  let pickupError = "";

  async function checkPickupCode(event: SubmitEvent) {
    event.preventDefault();
    const token = getMerchantToken();
    if (!token || !pickupCode.trim()) return;
    pickupChecking = true;
    pickupResult = "";
    pickupError = "";
    try {
      const validation = await validatePickup(pickupCode.trim().toUpperCase(), token);
      pickupResult = `✓ "${validation.produit_nom}" remis - code ${validation.code}`;
      pickupCode = "";
      // Les compteurs du jour (paniers sauvés / chiffre récupéré) viennent de bouger.
      merchantDashboard(token).then((s) => (stats = s));
    } catch (err) {
      pickupError = err instanceof ApiError ? err.message : "Vérification impossible.";
    } finally {
      pickupChecking = false;
    }
  }

  onMount(async () => {
    const token = getMerchantToken();
    if (!token) {
      window.location.href = "/pro/login";
      return;
    }
    try {
      [stats, products, merchant] = await Promise.all([
        merchantDashboard(token),
        listMyProducts(token),
        getMyMerchantProfile(token),
      ]);
    } catch (err) {
      loadError = err instanceof ApiError ? err.message : "Impossible de charger le tableau de bord.";
    } finally {
      loading = false;
    }
  });

  function pickupEnd(p: Product): string {
    return new Date(p.retrait_fin).toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
  }

  // Repart d'un panier déjà écoulé/expiré sans tout retaper : nom, description,
  // prix et photo repris, la fenêtre de retrait (forcément passée) est à resaisir.
  function recreate(product: Product) {
    sessionStorage.setItem(
      "dc_recreate_product",
      JSON.stringify({
        nom: product.nom,
        description: product.description,
        prix_initial: product.prix_initial,
        prix_demarque: product.prix_demarque,
        quantite: product.quantite,
        photo_url: product.photo_url,
      }),
    );
    window.location.href = "/pro/panier/nouveau";
  }

  function editProduct(product: Product) {
    sessionStorage.setItem(
      "dc_edit_product",
      JSON.stringify({
        id: product.id,
        nom: product.nom,
        description: product.description,
        prix_initial: product.prix_initial,
        prix_demarque: product.prix_demarque,
        quantite: product.quantite,
        retrait_debut: product.retrait_debut,
        retrait_fin: product.retrait_fin,
        photo_url: product.photo_url,
      }),
    );
    window.location.href = "/pro/panier/nouveau";
  }

  async function ecoule(id: string) {
    const token = getMerchantToken();
    if (!token) return;
    try {
      const updated = await markEcoule(id, token);
      products = products.map((p) => (p.id === updated.id ? updated : p));
    } catch {
      // le produit reste affiché comme actif, l'échec est silencieux ici (action secondaire)
    }
  }
</script>

<div class="screen">
  <div class="header-row">
    <h1>Aujourd'hui</h1>
    <div class="header-actions">
      <button class="logout-link" type="button" on:click={logout}>Déconnexion</button>
      {#if !loading && !loadError}
        <button class="logo-btn" on:click={() => logoFileInput.click()} type="button" aria-label="Changer le logo">
          <Photo shape="circle" label="Logo" src={logoPreview ?? merchant?.logo_url} />
          {#if uploadingLogo}<span class="logo-spinner">...</span>{/if}
        </button>
        <input
          class="visually-hidden"
          type="file"
          accept="image/jpeg,image/png,image/webp"
          capture="environment"
          bind:this={logoFileInput}
          on:change={onLogoChange}
        />
      {/if}
    </div>
  </div>
  {#if logoError}<p class="pickup-result error">{logoError}</p>{/if}
  {#if loading}
    <p class="state">Chargement...</p>
  {:else if loadError}
    <p class="state error">{loadError}</p>
  {:else}
    {#if editingProfile}
      <form class="card profile-edit" on:submit={saveProfile}>
        <input class="text-input" bind:value={profileNom} placeholder="Nom du commerce" required />
        <input class="text-input" bind:value={profileAdresse} placeholder="Adresse" required />
        <select class="text-input" bind:value={profileCategorie} required>
          {#each Object.entries(MERCHANT_CATEGORIES) as [cat, emoji]}
            <option value={cat}>{emoji} {cat}</option>
          {/each}
        </select>
        {#if profileError}<p class="pickup-result error">{profileError}</p>{/if}
        <div class="profile-edit-actions">
          <button class="btn btn-secondary" type="button" on:click={() => (editingProfile = false)}>Annuler</button>
          <button class="btn btn-primary" type="submit" disabled={profileSaving}>
            {profileSaving ? "..." : "Enregistrer"}
          </button>
        </div>
      </form>
    {:else if merchant}
      <div class="fiche-row">
        <div class="fiche-info">
          <div class="fiche-nom">{merchant.nom}</div>
          <div class="fiche-detail">{merchant.categorie} · {merchant.adresse}</div>
        </div>
        <button class="edit-link" type="button" on:click={startEditProfile}>Modifier</button>
      </div>
    {/if}
    <form class="pickup-check" on:submit={checkPickupCode}>
      <input
        class="pickup-input"
        placeholder="Code du client (ex. DC-4821)"
        bind:value={pickupCode}
        disabled={pickupChecking}
      />
      <button class="pickup-btn" type="submit" disabled={pickupChecking || !pickupCode.trim()}>
        {pickupChecking ? "..." : "Valider"}
      </button>
    </form>
    {#if pickupResult}<p class="pickup-result">{pickupResult}</p>{/if}
    {#if pickupError}<p class="pickup-result error">{pickupError}</p>{/if}

    <div class="stats">
      <div class="stat stat-dark">
        <div class="stat-value">{stats?.paniers_sauves ?? 0}</div>
        <div class="stat-label">paniers sauvés</div>
      </div>
      <div class="card stat">
        <div class="stat-value primary">{formatPrice(stats?.chiffre_recupere ?? "0")}</div>
        <div class="stat-label">chiffre récupéré</div>
      </div>
    </div>
    <div class="section-header">
      <div class="section-title">Paniers</div>
      <a class="add-link" href="/pro/panier/nouveau">+ Ajouter</a>
    </div>
    <div class="list">
      {#each products as product}
        <div class="card row">
          <div class="row-photo">
            <Photo shape="rounded" radius={10} label="Photo" src={product.photo_url} />
          </div>
          <div class="row-info">
            <div class="row-name">{product.nom}</div>
            <div class="row-detail">
              {product.statut === "publie"
                ? `${product.quantite} restants · retrait ${pickupEnd(product)}`
                : product.statut === "ecoule"
                  ? "Écoulé"
                  : "Expiré"}
            </div>
          </div>
          {#if product.statut === "publie"}
            <div class="row-actions">
              <button class="mark-btn" on:click={() => ecoule(product.id)}>Marquer écoulé</button>
              <button class="mark-btn secondary" on:click={() => editProduct(product)}>Modifier</button>
            </div>
          {:else}
            <div class="row-actions">
              <span class="status-badge exhausted">
                {product.statut === "ecoule" ? "Écoulé" : "Expiré"}
              </span>
              <button class="mark-btn" on:click={() => recreate(product)}>Recréer</button>
            </div>
          {/if}
        </div>
      {/each}
      {#if products.length === 0}
        <p class="empty">Aucun panier pour le moment.</p>
      {/if}
    </div>
  {/if}
</div>

<style>
  .screen {
    flex: 1;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
    overflow-y: auto;
  }

  .header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .logout-link {
    font-size: 12px;
    font-weight: 700;
    color: var(--color-muted);
    background: none;
    border: none;
    padding: 4px;
  }

  h1 {
    font-size: 19px;
  }

  .logo-btn {
    position: relative;
    width: 44px;
    height: 44px;
    border-radius: 50%;
    padding: 0;
    border: none;
    background: none;
    flex-shrink: 0;
    overflow: hidden;
  }

  .logo-spinner {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.5);
    color: #fff;
    font-size: 10px;
    font-weight: 700;
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
  }

  .state {
    padding: 24px 0;
    text-align: center;
    color: var(--color-muted);
  }

  .state.error {
    color: #c0392b;
  }

  .fiche-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .fiche-info {
    min-width: 0;
  }

  .fiche-nom {
    font-size: 14px;
    font-weight: 700;
    color: var(--color-ink);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .fiche-detail {
    font-size: 12px;
    color: var(--color-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .edit-link {
    flex-shrink: 0;
    font-size: 12px;
    font-weight: 700;
    color: var(--color-primary);
    background: none;
    border: none;
    padding: 4px;
  }

  .profile-edit {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .profile-edit .text-input {
    height: 46px;
    border-radius: 12px;
    border: 1px solid var(--color-border);
    padding: 0 14px;
    font-size: 14px;
    font-family: var(--font-body);
  }

  .profile-edit-actions {
    display: flex;
    gap: 8px;
  }

  .profile-edit-actions .btn {
    flex: 1;
    height: 44px;
  }

  .pickup-check {
    display: flex;
    gap: 8px;
  }

  .pickup-input {
    flex: 1;
    height: 46px;
    border-radius: 12px;
    border: 1px solid var(--color-border);
    padding: 0 14px;
    font-size: 14px;
    font-family: var(--font-body);
    text-transform: uppercase;
  }

  .pickup-btn {
    height: 46px;
    padding: 0 18px;
    border-radius: 12px;
    background: var(--color-ink);
    color: #fff;
    font-weight: 700;
    font-size: 13px;
  }

  .pickup-result {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-primary);
    margin-top: -8px;
  }

  .pickup-result.error {
    color: #c0392b;
  }

  .stats {
    display: flex;
    gap: 10px;
  }

  .stat {
    flex: 1;
    padding: 14px;
    border-radius: 16px;
  }

  .stat-dark {
    background: var(--color-ink);
    color: #fff;
  }

  .stat-value {
    font-family: var(--font-display);
    font-size: 22px;
    font-weight: 600;
  }

  .stat-value.primary {
    color: var(--color-primary);
  }

  .stat-label {
    font-size: 11px;
    opacity: 0.75;
  }

  .stat:not(.stat-dark) .stat-label {
    color: var(--color-muted-light);
    opacity: 1;
  }

  .section-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .section-title {
    font-size: 13px;
    font-weight: 700;
    color: var(--color-muted-light);
  }

  .add-link {
    font-size: 13px;
    font-weight: 700;
    color: var(--color-primary);
    text-decoration: none;
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .empty {
    color: var(--color-muted-light);
    font-size: 14px;
    text-align: center;
    padding: 24px 0;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px;
  }

  .row-photo {
    width: 48px;
    height: 48px;
    flex-shrink: 0;
  }

  .row-info {
    flex: 1;
  }

  .row-name {
    font-size: 13px;
    font-weight: 700;
    color: var(--color-ink);
  }

  .row-detail {
    font-size: 11px;
    color: var(--color-muted);
  }

  .row-actions {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 6px;
    flex-shrink: 0;
  }

  .mark-btn {
    font-size: 11px;
    font-weight: 700;
    color: var(--color-primary);
    background: none;
    border: 1px solid var(--color-primary);
    border-radius: 10px;
    padding: 6px 10px;
    flex-shrink: 0;
  }

  .mark-btn.secondary {
    color: var(--color-muted);
    border-color: var(--color-border);
  }
</style>
