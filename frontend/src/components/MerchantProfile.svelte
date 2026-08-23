<script lang="ts">
  import { onMount } from "svelte";
  import Photo from "./Photo.svelte";
  import BottomNav from "./BottomNav.svelte";
  import {
    getMerchantProfile,
    followMerchant,
    unfollowMerchant,
    listFollowedMerchants,
    formatPrice,
    formatDistance,
    ApiError,
    type MerchantProfile,
  } from "../lib/api";
  import { getConsumerToken } from "../lib/auth";
  import { getQueryParam } from "../lib/params";
  import { getBrowserPosition } from "../lib/geoloc";

  let merchant: MerchantProfile | null = null;
  let loading = true;
  let loadError = "";
  let following = false;
  let followBusy = false;
  let merchantId = "";

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
    <div class="cover">
      <Photo shape="rect" label="Photo de couverture du commerce" />
    </div>
    <div class="body">
      <div class="identity">
        <div class="avatar">
          <Photo shape="rounded" radius={16} label="Logo" src={merchant.logo_url} />
        </div>
        <div class="identity-text">
          <h1>{merchant.nom}</h1>
          <p class="subtitle">
            {merchant.categorie} · {merchant.adresse}{merchant.note ? ` · ⭐ ${merchant.note}` : ""}{formatDistance(merchant.distance_km) ? ` · ${formatDistance(merchant.distance_km)}` : ""}
          </p>
        </div>
      </div>
      <button class="btn btn-dark follow" class:following on:click={toggleFollow} disabled={followBusy}>
        {following ? "✓ Abonné" : "+ S'abonner"}
      </button>
      <div class="section-title">Offres actives</div>
      <div class="offers">
        {#each merchant.offres.filter((o) => o.statut === "publie") as offer}
          <a class="card offer-row" href={`/offre?id=${offer.id}`}>
            <div class="offer-photo">
              <Photo shape="rounded" radius={10} label="Photo" src={offer.photo_url} />
            </div>
            <div class="offer-info">
              <div class="offer-name">{offer.nom}</div>
              <div class="offer-price">{formatPrice(offer.prix_demarque)}</div>
            </div>
          </a>
        {/each}
        {#if merchant.offres.filter((o) => o.statut === "publie").length === 0}
          <p class="empty">Pas d'offre active pour le moment.</p>
        {/if}
      </div>
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
