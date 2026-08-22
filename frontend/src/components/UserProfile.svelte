<script lang="ts">
  import Photo from "./Photo.svelte";
  import { merchants } from "../lib/mock";
</script>

<div class="screen">
  <div class="identity">
    <div class="avatar">
      <Photo shape="circle" label="Photo" />
    </div>
    <div>
      <h1>Salut, Léa 👋</h1>
      <p class="subtitle">Membre depuis mars 2026</p>
    </div>
  </div>
  <div class="stats">
    <div class="card stat">
      <div class="stat-value">27</div>
      <div class="stat-label">paniers sauvés</div>
    </div>
    <div class="card stat">
      <div class="stat-value">61 €</div>
      <div class="stat-label">économisés</div>
    </div>
  </div>
  <div class="section-title">Commerçants suivis</div>
  <div class="list">
    {#each merchants as merchant}
      <a class="card row" href={`/marchand/${merchant.id}`}>
        <div class="row-photo">
          <Photo shape="rounded" radius={10} label="Logo" />
        </div>
        <div class="row-name">{merchant.name}</div>
        <span class="dot" class:on={merchant.followed}></span>
      </a>
    {/each}
  </div>
</div>

<style>
  .screen {
    flex: 1;
    padding: 24px 20px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .identity {
    display: flex;
    align-items: center;
    gap: 14px;
  }

  .avatar {
    width: 60px;
    height: 60px;
    flex-shrink: 0;
  }

  h1 {
    font-size: 18px;
  }

  .subtitle {
    font-size: 12px;
    color: var(--color-muted);
  }

  .stats {
    display: flex;
    gap: 10px;
  }

  .stat {
    flex: 1;
    text-align: center;
    padding: 14px;
  }

  .stat-value {
    font-family: var(--font-display);
    font-size: 22px;
    font-weight: 600;
    color: var(--color-primary);
  }

  .stat-label {
    font-size: 11px;
    color: var(--color-muted-light);
    font-weight: 600;
  }

  .section-title {
    font-size: 13px;
    font-weight: 700;
    color: var(--color-muted-light);
  }

  .list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px;
    text-decoration: none;
    color: inherit;
  }

  .row-photo {
    width: 40px;
    height: 40px;
    flex-shrink: 0;
  }

  .row-name {
    flex: 1;
    font-size: 13px;
    font-weight: 600;
    color: var(--color-ink);
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 4px;
    background: var(--color-border);
  }

  .dot.on {
    background: var(--color-primary);
  }
</style>
