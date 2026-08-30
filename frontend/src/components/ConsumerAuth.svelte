<script lang="ts">
  import { onMount } from "svelte";
  import { consumerLogin, consumerRegister, ApiError } from "../lib/api";
  import { setConsumerToken, setConsumerEmail } from "../lib/auth";
  import { getQueryParam } from "../lib/params";

  export let mode: "login" | "register" = "register";
  export let next = "/feed";

  onMount(() => {
    const qMode = getQueryParam("mode");
    if (qMode === "login" || qMode === "register") mode = qMode;
    next = getQueryParam("next") ?? next;
  });

  let email = "";
  let password = "";
  // Jamais pré-cochée : le consentement doit être un acte positif (RGPD
  // art. 4 §11). L'attribut `required` sur la case empêche la soumission
  // tant qu'elle ne l'est pas, et le backend revérifie de son côté.
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
      const auth = mode === "register"
        ? await consumerRegister(email, password)
        : await consumerLogin(email, password);
      setConsumerToken(auth.token);
      setConsumerEmail(email);
      window.location.href = next;
    } catch (err) {
      error = err instanceof ApiError ? err.message : "Une erreur est survenue.";
    } finally {
      loading = false;
    }
  }
</script>

<div class="screen">
  <div class="body">
    <h1>{mode === "register" ? "Créer un compte" : "Se connecter"}</h1>
    <form class="fields" on:submit={submit}>
      <input placeholder="Email" type="email" bind:value={email} required />
      <input placeholder="Mot de passe" type="password" bind:value={password} required minlength="8" />
      {#if mode === "register"}
        <label class="consent">
          <input type="checkbox" bind:checked={consent} required />
          <span>
            J'accepte de participer au programme bêta et la collecte de mes données
            telles que décrites dans la
            <a href="/confidentialite" target="_blank" rel="noopener">politique de confidentialité</a>.
          </span>
        </label>
      {/if}
      {#if error}<p class="error">{error}</p>{/if}
      <button class="btn btn-primary" type="submit" disabled={loading}>
        {loading ? "..." : mode === "register" ? "Créer mon compte" : "Se connecter"}
      </button>
    </form>
    <button class="switch" type="button" on:click={() => (mode = mode === "register" ? "login" : "register")}>
      {mode === "register" ? "J'ai déjà un compte" : "Créer un compte"}
    </button>
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
    gap: 24px;
  }

  h1 {
    font-size: 24px;
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

  .consent {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    font-size: 12px;
    line-height: 1.5;
    color: var(--color-muted);
    padding: 2px;
  }

  .consent input {
    width: 18px;
    height: 18px;
    flex-shrink: 0;
    margin-top: 1px;
    accent-color: var(--color-primary);
  }

  .consent a {
    color: var(--color-primary);
    font-weight: 600;
  }

  .error {
    color: #c0392b;
    font-size: 13px;
  }

  .switch {
    background: none;
    border: none;
    color: var(--color-primary);
    font-weight: 600;
    font-size: 14px;
    text-align: center;
    padding: 8px;
  }
</style>
