<script lang="ts">
  import Photo from "./Photo.svelte";
  import OffersMap from "./OffersMap.svelte";
  import BottomNav from "./BottomNav.svelte";
  import { listOffers, formatPrice, formatDistance, ApiError, MERCHANT_CATEGORIES, type Offer } from "../lib/api";
  import { getBrowserPosition, type Coords } from "../lib/geoloc";

  const categories = ["Tout", ...Object.keys(MERCHANT_CATEGORIES)];

  let view: "carte" | "liste" = "liste";
  let category = "Tout";
  let offers: Offer[] = [];
  let loading = true;
  let loadError = "";
  let coords: Coords | null = null;
  let coordsReady = false;

  async function load() {
    loading = true;
    loadError = "";
    try {
      offers = await listOffers(category === "Tout" ? undefined : category, coords);
    } catch (err) {
      loadError = err instanceof ApiError ? err.message : "Impossible de charger les offres.";
    } finally {
      loading = false;
    }
  }

  getBrowserPosition().then((c) => {
    coords = c;
    coordsReady = true;
  });

  $: if (coordsReady) {
    category;
    load();
  }

  function pickupEnd(offer: Offer): string {
    return new Date(offer.retrait_fin).toLocaleTimeString("fr-FR", { hour: "2-digit", minute: "2-digit" });
  }

  $: featured = offers[0];

  // La liste montre les commerces, pas les paniers : un commerce qui en
  // propose plusieurs n'apparaît qu'une fois, et ses paniers se découvrent
  // sur sa page (/marchand). Les offres arrivent triées par distance, donc
  // l'ordre de première apparition garde les commerces les plus proches en tête.
  interface CommerceAvecPaniers {
    id: string;
    nom: string;
    categorie: string;
    distance_km: number | null;
    logo_url: string | null;
    paniers: number;
    prixMin: number;
    reductionMax: number;
  }

  function regrouperParCommerce(liste: Offer[]): CommerceAvecPaniers[] {
    const parId = new Map<string, CommerceAvecPaniers>();
    for (const offer of liste) {
      const prix = Number(offer.prix_demarque);
      const commerce = parId.get(offer.marchand_id);
      if (commerce) {
        commerce.paniers += 1;
        commerce.prixMin = Math.min(commerce.prixMin, prix);
        commerce.reductionMax = Math.max(commerce.reductionMax, offer.reduction_pct);
      } else {
        parId.set(offer.marchand_id, {
          id: offer.marchand_id,
          nom: offer.marchand_nom,
          categorie: offer.marchand_categorie,
          distance_km: offer.distance_km,
          logo_url: offer.marchand_logo_url,
          paniers: 1,
          prixMin: prix,
          reductionMax: offer.reduction_pct,
        });
      }
    }
    return [...parId.values()];
  }

  $: commerces = regrouperParCommerce(offers);
</script>

<div class="screen">
  <div class="header">
    <div class="header-row">
      <h1>Autour de toi</h1>
      <div class="toggle">
        <button
          class:active={view === "carte"}
          on:click={() => (view = "carte")}
        >Carte</button>
        <button
          class:active={view === "liste"}
          on:click={() => (view = "liste")}
        >Liste</button>
      </div>
    </div>
    <div class="chips">
      {#each categories as cat}
        <button class="chip" class:active={category === cat} on:click={() => (category = cat)}>
          {MERCHANT_CATEGORIES[cat] ? `${MERCHANT_CATEGORIES[cat]} ${cat}` : cat}
        </button>
      {/each}
    </div>
  </div>

  {#if loading}
    <p class="state">Chargement...</p>
  {:else if loadError}
    <p class="state error">{loadError}</p>
  {:else if view === "carte"}
    <div class="map">
      <OffersMap {offers} {coords} />
      {#if featured}
        <a class="map-card" href={`/offre?id=${featured.id}`}>
          <div class="map-card-photo">
            <Photo shape="rounded" radius={12} label="Photo" src={featured.photo_url} />
          </div>
          <div class="map-card-info">
            <div class="map-card-name">{featured.marchand_nom}</div>
            <div class="map-card-detail">
              {featured.nom} · retrait {pickupEnd(featured)}
              {#if formatDistance(featured.distance_km)} · {formatDistance(featured.distance_km)}{/if}
            </div>
          </div>
          <div class="badge-discount">-{featured.reduction_pct}%</div>
        </a>
      {/if}
    </div>
  {:else}
    <div class="list">
      {#each commerces as commerce (commerce.id)}
        <a class="card offer-row" href={`/marchand?id=${commerce.id}`}>
          <div class="offer-photo">
            <!-- Le logo, pas la photo d'un panier : la ligne représente le
                 commerce. Sans logo, la vignette neutre « Logo » s'affiche. -->
            <Photo shape="rounded" radius={14} label="Logo" src={commerce.logo_url} />
          </div>
          <div class="offer-info">
            <div class="offer-name">{commerce.nom}</div>
            <div class="offer-detail">
              {MERCHANT_CATEGORIES[commerce.categorie] ?? ""} {commerce.categorie}
              {#if formatDistance(commerce.distance_km)} · {formatDistance(commerce.distance_km)}{/if}
            </div>
            <div class="price-row">
              <span class="paniers">
                {commerce.paniers} {commerce.paniers > 1 ? "paniers" : "panier"}
              </span>
              <span class="price-new">
                {commerce.paniers > 1 ? "dès " : ""}{formatPrice(commerce.prixMin)}
              </span>
            </div>
          </div>
          <div class="badge-discount">
            {commerce.paniers > 1 ? "jusqu'à " : ""}-{commerce.reductionMax}%
          </div>
        </a>
      {/each}
      {#if offers.length === 0}
        <p class="empty">Aucune offre dans cette catégorie pour le moment.</p>
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

  .header {
    padding: 8px 20px 14px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  h1 {
    font-size: 22px;
  }

  .toggle {
    display: flex;
    background: var(--color-chip);
    border-radius: 12px;
    padding: 3px;
    gap: 2px;
  }

  .toggle button {
    padding: 6px 12px;
    border-radius: 10px;
    border: none;
    background: transparent;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-muted-light);
    cursor: pointer;
  }

  .toggle button.active {
    background: #fff;
    color: var(--color-ink);
    font-weight: 700;
  }

  .chips {
    display: flex;
    gap: 8px;
    overflow-x: auto;
  }

  .chip.active {
    background: var(--color-ink);
    color: #fff;
  }

  .map {
    position: relative;
    flex: 1;
    margin: 0 20px 16px;
    border-radius: 20px;
    overflow: hidden;
    min-height: 320px;
    background: #dce8da;
  }

  .map-card {
    position: absolute;
    left: 16px;
    bottom: 16px;
    right: 16px;
    background: #fff;
    border-radius: 16px;
    padding: 14px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
    display: flex;
    gap: 12px;
    align-items: center;
    text-decoration: none;
    color: inherit;
  }

  .map-card-photo {
    width: 56px;
    height: 56px;
    flex-shrink: 0;
  }

  .map-card-info {
    flex: 1;
  }

  .map-card-name {
    font-weight: 700;
    font-size: 14px;
    color: var(--color-ink);
  }

  .map-card-detail {
    font-size: 12px;
    color: var(--color-muted);
  }

  .list {
    flex: 1;
    padding: 0 20px 16px;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .offer-row {
    display: flex;
    gap: 12px;
    text-decoration: none;
    color: inherit;
  }

  .offer-photo {
    width: 72px;
    height: 72px;
    flex-shrink: 0;
  }

  .offer-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 4px;
    justify-content: center;
  }

  .offer-name {
    font-weight: 700;
    font-size: 14px;
    color: var(--color-ink);
  }

  .offer-detail {
    font-size: 12px;
    color: var(--color-muted);
  }

  .price-new {
    font-size: 15px;
  }

  .paniers {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-ink);
  }

  .offer-row .badge-discount {
    align-self: center;
    white-space: nowrap;
  }

  .empty {
    color: var(--color-muted-light);
    font-size: 14px;
    text-align: center;
    padding: 24px 0;
  }
</style>
