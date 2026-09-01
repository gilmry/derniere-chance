<script lang="ts">
  import { forgotPassword, ApiError } from "../lib/api";

  let email = "";
  let sent = false;
  let error = "";
  let loading = false;

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    error = "";
    loading = true;
    try {
      await forgotPassword(email);
      // Confirmation volontairement identique que le compte existe ou non :
      // afficher « adresse inconnue » ferait de cet écran un moyen de savoir
      // qui est inscrit.
      sent = true;
    } catch (err) {
      error = err instanceof ApiError ? err.message : "Une erreur est survenue.";
    } finally {
      loading = false;
    }
  }
</script>

<div class="screen">
  <div class="body">
    <h1>Mot de passe oublié</h1>
    {#if sent}
      <p class="lead">
        Si un compte existe pour <strong>{email}</strong>, un lien vient d'y être
        envoyé. Il est valable une heure et ne fonctionne qu'une fois.
      </p>
      <p class="hint">
        Pensez à regarder dans les indésirables. Si rien n'arrive, c'est
        probablement qu'aucun compte n'utilise cette adresse.
      </p>
      <a class="btn btn-primary" href="/compte?mode=login">Retour à la connexion</a>
    {:else}
      <p class="lead">
        Indiquez l'adresse de votre compte, client ou commerçant. Nous vous
        enverrons un lien pour choisir un nouveau mot de passe.
      </p>
      <form class="fields" on:submit={submit}>
        <input placeholder="Email" type="email" bind:value={email} required />
        {#if error}<p class="error">{error}</p>{/if}
        <button class="btn btn-primary" type="submit" disabled={loading}>
          {loading ? "..." : "Envoyer le lien"}
        </button>
      </form>
      <a class="switch" href="/compte?mode=login">Retour à la connexion</a>
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

  .hint {
    font-size: 12px;
    line-height: 1.6;
    color: var(--color-muted);
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
