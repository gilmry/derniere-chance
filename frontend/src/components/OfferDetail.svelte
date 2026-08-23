<script lang="ts">
  import { onMount } from "svelte";
  import Photo from "./Photo.svelte";
  import BottomNav from "./BottomNav.svelte";
  import { getOffer, reserveOffer, ApiError, formatPrice, formatDistance, type Offer } from "../lib/api";
  import { getConsumerToken } from "../lib/auth";
  import { getQueryParam } from "../lib/params";
  import { getBrowserPosition } from "../lib/geoloc";

  let offer: Offer | null = null;
  let loading = true;
  let loadError = "";
  let reserving = false;
  let reserveError = "";

  onMount(async () => {
    const id = getQueryParam("id");
    if (!id) {
      loadError = "Offre introuvable.";
      loading = false;
      return;
    }
    try {
      const coords = await getBrowserPosition();
      offer = await getOffer(id, coords);
    } catch (err) {
      loadError = err instanceof ApiError ? err.message : "Impossible de charger cette offre.";
    } finally {
      loading = false;
    }
  });

  function pickupEndTime(offer: Offer): string {
    return new Date(offer.retrait_fin).toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
  }

  function pickupWindow(offer: Offer): string {
    const start = new Date(offer.retrait_debut).toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
    const end = pickupEndTime(offer);
    return `${start} – ${end}`;
  }

  async function reserve() {
    if (!offer) return;
    const token = getConsumerToken();
    if (!token) {
      window.location.href = `/compte?mode=login&next=${encodeURIComponent(`/offre?id=${offer.id}`)}`;
      return;
    }
    reserving = true;
    reserveError = "";
    try {
      const confirmation = await reserveOffer(offer.id, token);
      sessionStorage.setItem("dc_last_reservation", JSON.stringify(confirmation));
      window.location.href = "/reservation";
    } catch (err) {
      reserveError = err instanceof ApiError ? err.message : "La réservation a échoué.";
    } finally {
      reserving = false;
    }
  }
</script>

<div class="screen">
  {#if loading}
    <p class="state">Chargement...</p>
  {:else if loadError || !offer}
    <p class="state error">{loadError || "Offre introuvable."}</p>
  {:else}
    <div class="photo-wrap">
      <Photo shape="rect" label="Photo du panier surprise" src={offer.photo_url} />
      <span class="badge-discount overlay-badge">-{offer.reduction_pct}%</span>
    </div>
    <div class="body">
      <div>
        <h1>{offer.nom}</h1>
        <p class="subtitle">
          <a href={`/marchand?id=${offer.marchand_id}`}>{offer.marchand_nom}</a> · {offer.marchand_categorie}
          {#if formatDistance(offer.distance_km)} · {formatDistance(offer.distance_km)}{/if}
        </p>
      </div>
      {#if offer.description}<p class="description">{offer.description}</p>{/if}
      <div class="price-row">
        <span class="price-old">{formatPrice(offer.prix_initial)}</span>
        <span class="price-new big">{formatPrice(offer.prix_demarque)}</span>
      </div>
      <div class="info-grid">
        <div class="info-tile">
          <div class="info-label">RETRAIT</div>
          <div class="info-value">{pickupWindow(offer)}</div>
        </div>
        <div class="info-tile">
          <div class="info-label">RESTANTS</div>
          <div class="info-value">{offer.quantite} paniers</div>
        </div>
      </div>
      <div class="spacer"></div>
      {#if reserveError}<p class="state error">{reserveError}</p>{/if}
      {#if offer.statut === "publie" && offer.quantite > 0}
        <button class="btn btn-primary" on:click={reserve} disabled={reserving}>
          {reserving ? "..." : "Réserver ce panier"}
        </button>
      {:else}
        <button class="btn btn-primary" disabled>Panier épuisé</button>
      {/if}
    </div>
  {/if}
  <BottomNav active="feed" />
</div>

<style>
  .screen {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .state {
    padding: 40px 20px;
    text-align: center;
    color: var(--color-muted);
  }

  .state.error {
    color: #c0392b;
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

  .description {
    font-size: 13px;
    color: var(--color-muted);
    line-height: 1.5;
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
