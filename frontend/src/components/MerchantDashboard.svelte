<script lang="ts">
  import Photo from "./Photo.svelte";
  import { getMerchantOffers } from "../lib/mock";

  const merchantId = "boulangerie-martin";
  const products = getMerchantOffers(merchantId);
</script>

<div class="screen">
  <h1>Aujourd'hui</h1>
  <div class="stats">
    <div class="stat stat-dark">
      <div class="stat-value">12</div>
      <div class="stat-label">paniers sauvés</div>
    </div>
    <div class="card stat">
      <div class="stat-value primary">38 €</div>
      <div class="stat-label">chiffre récupéré</div>
    </div>
  </div>
  <div class="section-header">
    <div class="section-title">Paniers actifs</div>
    <a class="add-link" href="/pro/panier/nouveau">+ Ajouter</a>
  </div>
  <div class="list">
    {#each products as product}
      <div class="card row">
        <div class="row-photo">
          <Photo shape="rounded" radius={10} label="Photo" />
        </div>
        <div class="row-info">
          <div class="row-name">{product.title}</div>
          <div class="row-detail">
            {product.status === "active"
              ? `${product.quantityLeft} restants · retrait ${product.pickupWindow.split("–")[1]?.trim() ?? product.pickupWindow}`
              : "Épuisé"}
          </div>
        </div>
        <span class="status-badge {product.status}">
          {product.status === "active" ? "Actif" : "Épuisé"}
        </span>
      </div>
    {/each}
  </div>
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
</style>
