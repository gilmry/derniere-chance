<script lang="ts">
  import { onMount } from "svelte";
  import Photo from "./Photo.svelte";
  import {
    consumerProfile,
    listFollowedMerchants,
    listMyReservations,
    formatPrice,
    ApiError,
    type ConsumerProfile,
    type Merchant,
    type ReservationSummary,
  } from "../lib/api";
  import { getConsumerToken, getConsumerEmail } from "../lib/auth";

  let profile: ConsumerProfile | null = null;
  let followed: Merchant[] = [];
  let reservations: ReservationSummary[] = [];
  let loading = true;
  let loadError = "";
  let email = "";

  const STATUT_LABELS: Record<string, string> = {
    reservee: "À retirer",
    recuperee: "Récupéré",
    expiree: "Expiré",
  };

  function pickupWindow(r: ReservationSummary): string {
    const opts: Intl.DateTimeFormatOptions = { hour: "2-digit", minute: "2-digit" };
    const start = new Date(r.retrait_debut).toLocaleTimeString("fr-FR", opts);
    const end = new Date(r.retrait_fin).toLocaleTimeString("fr-FR", opts);
    return `${start} – ${end}`;
  }

  onMount(async () => {
    const token = getConsumerToken();
    email = getConsumerEmail() ?? "";
    if (!token) {
      loading = false;
      return;
    }
    try {
      [profile, followed, reservations] = await Promise.all([
        consumerProfile(token),
        listFollowedMerchants(token),
        listMyReservations(token),
      ]);
    } catch (err) {
      loadError = err instanceof ApiError ? err.message : "Impossible de charger ton profil.";
    } finally {
      loading = false;
    }
  });
</script>

<div class="screen">
  {#if loading}
    <p class="state">Chargement...</p>
  {:else if !getConsumerToken()}
    <div class="signed-out">
      <p>Connecte-toi pour voir ton profil et tes commerçants suivis.</p>
      <a class="btn btn-primary" href="/compte?mode=login">Se connecter</a>
    </div>
  {:else if loadError}
    <p class="state error">{loadError}</p>
  {:else}
    <div class="identity">
      <div class="avatar">
        <Photo shape="circle" label="Photo" />
      </div>
      <div>
        <h1>Salut 👋</h1>
        {#if email}<p class="subtitle">{email}</p>{/if}
      </div>
    </div>
    <div class="stats">
      <div class="card stat">
        <div class="stat-value">{profile?.paniers_sauves ?? 0}</div>
        <div class="stat-label">paniers sauvés</div>
      </div>
      <div class="card stat">
        <div class="stat-value">{formatPrice(profile?.montant_economise ?? "0")}</div>
        <div class="stat-label">économisés</div>
      </div>
    </div>
    <div class="section-title">Mes réservations</div>
    <div class="list">
      {#each reservations as r}
        <div class="card reservation-row">
          <div class="reservation-info">
            <div class="row-name">{r.produit_nom} <span class="status-badge {r.statut === 'reservee' ? 'active' : 'exhausted'}">{STATUT_LABELS[r.statut] ?? r.statut}</span></div>
            <div class="row-detail">{r.marchand_nom} · retrait {pickupWindow(r)} · {formatPrice(r.prix_demarque)}</div>
          </div>
          {#if r.statut === "reservee"}
            <div class="code">{r.code}</div>
          {/if}
        </div>
      {/each}
      {#if reservations.length === 0}
        <p class="empty">Aucune réservation pour le moment.</p>
      {/if}
    </div>

    <div class="section-title">Commerçants suivis</div>
    <div class="list">
      {#each followed as merchant}
        <a class="card row" href={`/marchand?id=${merchant.id}`}>
          <div class="row-photo">
            <Photo shape="rounded" radius={10} label="Logo" />
          </div>
          <div class="row-name">{merchant.nom}</div>
        </a>
      {/each}
      {#if followed.length === 0}
        <p class="empty">Tu ne suis aucun commerçant pour le moment.</p>
      {/if}
    </div>
  {/if}
</div>

<style>
  .screen {
    flex: 1;
    padding: 24px 20px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .state {
    padding: 40px 0;
    text-align: center;
    color: var(--color-muted);
  }

  .state.error {
    color: #c0392b;
  }

  .signed-out {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    text-align: center;
    color: var(--color-muted);
  }

  .identity {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .avatar {
    width: 60px;
    height: 60px;
    flex-shrink: 0;
  }

  h1 {
    font-size: 18px;
  }

  .subtitle {
    font-size: 12px;
    color: var(--color-muted);
  }

  .stats {
    display: flex;
    gap: 10px;
  }

  .stat {
    flex: 1;
    text-align: center;
    padding: 14px;
  }

  .stat-value {
    font-family: var(--font-display);
    font-size: 22px;
    font-weight: 600;
    color: var(--color-primary);
  }

  .stat-label {
    font-size: 11px;
    color: var(--color-muted-light);
    font-weight: 600;
  }

  .section-title {
    font-size: 13px;
    font-weight: 700;
    color: var(--color-muted-light);
  }

  .empty {
    color: var(--color-muted-light);
    font-size: 14px;
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
    text-decoration: none;
    color: inherit;
  }

  .row-photo {
    width: 40px;
    height: 40px;
    flex-shrink: 0;
  }

  .row-name {
    flex: 1;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-ink);
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .reservation-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px;
  }

  .reservation-info {
    flex: 1;
    min-width: 0;
  }

  .row-detail {
    font-size: 11px;
    color: var(--color-muted);
  }

  .code {
    font-family: var(--font-display);
    font-size: 15px;
    font-weight: 700;
    color: var(--color-primary);
    background: var(--color-success-bg);
    padding: 6px 10px;
    border-radius: 10px;
    flex-shrink: 0;
  }
</style>
