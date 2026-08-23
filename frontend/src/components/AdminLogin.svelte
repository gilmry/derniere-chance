<script lang="ts">
  import { adminLogin, ApiError } from "../lib/api";
  import { setAdminToken } from "../lib/auth";

  let email = "";
  let password = "";
  let error = "";
  let loading = false;

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = "";
    loading = true;
    try {
      const auth = await adminLogin(email, password);
      setAdminToken(auth.token);
      window.location.href = "/admin";
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
    <div class="brand-label">BACKOFFICE ADMIN</div>
  </div>
  <form class="body" on:submit={submit}>
    <div class="fields">
      <input placeholder="Email" type="email" bind:value={email} required />
      <input placeholder="Mot de passe" type="password" bind:value={password} required />
      {#if error}<p class="error">{error}</p>{/if}
    </div>
    <button class="btn btn-primary" type="submit" disabled={loading}>
      {loading ? "..." : "Se connecter"}
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
</style>
