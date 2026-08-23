<script lang="ts">
  import { onMount } from "svelte";
  import { merchantLogin, merchantRegister, ApiError } from "../lib/api";
  import { setMerchantToken } from "../lib/auth";
  import { getBrowserPosition } from "../lib/geoloc";
  import { getQueryParam } from "../lib/params";

  let mode: "login" | "register" = "login";

  onMount(() => {
    const qMode = getQueryParam("mode");
    if (qMode === "login" || qMode === "register") mode = qMode;
  });
  let email = "";
  let password = "";
  let nom = "";
  let adresse = "";
  let categorie = "";
  let error = "";
  let loading = false;

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = "";
    loading = true;
    try {
      let auth;
      if (mode === "register") {
        const coords = await getBrowserPosition();
        auth = await merchantRegister({
          nom,
          adresse,
          categorie,
          email,
          password,
          latitude: coords?.lat,
          longitude: coords?.lon,
        });
      } else {
        auth = await merchantLogin(email, password);
      }
      setMerchantToken(auth.token);
      window.location.href = "/pro/dashboard";
    } catch (err) {
      error = err instanceof ApiError ? err.message : "Une erreur est survenue.";
    } finally {
      loading = false;
    }
  }
</script>

<div class="screen">
  <div class="brand">
    <div class="brand-mark">D</div>
    <div class="brand-label">ESPACE MARCHAND</div>
  </div>
  <form class="body" on:submit={submit}>
    <h1>Vos invendus, en 30 secondes.</h1>
    <div class="fields">
      {#if mode === "register"}
        <input placeholder="Nom du commerce" bind:value={nom} required />
        <input placeholder="Adresse" bind:value={adresse} required />
        <input placeholder="Catégorie (ex. Boulangerie)" bind:value={categorie} required />
      {/if}
      <input placeholder="Email professionnel" type="email" bind:value={email} required />
      <input placeholder="Mot de passe" type="password" bind:value={password} required minlength="8" />
      {#if error}<p class="error">{error}</p>{/if}
    </div>
    <button class="btn btn-primary" type="submit" disabled={loading}>
      {loading ? "..." : mode === "register" ? "Créer mon compte marchand" : "Se connecter"}
    </button>
    <button
      class="switch"
      type="button"
      on:click={() => (mode = mode === "register" ? "login" : "register")}
    >
      {mode === "register" ? "J'ai déjà un compte" : "Créer un compte marchand"}
    </button>
  </form>
</div>

<style>
  .screen {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: var(--color-ink);
    color: #fff;
    padding: 36px 24px;
    gap: 24px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .brand-label {
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 1px;
    opacity: 0.7;
  }

  .body {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 24px;
  }

  h1 {
    font-size: 26px;
    color: #fff;
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  input {
    height: 50px;
    border-radius: 14px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
    padding: 0 16px;
    font-size: 14px;
    font-family: var(--font-body);
  }

  input::placeholder {
    color: rgba(255, 255, 255, 0.5);
  }

  .error {
    color: #ff8a80;
    font-size: 13px;
  }

  .switch {
    background: none;
    border: none;
    color: rgba(255, 255, 255, 0.7);
    font-weight: 600;
    font-size: 13px;
    text-align: center;
    padding: 4px;
  }
</style>
