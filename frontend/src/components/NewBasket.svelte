<script lang="ts">
  let photoUrl: string | null = null;
  let priceOriginal = "8,00 €";
  let pricePromo = "3,20 €";
  let quantity = 5;
  let fileInput: HTMLInputElement;

  function onPhotoChange(event: Event) {
    const file = (event.target as HTMLInputElement).files?.[0];
    if (file) {
      photoUrl = URL.createObjectURL(file);
    }
  }

  function decrement() {
    quantity = Math.max(0, quantity - 1);
  }

  function increment() {
    quantity += 1;
  }

  function publish() {
    window.location.href = "/pro/dashboard";
  }

  const now = new Date();
  const time = `${now.getHours()}:${String(now.getMinutes()).padStart(2, "0")}`;
</script>

<div class="screen">
  <div class="header">
    <h1>Nouveau panier</h1>
    <span class="time">{time}</span>
  </div>

  <button class="photo" on:click={() => fileInput.click()} type="button">
    {#if photoUrl}
      <img src={photoUrl} alt="Photo du produit" />
    {:else}
      <span>Prendre une photo du produit</span>
    {/if}
  </button>
  <input
    class="visually-hidden"
    type="file"
    accept="image/*"
    bind:this={fileInput}
    on:change={onPhotoChange}
  />

  <div class="field">
    <div class="field-label">PRIX BARRÉ → PRIX PROMO</div>
    <div class="price-inputs">
      <input class="price-old-input" bind:value={priceOriginal} />
      <input class="price-new-input" bind:value={pricePromo} />
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

  <div class="spacer"></div>
  <button class="btn btn-primary publish" on:click={publish}>Publier maintenant</button>
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

  .spacer {
    flex: 1;
  }

  .publish {
    height: 56px;
    font-weight: 800;
  }
</style>
