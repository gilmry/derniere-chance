<script lang="ts">
  import Photo from "./Photo.svelte";
  import { getMerchantOffers, formatPrice, type Merchant } from "../lib/mock";

  export let merchant: Merchant;

  let following = merchant.followed;
  const merchantOffers = getMerchantOffers(merchant.id).filter((o) => o.status === "active");
</script>

<div class="screen">
  <div class="cover">
    <Photo shape="rect" label="Photo de couverture du commerce" />
  </div>
  <div class="body">
    <div class="identity">
      <div class="avatar">
        <Photo shape="rounded" radius={16} label="Logo" />
      </div>
      <div class="identity-text">
        <h1>{merchant.name}</h1>
        <p class="subtitle">{merchant.category} · {merchant.distance} · ⭐ {merchant.rating}</p>
      </div>
    </div>
    <button class="btn btn-dark follow" class:following on:click={() => (following = !following)}>
      {following ? "✓ Abonné" : "+ S'abonner"}
    </button>
    <div class="section-title">Offres actives</div>
    <div class="offers">
      {#each merchantOffers as offer}
        <a class="card offer-row" href={`/offre/${offer.id}`}>
          <div class="offer-photo">
            <Photo shape="rounded" radius={10} label="Photo" />
          </div>
          <div class="offer-info">
            <div class="offer-name">{offer.title}</div>
            <div class="offer-price">{formatPrice(offer.pricePromo)}</div>
          </div>
        </a>
      {/each}
      {#if merchantOffers.length === 0}
        <p class="empty">Pas d'offre active pour le moment.</p>
      {/if}
    </div>
  </div>
</div>

<style>
  .screen {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .cover {
    height: 140px;
    flex-shrink: 0;
  }

  .body {
    padding: 0 20px 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    flex: 1;
    overflow-y: auto;
  }

  .identity {
    display: flex;
    gap: 12px;
    align-items: flex-end;
    margin-top: -32px;
  }

  .avatar {
    width: 64px;
    height: 64px;
    border: 4px solid var(--color-bg);
    border-radius: 16px;
    flex-shrink: 0;
  }

  .identity-text {
    padding-bottom: 2px;
  }

  h1 {
    font-size: 19px;
  }

  .subtitle {
    font-size: 12px;
    color: var(--color-muted);
  }

  .follow {
    width: 100%;
  }

  .follow.following {
    background: var(--color-primary);
  }

  .section-title {
    font-size: 13px;
    font-weight: 700;
    color: var(--color-muted-light);
    margin-top: 4px;
  }

  .offers {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .offer-row {
    display: flex;
    gap: 10px;
    padding: 10px;
    text-decoration: none;
    color: inherit;
  }

  .offer-photo {
    width: 52px;
    height: 52px;
    flex-shrink: 0;
  }

  .offer-info {
    flex: 1;
  }

  .offer-name {
    font-size: 13px;
    font-weight: 700;
    color: var(--color-ink);
  }

  .offer-price {
    font-size: 12px;
    color: var(--color-primary);
    font-weight: 700;
  }

  .empty {
    color: var(--color-muted-light);
    font-size: 14px;
  }
</style>
