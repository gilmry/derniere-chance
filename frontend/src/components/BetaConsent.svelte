<script lang="ts">
  import { onMount } from "svelte";
  import {
    consentStatus,
    grantConsent,
    withdrawConsent,
    ApiError,
    type ConsentStatus,
  } from "../lib/api";
  import { getConsumerToken, clearConsumerToken } from "../lib/auth";
  import { getQueryParam } from "../lib/params";

  let token: string | null = null;
  let status: ConsentStatus | null = null;
  let checked = false;
  let confirmingWithdrawal = false;
  let loading = true;
  let submitting = false;
  let error = "";
  let next = "/feed";

  onMount(async () => {
    next = getQueryParam("next") ?? next;
    token = getConsumerToken();
    if (!token) {
      loading = false;
      return;
    }
    try {
      status = await consentStatus(token);
    } catch (err) {
      error = err instanceof ApiError ? err.message : "Impossible de lire ton consentement.";
    }
    loading = false;
  });

  async function accept() {
    if (!token || !checked) return;
    error = "";
    submitting = true;
    try {
      status = await grantConsent(token);
      window.location.href = next;
    } catch (err) {
      error = err instanceof ApiError ? err.message : "Une erreur est survenue.";
    } finally {
      submitting = false;
    }
  }

  async function withdraw() {
    if (!token) return;
    error = "";
    submitting = true;
    try {
      await withdrawConsent(token);
      // Le compte est anonymisé côté serveur : le jeton local ne sert plus à
      // rien et le garder induirait en erreur au prochain chargement.
      clearConsumerToken();
      window.location.href = "/?consentement=retire";
    } catch (err) {
      error = err instanceof ApiError ? err.message : "Une erreur est survenue.";
      submitting = false;
    }
  }

  function formatDate(iso: string | null): string {
    if (!iso) return "";
    return new Date(iso).toLocaleString("fr-FR", {
      dateStyle: "long",
      timeStyle: "short",
    });
  }
</script>

<div class="screen">
  <h1>Programme bêta</h1>

  {#if loading}
    <p class="state">Chargement...</p>
  {:else if !token}
    <div class="card">
      <p>
        Connecte-toi pour gérer ta participation au programme bêta de
        DernièreChance.
      </p>
      <a class="btn btn-primary" href="/compte?mode=login">Se connecter</a>
    </div>
  {:else if status?.consenti}
    <div class="card">
      <h2>Tu participes au programme bêta</h2>
      <p>
        Consentement donné le {formatDate(status.accepte_le)} (version
        <code>{status.version_acceptee}</code> de la politique de
        confidentialité).
      </p>
      <a class="btn btn-primary" href={next}>Retourner à l'application</a>
    </div>

    <div class="card">
      <h2>Retirer mon consentement</h2>
      <p>
        Tu peux retirer ton consentement à tout moment, sans avoir à te
        justifier. Nous n'aurons alors plus de base légale pour traiter tes
        données : <strong>ton compte est immédiatement anonymisé</strong>, ton
        adresse email est effacée et tu ne peux plus te reconnecter. Tes
        réservations passées restent comptabilisées chez les commerçants
        concernés, mais sans aucun lien avec toi.
      </p>
      <p>Cette opération est définitive et ne peut pas être annulée.</p>
      {#if confirmingWithdrawal}
        <p class="warn">Confirmer le retrait et la suppression de mon compte ?</p>
        <div class="actions">
          <button class="btn btn-danger" type="button" on:click={withdraw} disabled={submitting}>
            {submitting ? "..." : "Oui, retirer et supprimer"}
          </button>
          <button
            class="btn btn-secondary"
            type="button"
            on:click={() => (confirmingWithdrawal = false)}
            disabled={submitting}
          >
            Annuler
          </button>
        </div>
      {:else}
        <button
          class="btn btn-secondary"
          type="button"
          on:click={() => (confirmingWithdrawal = true)}
        >
          Retirer mon consentement
        </button>
      {/if}
    </div>
  {:else}
    <div class="card">
      <h2>
        {status?.version_acceptee
          ? "La politique de confidentialité a été mise à jour"
          : "Un accord est nécessaire pour continuer"}
      </h2>
      <p>
        {#if status?.version_acceptee}
          Tu avais accepté la version <code>{status.version_acceptee}</code>. Le
          texte a changé depuis : merci de relire la nouvelle version et de
          confirmer ton accord pour continuer à utiliser l'application.
        {:else}
          DernièreChance est en phase de test. Pour continuer, il faut accepter
          de participer au programme bêta et la collecte de données qui va avec.
        {/if}
      </p>
      <p>
        Tout est détaillé dans la
        <a href="/confidentialite">politique de confidentialité</a> : ce qui est
        collecté, pourquoi, combien de temps, et comment tout effacer.
      </p>
      <label class="consent">
        <input type="checkbox" bind:checked={checked} />
        <span>
          J'accepte de participer au programme bêta et la collecte de mes données
          telles que décrites dans la politique de confidentialité.
        </span>
      </label>
      <button
        class="btn btn-primary"
        type="button"
        on:click={accept}
        disabled={!checked || submitting}
      >
        {submitting ? "..." : "Je participe"}
      </button>
    </div>
  {/if}

  {#if error}<p class="state error">{error}</p>{/if}
</div>

<style>
  .screen {
    padding: 24px 20px 40px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  h1 {
    font-size: 22px;
  }

  h2 {
    font-size: 15px;
    margin-bottom: 8px;
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 16px;
  }

  .card p {
    font-size: 13px;
    color: var(--color-muted);
    line-height: 1.6;
  }

  .state {
    padding: 8px 0;
    color: var(--color-muted);
    font-size: 13px;
  }

  .state.error {
    color: #c0392b;
  }

  .warn {
    font-weight: 600;
  }

  .actions {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .btn-danger {
    background: #c0392b;
    color: #fff;
  }

  .consent {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    font-size: 13px;
    line-height: 1.5;
    padding: 4px 0;
  }

  .consent input {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    margin-top: 2px;
    accent-color: var(--color-primary);
  }

  code {
    font-family: monospace;
    background: var(--color-chip);
    padding: 1px 5px;
    border-radius: 4px;
  }
</style>
