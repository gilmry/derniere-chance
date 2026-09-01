<script lang="ts">
  import { onMount } from "svelte";
  import { resetPassword, MIN_PASSWORD_LENGTH, ApiError } from "../lib/api";
  import { getQueryParam } from "../lib/params";

  // Le jeton vit dans la query string : la sortie statique d'Astro ne connaît
  // pas les segments dynamiques à la construction (voir astro.config.mjs).
  let token: string | null = null;
  let ready = false;

  onMount(() => {
    token = getQueryParam("token");
    ready = true;
  });

  let password = "";
  let confirmation = "";
  let done = false;
  let error = "";
  let loading = false;

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = "";
    if (password !== confirmation) {
      error = "Les deux mots de passe ne sont pas identiques.";
      return;
    }
    if (!token) {
      error = "Ce lien est incomplet. Demandez-en un nouveau.";
      return;
    }
    loading = true;
    try {
      await resetPassword(token, password);
      done = true;
    } catch (err) {
      error = err instanceof ApiError ? err.message : "Une erreur est survenue.";
    } finally {
      loading = false;
    }
  }
</script>

<div class="screen">
  <div class="body">
    <h1>Nouveau mot de passe</h1>
    {#if done}
      <p class="lead">
        C'est fait. Votre mot de passe a été changé et le lien que vous venez
        d'utiliser ne fonctionne plus.
      </p>
      <a class="btn btn-primary" href="/compte?mode=login">Se connecter</a>
    {:else if ready && !token}
      <p class="lead">
        Ce lien est incomplet ; il a peut-être été coupé par votre logiciel de
        messagerie.
      </p>
      <a class="btn btn-primary" href="/mot-de-passe-oublie">Demander un nouveau lien</a>
    {:else}
      <p class="lead">
        Choisissez un mot de passe d'au moins {MIN_PASSWORD_LENGTH} caractères.
        Une phrase dont vous vous souvenez vaut mieux qu'un mot court et
        compliqué.
      </p>
      <form class="fields" on:submit={submit}>
        <input
          placeholder="Nouveau mot de passe"
          type="password"
          bind:value={password}
          required
          minlength={MIN_PASSWORD_LENGTH}
          autocomplete="new-password"
        />
        <input
          placeholder="Confirmer le mot de passe"
          type="password"
          bind:value={confirmation}
          required
          minlength={MIN_PASSWORD_LENGTH}
          autocomplete="new-password"
        />
        {#if error}<p class="error">{error}</p>{/if}
        <button class="btn btn-primary" type="submit" disabled={loading || !ready}>
          {loading ? "..." : "Changer mon mot de passe"}
        </button>
      </form>
      <a class="switch" href="/mot-de-passe-oublie">Demander un nouveau lien</a>
    {/if}
  </div>
</div>

<style>
  .screen {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 36px 24px;
  }

  .body {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 20px;
  }

  h1 {
    font-size: 24px;
  }

  .lead {
    font-size: 14px;
    line-height: 1.6;
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  input {
    height: 50px;
    border-radius: 14px;
    border: 1px solid var(--color-border);
    padding: 0 16px;
    font-size: 14px;
    font-family: var(--font-body);
  }

  .error {
    color: #c0392b;
    font-size: 13px;
  }

  .switch {
    color: var(--color-primary);
    font-weight: 600;
    font-size: 14px;
    text-align: center;
    padding: 8px;
    text-decoration: none;
  }
</style>
