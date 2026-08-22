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
  let error = "";
  let loading = false;

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = "";
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
