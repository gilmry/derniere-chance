<script lang="ts">
  import { onMount } from "svelte";
  import Photo from "./Photo.svelte";
  import {
    merchantDashboard,
    listMyProducts,
    markEcoule,
    formatPrice,
    ApiError,
    type MerchantDashboard,
    type Product,
  } from "../lib/api";
  import { getMerchantToken } from "../lib/auth";

  let stats: MerchantDashboard | null = null;
  let products: Product[] = [];
  let loading = true;
  let loadError = "";

  onMount(async () => {
    const token = getMerchantToken();
    if (!token) {
      window.location.href = "/pro/login";
      return;
    }
    try {
      [stats, products] = await Promise.all([merchantDashboard(token), listMyProducts(token)]);
    } catch (err) {
      loadError = err instanceof ApiError ? err.message : "Impossible de charger le tableau de bord.";
    } finally {
      loading = false;
    }
  });

  function pickupEnd(p: Product): string {
    return new Date(p.retrait_fin).toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
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
  <h1>Aujourd'hui</h1>
  {#if loading}
    <p class="state">Chargement...</p>
  {:else if loadError}
    <p class="state error">{loadError}</p>
  {:else}
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
            <button class="mark-btn" on:click={() => ecoule(product.id)}>Marquer écoulé</button>
          {:else}
            <span class="status-badge exhausted">
              {product.statut === "ecoule" ? "Écoulé" : "Expiré"}
            </span>
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

  h1 {
    font-size: 19px;
  }

  .state {
    padding: 24px 0;
    text-align: center;
    color: var(--color-muted);
  }

  .state.error {
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
</style>
