<script lang="ts">
  import Photo from "./Photo.svelte";
  import { getMerchant, discountPercent, formatPrice, reservationCodeFor, type Offer } from "../lib/mock";

  export let offer: Offer;

  const merchant = getMerchant(offer.merchantId);
  const code = reservationCodeFor(offer);

  let saved = false;
</script>

<div class="screen">
  <div class="photo-wrap">
    <Photo shape="rect" label="Photo du panier surprise" />
    <span class="badge-discount overlay-badge">-{discountPercent(offer)}%</span>
    <button class="fav" class:saved on:click={() => (saved = !saved)} aria-label="Ajouter aux favoris">
      {saved ? "♥" : "♡"}
    </button>
  </div>
  <div class="body">
    <div>
      <h1>{offer.title}</h1>
      {#if merchant}
        <p class="subtitle"><a href={`/marchand/${merchant.id}`}>{merchant.name}</a> · {merchant.distance}</p>
      {/if}
    </div>
    <div class="price-row">
      <span class="price-old">{formatPrice(offer.priceOriginal)}</span>
      <span class="price-new big">{formatPrice(offer.pricePromo)}</span>
    </div>
    <div class="info-grid">
      <div class="info-tile">
        <div class="info-label">RETRAIT</div>
        <div class="info-value">{offer.pickupWindow}</div>
      </div>
      <div class="info-tile">
        <div class="info-label">RESTANTS</div>
        <div class="info-value">{offer.quantityLeft} paniers</div>
      </div>
    </div>
    <div class="spacer"></div>
    {#if offer.status === "active"}
      <a class="btn btn-primary" href={`/reservation/${code}`}>Réserver ce panier</a>
    {:else}
      <button class="btn btn-primary" disabled>Panier épuisé</button>
    {/if}
  </div>
</div>

<style>
  .screen {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .photo-wrap {
    position: relative;
    height: 280px;
    flex-shrink: 0;
  }

  .overlay-badge {
    position: absolute;
    top: 16px;
    left: 16px;
  }

  .fav {
    position: absolute;
    top: 16px;
    right: 16px;
    width: 36px;
    height: 36px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.9);
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    cursor: pointer;
  }

  .fav.saved {
    color: var(--color-accent);
  }

  .body {
    flex: 1;
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  h1 {
    font-size: 21px;
  }

  .subtitle {
    font-size: 13px;
    color: var(--color-muted);
    margin-top: 2px;
  }

  .subtitle a {
    font-weight: 600;
  }

  .big {
    font-size: 24px;
  }

  .info-grid {
    display: flex;
    gap: 8px;
  }

  .info-tile {
    flex: 1;
    background: var(--color-chip);
    border-radius: 14px;
    padding: 10px 12px;
    text-align: center;
  }

  .info-label {
    font-size: 11px;
    color: var(--color-muted-light);
    font-weight: 700;
  }

  .info-value {
    font-size: 14px;
    font-weight: 700;
    color: var(--color-ink);
  }

  .spacer {
    flex: 1;
  }

  button.btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
