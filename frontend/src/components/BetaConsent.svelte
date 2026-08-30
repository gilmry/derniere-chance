<script lang="ts">
  import { onMount } from "svelte";
  import {
    consentStatus,
    grantConsent,
    withdrawConsent,
    ApiError,
    type ConsentRole,
    type ConsentStatus,
  } from "../lib/api";
  import {
    getConsumerToken,
    clearConsumerToken,
    getMerchantToken,
    clearMerchantToken,
  } from "../lib/auth";
  import { getQueryParam } from "../lib/params";

  /// Même écran pour les deux principaux : le circuit est identique, seuls
  /// les données en jeu et l'espace de retour changent.
  export let role: ConsentRole = "consommateur";

  $: marchand = role === "marchand";

  let token: string | null = null;
  let status: ConsentStatus | null = null;
  let checked = false;
  let confirmingWithdrawal = false;
  let loading = true;
  let submitting = false;
  let error = "";
  let next = "/feed";

  onMount(async () => {
    if (!getQueryParam("next") && marchand) next = "/pro/dashboard";
    next = getQueryParam("next") ?? next;
    token = marchand ? getMerchantToken() : getConsumerToken();
    if (!token) {
      loading = false;
      return;
    }
    try {
      status = await consentStatus(token, role);
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
      status = await grantConsent(token, role);
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
      await withdrawConsent(token, role);
      // Le compte est anonymisé côté serveur : le jeton local ne sert plus à
      // rien et le garder induirait en erreur au prochain chargement.
      if (marchand) clearMerchantToken();
      else clearConsumerToken();
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
        {marchand
          ? "Connectez-vous pour gérer la participation de votre commerce au programme bêta de DernièreChance."
          : "Connecte-toi pour gérer ta participation au programme bêta de DernièreChance."}
      </p>
      <a class="btn btn-primary" href={marchand ? "/pro/login?mode=login" : "/compte?mode=login"}>
        Se connecter
      </a>
    </div>
  {:else if status?.consenti}
    <div class="card">
      <h2>{marchand ? "Votre commerce participe au programme bêta" : "Tu participes au programme bêta"}</h2>
      <p>
        Consentement donné le {formatDate(status.accepte_le)} (version
        <code>{status.version_acceptee}</code> de la politique de
        confidentialité).
      </p>
      <a class="btn btn-primary" href={next}>
        {marchand ? "Retourner au tableau de bord" : "Retourner à l'application"}
      </a>
    </div>

    <div class="card">
      <h2>Retirer mon consentement</h2>
      {#if marchand}
        <p>
          Vous pouvez retirer votre consentement à tout moment, sans avoir à
          vous justifier. Nous n'aurons alors plus de base légale pour traiter
          vos données : <strong>vos paniers encore en ligne sont retirés de la
          carte</strong> et <strong>votre compte est immédiatement
          anonymisé</strong>. Le nom, l'adresse et la position de votre
          commerce sont effacés, votre email aussi, et vous ne pouvez plus
          vous reconnecter. Les paniers déjà retirés par des clients restent
          comptabilisés, mais sans aucun lien avec votre commerce.
        </p>
      {:else}
        <p>
          Tu peux retirer ton consentement à tout moment, sans avoir à te
          justifier. Nous n'aurons alors plus de base légale pour traiter tes
          données : <strong>ton compte est immédiatement anonymisé</strong>, ton
          adresse email est effacée et tu ne peux plus te reconnecter. Tes
          réservations passées restent comptabilisées chez les commerçants
          concernés, mais sans aucun lien avec toi.
        </p>
      {/if}
      <p>Cette opération est définitive et ne peut pas être annulée.</p>
      {#if confirmingWithdrawal}
        <p class="warn">
          {marchand
            ? "Confirmer le retrait, la dépublication des paniers et la suppression du compte ?"
            : "Confirmer le retrait et la suppression de mon compte ?"}
        </p>
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
        {:else if marchand}
          DernièreChance est en phase de test. Pour continuer, il faut accepter
          de participer au programme bêta et la collecte de données qui va avec,
          dont le nom, l'adresse et la position de votre commerce, affichés
          publiquement sur la carte.
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
          {#if marchand}
            J'accepte de participer au programme bêta et la collecte de mes données,
            dont le nom, l'adresse et la position de mon commerce affichés
            publiquement, telles que décrites dans la politique de confidentialité.
          {:else}
            J'accepte de participer au programme bêta et la collecte de mes données
            telles que décrites dans la politique de confidentialité.
          {/if}
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

    <div class="card">
      <h2>{marchand ? "Je ne souhaite pas participer" : "Je ne veux pas participer"}</h2>
      <p>
        {#if marchand}
          Refuser est votre droit, et cela ne doit pas vous laisser avec un
          compte inutilisable. Vous pouvez faire effacer le vôtre dès
          maintenant : <strong>vos paniers encore en ligne sont retirés de la
          carte</strong> et le nom, l'adresse, la position et l'email de votre
          commerce sont remplacés par des valeurs neutres.
        {:else}
          Refuser est ton droit, et ça ne doit pas te laisser avec un compte
          inutilisable. Tu peux faire effacer le tien dès maintenant : ton
          adresse email est effacée et tu ne peux plus te reconnecter.
        {/if}
      </p>
      <p>Cette opération est définitive et ne peut pas être annulée.</p>
      {#if confirmingWithdrawal}
        <p class="warn">Confirmer la suppression du compte ?</p>
        <div class="actions">
          <button class="btn btn-danger" type="button" on:click={withdraw} disabled={submitting}>
            {submitting ? "..." : "Oui, supprimer mon compte"}
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
          Supprimer mon compte
        </button>
      {/if}
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
