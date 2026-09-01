<script lang="ts">
  import { onMount } from "svelte";
  import { merchantLogin, merchantRegister, ApiError, MERCHANT_CATEGORIES } from "../lib/api";
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
  // Jamais pré-cochée : le consentement doit être un acte positif (RGPD
  // art. 4 §11). Le commerçant confie plus qu'un client - nom, adresse et
  // position, tous publiés sur la carte - donc la case le dit explicitement.
  let consent = false;
  let error = "";
  let loading = false;

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = "";
    if (mode === "register" && !consent) {
      error = "Il faut accepter de participer au programme bêta pour créer un compte.";
      return;
    }
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
        <select bind:value={categorie} required>
          <option value="" disabled selected>Catégorie</option>
          {#each Object.entries(MERCHANT_CATEGORIES) as [cat, emoji]}
            <option value={cat}>{emoji} {cat}</option>
          {/each}
        </select>
      {/if}
      <input placeholder="Email professionnel" type="email" bind:value={email} required />
      <input placeholder="Mot de passe" type="password" bind:value={password} required minlength="8" />
      {#if mode === "register"}
        <label class="consent">
          <input type="checkbox" bind:checked={consent} required />
          <span>
            J'accepte de participer au programme bêta et la collecte de mes données,
            dont le nom, l'adresse et la position de mon commerce affichés publiquement,
            telles que décrites dans la
            <a href="/confidentialite" target="_blank" rel="noopener">politique de confidentialité</a>.
          </span>
        </label>
      {/if}
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
    {#if mode === "login"}
      <a class="forgot" href="/mot-de-passe-oublie">Mot de passe oublié ?</a>
    {/if}
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

  input,
  select {
    height: 50px;
    border-radius: 14px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
    padding: 0 16px;
    font-size: 14px;
    font-family: var(--font-body);
  }

  select option {
    color: #000;
  }

  input::placeholder {
    color: rgba(255, 255, 255, 0.5);
  }

  .consent {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    font-size: 12px;
    line-height: 1.5;
    color: rgba(255, 255, 255, 0.75);
    padding: 2px;
  }

  .consent input {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    margin-top: 1px;
    padding: 0;
    accent-color: var(--color-primary);
  }

  .consent a {
    color: #fff;
    font-weight: 600;
  }

  .error {
    color: #ff8a80;
    font-size: 13px;
  }

  .forgot {
    color: var(--color-muted);
    font-size: 13px;
    text-align: center;
    padding: 4px;
    text-decoration: none;
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
