<script lang="ts">
  import { formatPrice, type Offer, type Merchant } from "../lib/mock";

  export let offer: Offer;
  export let merchant: Merchant | undefined;
  export let code: string;
</script>

<div class="screen">
  <div class="check">✓</div>
  <div class="heading">
    <h1>Panier réservé !</h1>
    <p>Présente ce code en boutique</p>
  </div>
  <div class="code">{code}</div>
  <div class="details">
    <div class="row"><span class="label">Commerçant</span><span class="value">{merchant?.name ?? "—"}</span></div>
    <div class="row"><span class="label">Retrait</span><span class="value">Aujourd'hui, {offer.pickupWindow}</span></div>
    <div class="row"><span class="label">Total</span><span class="value">{formatPrice(offer.pricePromo)}</span></div>
  </div>
  <div class="spacer"></div>
  <button class="btn itinerary">Voir l'itinéraire</button>
</div>

<style>
  .screen {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 22px;
    padding: 32px 24px;
    background: var(--color-primary);
    color: #fff;
  }

  .check {
    width: 64px;
    height: 64px;
    border-radius: 20px;
    background: rgba(255, 255, 255, 0.16);
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 30px;
  }

  .heading h1 {
    font-size: 24px;
    color: #fff;
  }

  .heading p {
    font-size: 14px;
    opacity: 0.85;
    margin-top: 6px;
  }

  .code {
    background: #fff;
    border-radius: 20px;
    padding: 20px 28px;
    color: var(--color-ink);
    font-family: var(--font-display);
    font-size: 36px;
    font-weight: 700;
    letter-spacing: 2px;
  }

  .details {
    width: 100%;
    background: rgba(255, 255, 255, 0.12);
    border-radius: 16px;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    text-align: left;
  }

  .row {
    display: flex;
    justify-content: space-between;
    font-size: 13px;
  }

  .label {
    opacity: 0.75;
  }

  .value {
    font-weight: 700;
  }

  .spacer {
    flex: 1;
  }

  .itinerary {
    width: 100%;
    height: 52px;
    background: #fff;
    color: var(--color-primary);
    border-radius: 16px;
    font-size: 15px;
  }
</style>
