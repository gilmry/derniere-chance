<script lang="ts">
  import { onMount } from "svelte";
  import Photo from "./Photo.svelte";
  import BottomNav from "./BottomNav.svelte";
  import {
    getMerchantProfile,
    followMerchant,
    unfollowMerchant,
    listFollowedMerchants,
    reserveSeveral,
    formatPrice,
    formatDistance,
    ApiError,
    type MerchantProfile,
    type Product,
  } from "../lib/api";
  import { getConsumerToken } from "../lib/auth";
  import { getQueryParam } from "../lib/params";
  import { getBrowserPosition } from "../lib/geoloc";
  import { nomDuPays } from "../lib/pays";

  let merchant: MerchantProfile | null = null;
  let loading = true;
  let loadError = "";
  let following = false;
  let followBusy = false;
  let merchantId = "";

  // Paniers choisis pour une réservation groupée : id du panier → quantité.
  // Un seul code de retrait pour le tout, chez ce commerce.
  const MAX_PAR_PANIER = 10;
  let choix: Record<string, number> = {};
  let reserving = false;
  let reserveError = "";

  $: actives = merchant ? merchant.offres.filter((o) => o.statut === "publie") : [];
  $: nbChoisis = Object.values(choix).reduce((n, q) => n + q, 0);
  $: totalChoisi = actives.reduce((t, o) => t + (choix[o.id] ?? 0) * Number(o.prix_demarque), 0);

  function changer(offer: Product, delta: number) {
    const max = Math.min(offer.quantite, MAX_PAR_PANIER);
    const q = Math.max(0, Math.min(max, (choix[offer.id] ?? 0) + delta));
    choix = { ...choix, [offer.id]: q };
    reserveError = "";
  }

  async function reserverLaSelection() {
    const token = getConsumerToken();
    if (!token) {
      window.location.href = `/compte?mode=login&next=${encodeURIComponent(`/marchand?id=${merchantId}`)}`;
      return;
    }
    const lignes = Object.entries(choix)
      .filter(([, quantite]) => quantite > 0)
      .map(([offre_id, quantite]) => ({ offre_id, quantite }));
    reserving = true;
    reserveError = "";
    try {
      const confirmation = await reserveSeveral(lignes, token);
      sessionStorage.setItem("dc_last_reservation", JSON.stringify(confirmation));
      window.location.href = "/reservation";
    } catch (err) {
      reserveError = err instanceof ApiError ? err.message : "La réservation a échoué.";
      reserving = false;
    }
  }

  onMount(async () => {
    const id = getQueryParam("id");
    if (!id) {
      loadError = "Commerçant introuvable.";
      loading = false;
      return;
    }
    merchantId = id;
    try {
      const coords = await getBrowserPosition();
      merchant = await getMerchantProfile(id, coords);
      const token = getConsumerToken();
      if (token) {
        const followed = await listFollowedMerchants(token);
        following = followed.some((m) => m.id === id);
      }
    } catch (err) {
      loadError = err instanceof ApiError ? err.message : "Impossible de charger ce commerçant.";
    } finally {
      loading = false;
    }
  });

  async function toggleFollow() {
    const token = getConsumerToken();
    if (!token) {
      window.location.href = `/compte?mode=login&next=${encodeURIComponent(`/marchand?id=${merchantId}`)}`;
      return;
    }
    followBusy = true;
    try {
      if (following) {
        await unfollowMerchant(merchantId, token);
      } else {
        await followMerchant(merchantId, token);
      }
      following = !following;
    } catch {
      // reste dans l'état précédent, pas de state d'erreur dédié pour cette action mineure
    } finally {
      followBusy = false;
    }
  }
</script>

<div class="screen">
  {#if loading}
    <p class="state">Chargement...</p>
  {:else if loadError || !merchant}
    <p class="state error">{loadError || "Commerçant introuvable."}</p>
  {:else}
    <div class="body">
      <div class="identity">
        <div class="avatar">
          <Photo shape="rounded" radius={14} label="Logo" src={merchant.logo_url} />
        </div>
        <div class="identity-text">
          <h1>{merchant.nom}</h1>
          <p class="subtitle">
            {merchant.categorie} · {merchant.adresse}{merchant.adresse ? `, ${nomDuPays(merchant.pays)}` : ""}{merchant.note ? ` · ⭐ ${merchant.note}` : ""}{formatDistance(merchant.distance_km) ? ` · ${formatDistance(merchant.distance_km)}` : ""}
          </p>
        </div>
      </div>
      <button class="btn btn-dark follow" class:following on:click={toggleFollow} disabled={followBusy}>
        {following ? "✓ Abonné" : "+ S'abonner"}
      </button>
      <div class="section-title">Offres actives</div>
      <div class="offers">
        {#each actives as offer (offer.id)}
          <div class="card offer-row">
            <a class="offer-link" href={`/offre?id=${offer.id}`}>
              <div class="offer-photo">
                <Photo shape="rounded" radius={10} label="Photo" src={offer.photo_url} />
              </div>
              <div class="offer-info">
                <div class="offer-name">{offer.nom}</div>
                <div class="offer-price">{formatPrice(offer.prix_demarque)}</div>
                <div class="offer-stock">{offer.quantite} restant{offer.quantite > 1 ? "s" : ""}</div>
              </div>
            </a>
            <div class="stepper">
              {#if (choix[offer.id] ?? 0) > 0}
                <button type="button" aria-label={`Retirer un ${offer.nom}`} on:click={() => changer(offer, -1)}>−</button>
                <span class="qty" aria-live="polite">{choix[offer.id]}</span>
              {/if}
              <button
                type="button"
                aria-label={`Ajouter un ${offer.nom}`}
                on:click={() => changer(offer, 1)}
                disabled={(choix[offer.id] ?? 0) >= Math.min(offer.quantite, MAX_PAR_PANIER)}
              >+</button>
            </div>
          </div>
        {/each}
        {#if actives.length === 0}
          <p class="empty">Pas d'offre active pour le moment.</p>
        {/if}
      </div>
      {#if nbChoisis > 0}
        <button class="btn btn-primary reserve-all" on:click={reserverLaSelection} disabled={reserving}>
          {reserving
            ? "Réservation..."
            : `Réserver ${nbChoisis} panier${nbChoisis > 1 ? "s" : ""} · ${formatPrice(totalChoisi)}`}
        </button>
        <p class="hint">Un seul code de retrait pour l'ensemble.</p>
      {/if}
      {#if reserveError}<p class="state error">{reserveError}</p>{/if}
    </div>
  {/if}
  <BottomNav active="feed" />
</div>

<style>
  .screen {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .state {
    padding: 40px 20px;
    text-align: center;
    color: var(--color-muted);
  }

  .state.error {
    color: #c0392b;
  }

  .body {
    padding: 16px 20px 20px;
    display: flex;
    flex-direction: column;
    gap: 12px;
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }

  .identity {
    display: flex;
    gap: 12px;
    align-items: center;
  }

  .avatar {
    width: 56px;
    height: 56px;
    border-radius: 14px;
    flex-shrink: 0;
  }

  .identity-text {
    min-width: 0;
  }

  h1 {
    font-size: 18px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .subtitle {
    font-size: 12px;
    color: var(--color-muted);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
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
    align-items: center;
    gap: 10px;
    padding: 10px;
  }

  .offer-link {
    flex: 1;
    min-width: 0;
    display: flex;
    gap: 10px;
    text-decoration: none;
    color: inherit;
  }

  .offer-stock {
    font-size: 11px;
    color: var(--color-muted);
  }

  .stepper {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .stepper button {
    width: 34px;
    height: 34px;
    border-radius: 10px;
    border: 1px solid var(--color-border);
    background: #fff;
    font-size: 18px;
    font-weight: 700;
    color: var(--color-ink);
    cursor: pointer;
  }

  .stepper button:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .qty {
    min-width: 16px;
    text-align: center;
    font-weight: 700;
  }

  .reserve-all {
    width: 100%;
  }

  .hint {
    font-size: 12px;
    color: var(--color-muted);
    text-align: center;
    margin-top: -6px;
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
