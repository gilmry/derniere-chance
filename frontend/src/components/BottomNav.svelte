<script lang="ts">
  export let active: "feed" | "profil" = "feed";
</script>

<nav class="bottom-nav">
  <a href="/feed" class:active={active === "feed"}>
    <span class="icon">🏠</span>
    <span>Autour de toi</span>
  </a>
  <a href="/profil" class:active={active === "profil"}>
    <span class="icon">👤</span>
    <span>Profil</span>
  </a>
</nav>

<style>
  .bottom-nav {
    display: flex;
    margin-top: auto;
    border-top: 1px solid var(--color-border);
    background: var(--color-surface);
    flex-shrink: 0;
  }

  .bottom-nav a {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 10px 0 calc(10px + env(safe-area-inset-bottom, 0px));
    text-decoration: none;
    color: var(--color-muted-light);
    font-size: 11px;
    font-weight: 600;
  }

  .bottom-nav a.active {
    color: var(--color-primary);
  }

  .icon {
    font-size: 20px;
  }
</style>
