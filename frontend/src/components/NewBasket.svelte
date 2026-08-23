<script lang="ts">
  import { publishProduct, uploadProductPhoto, ApiError } from "../lib/api";
  import { getMerchantToken } from "../lib/auth";

  // Aperçu local instantané (blob) pendant l'upload ; remplacé par
  // uploadedPhotoUrl (URL publique MinIO) une fois l'upload terminé, envoyé
  // avec la publication.
  let photoPreview: string | null = null;
  let uploadedPhotoUrl: string | null = null;
  let uploadingPhoto = false;
  let photoError = "";
  let fileInput: HTMLInputElement;

  let nom = "";
  let description = "";
  let prixInitial = "8.00";
  let prixDemarque = "3.20";
  let quantity = 5;
  let retraitDebut = "";
  let retraitFin = "";
  let error = "";
  let loading = false;

  async function onPhotoChange(event: Event) {
    const file = (event.target as HTMLInputElement).files?.[0];
    if (!file) return;

    photoPreview = URL.createObjectURL(file);
    uploadedPhotoUrl = null;
    photoError = "";
    uploadingPhoto = true;

    const token = getMerchantToken();
    if (!token) {
      window.location.href = "/pro/login";
      return;
    }

    try {
      uploadedPhotoUrl = await uploadProductPhoto(file, token);
    } catch (err) {
      photoError = err instanceof ApiError ? err.message : "L'envoi de la photo a échoué.";
    } finally {
      uploadingPhoto = false;
    }
  }

  function decrement() {
    quantity = Math.max(0, quantity - 1);
  }

  function increment() {
    quantity += 1;
  }

  // "HH:MM" (aujourd'hui, heure locale) -> ISO UTC attendu par le backend.
  function timeToIso(time: string): string | null {
    if (!time) return null;
    const [h, m] = time.split(":").map(Number);
    const d = new Date();
    d.setHours(h, m, 0, 0);
    return d.toISOString();
  }

  async function publish(event: SubmitEvent) {
    event.preventDefault();
    const token = getMerchantToken();
    if (!token) {
      window.location.href = "/pro/login";
      return;
    }
    const debut = timeToIso(retraitDebut);
    const fin = timeToIso(retraitFin);
    if (!debut || !fin) {
      error = "Renseigne la fenêtre de retrait.";
      return;
    }
    error = "";
    loading = true;
    try {
      await publishProduct(
        {
          nom,
          description,
          prix_initial: prixInitial,
          prix_demarque: prixDemarque,
          quantite: quantity,
          retrait_debut: debut,
          retrait_fin: fin,
          photo_url: uploadedPhotoUrl,
        },
        token,
      );
      window.location.href = "/pro/dashboard";
    } catch (err) {
      error = err instanceof ApiError ? err.message : "La publication a échoué.";
    } finally {
      loading = false;
    }
  }

  const now = new Date();
  const time = `${now.getHours()}:${String(now.getMinutes()).padStart(2, "0")}`;
</script>

<div class="screen">
  <div class="header">
    <h1>Nouveau panier</h1>
    <span class="time">{time}</span>
  </div>

  <form on:submit={publish} class="form">
    <button class="photo" on:click={() => fileInput.click()} type="button">
      {#if photoPreview}
        <img src={photoPreview} alt="Photo du produit" />
        {#if uploadingPhoto}<span class="photo-status">Envoi...</span>{/if}
      {:else}
        <span>Prendre une photo du produit</span>
      {/if}
    </button>
    {#if photoError}<p class="error">{photoError}</p>{/if}
    <input
      class="visually-hidden"
      type="file"
      accept="image/jpeg,image/png,image/webp"
      bind:this={fileInput}
      on:change={onPhotoChange}
    />

    <div class="field">
      <div class="field-label">NOM DU PANIER</div>
      <input class="text-input" bind:value={nom} placeholder="Panier boulanger surprise" required />
    </div>

    <div class="field">
      <div class="field-label">DESCRIPTION</div>
      <input class="text-input" bind:value={description} placeholder="Ce qu'il contient" required />
    </div>

    <div class="field">
      <div class="field-label">PRIX BARRÉ → PRIX PROMO (€)</div>
      <div class="price-inputs">
        <input class="price-old-input" type="number" step="0.01" min="0" bind:value={prixInitial} required />
        <input class="price-new-input" type="number" step="0.01" min="0" bind:value={prixDemarque} required />
      </div>
    </div>

    <div class="field">
      <div class="field-label">FENÊTRE DE RETRAIT (AUJOURD'HUI)</div>
      <div class="price-inputs">
        <input class="text-input" type="time" bind:value={retraitDebut} required />
        <input class="text-input" type="time" bind:value={retraitFin} required />
      </div>
    </div>

    <div class="field">
      <div class="field-label">QUANTITÉ</div>
      <div class="stepper">
        <button type="button" on:click={decrement}>–</button>
        <div class="stepper-value">{quantity}</div>
        <button type="button" on:click={increment}>+</button>
      </div>
    </div>

    {#if error}<p class="error">{error}</p>{/if}

    <div class="spacer"></div>
    <button class="btn btn-primary publish" type="submit" disabled={loading || uploadingPhoto}>
      {loading ? "..." : uploadingPhoto ? "Envoi de la photo..." : "Publier maintenant"}
    </button>
  </form>
</div>

<style>
  .screen {
    flex: 1;
    display: flex;
    flex-direction: column;
    background: #141914;
    color: #fff;
    padding: 20px;
    gap: 16px;
    overflow-y: auto;
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: 16px;
    flex: 1;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  h1 {
    font-size: 19px;
    color: #fff;
  }

  .time {
    font-size: 13px;
    color: #8a9188;
  }

  .photo {
    position: relative;
    width: 100%;
    height: 190px;
    border-radius: 20px;
    background: #1d231d;
    border: 1px dashed #2a322a;
    color: #8a9188;
    font-size: 13px;
    font-family: var(--font-body);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
    overflow: hidden;
  }

  .photo img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .photo-status {
    position: absolute;
    bottom: 10px;
    right: 10px;
    background: rgba(0, 0, 0, 0.6);
    color: #fff;
    font-size: 11px;
    font-weight: 700;
    padding: 4px 10px;
    border-radius: 8px;
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .field-label {
    font-size: 12px;
    font-weight: 700;
    color: #8a9188;
  }

  .text-input {
    height: 52px;
    border-radius: 14px;
    border: 1px solid #2a322a;
    background: #1d231d;
    color: #fff;
    padding: 0 14px;
    font-size: 14px;
    font-family: var(--font-body);
    flex: 1;
  }

  .price-inputs {
    display: flex;
    gap: 10px;
  }

  .price-inputs input {
    flex: 1;
    height: 52px;
    border-radius: 14px;
    border: 1px solid #2a322a;
    background: #1d231d;
    color: #fff;
    text-align: center;
    font-size: 16px;
    font-family: var(--font-body);
  }

  .price-old-input {
    text-decoration: line-through;
  }

  .price-new-input {
    border-color: var(--color-primary) !important;
    color: #4ade80 !important;
    font-size: 18px !important;
    font-weight: 800;
  }

  .stepper {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: #1d231d;
    border-radius: 14px;
    padding: 6px;
  }

  .stepper button {
    width: 44px;
    height: 44px;
    border-radius: 12px;
    background: #2a322a;
    color: #fff;
    font-size: 20px;
    border: none;
  }

  .stepper-value {
    font-size: 20px;
    font-weight: 800;
  }

  .error {
    color: #ff8a80;
    font-size: 13px;
  }

  .spacer {
    flex: 1;
  }

  .publish {
    height: 56px;
    font-weight: 800;
  }
</style>
