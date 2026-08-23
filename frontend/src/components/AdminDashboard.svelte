<script lang="ts">
  import { onMount } from "svelte";
  import {
    adminStats,
    adminListMerchants,
    adminDeleteMerchant,
    adminListConsumers,
    adminDeleteConsumer,
    adminListProducts,
    adminDeleteProduct,
    adminUnpublishProduct,
    formatPrice,
    ApiError,
    type AdminStats,
    type AdminMerchant,
    type AdminConsumer,
    type AdminProduct,
  } from "../lib/api";
  import { getAdminToken, clearAdminToken } from "../lib/auth";

  type Tab = "marchands" | "consommateurs" | "produits";
  let tab: Tab = "marchands";

  let stats: AdminStats | null = null;
  let merchants: AdminMerchant[] = [];
  let consumers: AdminConsumer[] = [];
  let products: AdminProduct[] = [];
  let loading = true;
  let error = "";
  let token = "";

  async function loadAll() {
    loading = true;
    error = "";
    try {
      [stats, merchants, consumers, products] = await Promise.all([
        adminStats(token),
        adminListMerchants(token),
        adminListConsumers(token),
        adminListProducts(token),
      ]);
    } catch (err) {
      error = err instanceof ApiError ? err.message : "Impossible de charger le backoffice.";
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    const t = getAdminToken();
    if (!t) {
      window.location.href = "/admin/login";
      return;
    }
    token = t;
    loadAll();
  });

  function logout() {
    clearAdminToken();
    window.location.href = "/admin/login";
  }

  async function removeMerchant(id: string, nom: string) {
    if (!confirm(`Supprimer le marchand "${nom}" ? Ses paniers, abonnements et réservations liées seront supprimés aussi.`)) return;
    try {
      await adminDeleteMerchant(id, token);
      merchants = merchants.filter((m) => m.id !== id);
    } catch (err) {
      alert(err instanceof ApiError ? err.message : "La suppression a échoué.");
    }
  }

  async function removeConsumer(id: string, email: string) {
    if (!confirm(`Supprimer le consommateur "${email}" ? Ses abonnements et réservations seront supprimés aussi.`)) return;
    try {
      await adminDeleteConsumer(id, token);
      consumers = consumers.filter((c) => c.id !== id);
    } catch (err) {
      alert(err instanceof ApiError ? err.message : "La suppression a échoué.");
    }
  }

  async function removeProduct(id: string, nom: string) {
    if (!confirm(`Supprimer le panier "${nom}" ?`)) return;
    try {
      await adminDeleteProduct(id, token);
      products = products.filter((p) => p.id !== id);
    } catch (err) {
      alert(err instanceof ApiError ? err.message : "La suppression a échoué.");
    }
  }

  async function unpublishProduct(id: string) {
    try {
      await adminUnpublishProduct(id, token);
      products = products.map((p) => (p.id === id ? { ...p, statut: "ecoule" } : p));
    } catch (err) {
      alert(err instanceof ApiError ? err.message : "L'action a échoué.");
    }
  }
</script>

<div class="screen">
  <div class="header">
    <h1>Backoffice admin</h1>
    <button class="logout" on:click={logout}>Déconnexion</button>
  </div>

  {#if loading}
    <p class="state">Chargement...</p>
  {:else if error}
    <p class="state error">{error}</p>
  {:else}
    {#if stats}
      <div class="stats">
        <div class="stat">
          <div class="stat-value">{stats.marchands}</div>
          <div class="stat-label">marchands</div>
        </div>
        <div class="stat">
          <div class="stat-value">{stats.consommateurs}</div>
          <div class="stat-label">consommateurs</div>
        </div>
        <div class="stat">
          <div class="stat-value">{stats.produits_actifs}</div>
          <div class="stat-label">paniers actifs</div>
        </div>
        <div class="stat">
          <div class="stat-value">{stats.reservations}</div>
          <div class="stat-label">réservations</div>
        </div>
      </div>
    {/if}

    <div class="tabs">
      <button class:active={tab === "marchands"} on:click={() => (tab = "marchands")}>
        Marchands ({merchants.length})
      </button>
      <button class:active={tab === "consommateurs"} on:click={() => (tab = "consommateurs")}>
        Consommateurs ({consumers.length})
      </button>
      <button class:active={tab === "produits"} on:click={() => (tab = "produits")}>
        Paniers ({products.length})
      </button>
    </div>

    <div class="list">
      {#if tab === "marchands"}
        {#each merchants as m}
          <div class="card row">
            <div class="row-info">
              <div class="row-name">{m.nom}</div>
              <div class="row-detail">{m.email} · {m.categorie} · {m.adresse}</div>
            </div>
            <button class="danger" on:click={() => removeMerchant(m.id, m.nom)}>Supprimer</button>
          </div>
        {/each}
        {#if merchants.length === 0}<p class="empty">Aucun marchand.</p>{/if}
      {:else if tab === "consommateurs"}
        {#each consumers as c}
          <div class="card row">
            <div class="row-info">
              <div class="row-name">{c.email}</div>
              <div class="row-detail">Inscrit le {new Date(c.created_at).toLocaleDateString("fr-FR")}</div>
            </div>
            <button class="danger" on:click={() => removeConsumer(c.id, c.email)}>Supprimer</button>
          </div>
        {/each}
        {#if consumers.length === 0}<p class="empty">Aucun consommateur.</p>{/if}
      {:else}
        {#each products as p}
          <div class="card row">
            <div class="row-info">
              <div class="row-name">{p.nom} <span class="status-badge {p.statut === 'publie' ? 'active' : 'exhausted'}">{p.statut}</span></div>
              <div class="row-detail">{p.marchand_nom} · {formatPrice(p.prix_demarque)} · {p.quantite} restants</div>
            </div>
            <div class="row-actions">
              {#if p.statut === "publie"}
                <button class="secondary" on:click={() => unpublishProduct(p.id)}>Dépublier</button>
              {/if}
              <button class="danger" on:click={() => removeProduct(p.id, p.nom)}>Supprimer</button>
            </div>
          </div>
        {/each}
        {#if products.length === 0}<p class="empty">Aucun panier.</p>{/if}
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

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  h1 {
    font-size: 19px;
  }

  .logout {
    background: none;
    border: 1px solid var(--color-border);
    border-radius: 10px;
    padding: 6px 12px;
    font-size: 12px;
    font-weight: 700;
    color: var(--color-muted);
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
    gap: 8px;
  }

  .stat {
    flex: 1;
    background: var(--color-surface);
    border-radius: 14px;
    padding: 10px;
    text-align: center;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.05);
  }

  .stat-value {
    font-family: var(--font-display);
    font-size: 18px;
    font-weight: 600;
    color: var(--color-primary);
  }

  .stat-label {
    font-size: 10px;
    color: var(--color-muted-light);
    font-weight: 600;
  }

  .tabs {
    display: flex;
    background: var(--color-chip);
    border-radius: 12px;
    padding: 3px;
    gap: 2px;
  }

  .tabs button {
    flex: 1;
    padding: 8px 4px;
    border-radius: 10px;
    border: none;
    background: transparent;
    font-size: 12px;
    font-weight: 700;
    color: var(--color-muted-light);
    cursor: pointer;
  }

  .tabs button.active {
    background: #fff;
    color: var(--color-ink);
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px;
  }

  .row-info {
    flex: 1;
    min-width: 0;
  }

  .row-name {
    font-size: 13px;
    font-weight: 700;
    color: var(--color-ink);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .row-detail {
    font-size: 11px;
    color: var(--color-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-actions {
    display: flex;
    flex-direction: column;
    gap: 6px;
    flex-shrink: 0;
  }

  .danger,
  .secondary {
    font-size: 11px;
    font-weight: 700;
    border-radius: 10px;
    padding: 6px 10px;
    white-space: nowrap;
  }

  .danger {
    color: #c0392b;
    background: none;
    border: 1px solid #c0392b;
  }

  .secondary {
    color: var(--color-muted);
    background: none;
    border: 1px solid var(--color-border);
  }

  .empty {
    color: var(--color-muted-light);
    font-size: 14px;
    text-align: center;
    padding: 24px 0;
  }
</style>
