<script lang="ts">
  export let shape: "rect" | "rounded" | "circle" = "rounded";
  export let radius = 16;
  export let label = "";
</script>

<div
  class="photo photo--{shape}"
  style={shape === "rounded" ? `border-radius:${radius}px` : ""}
  role="img"
  aria-label={label}
>
  {#if label}<span>{label}</span>{/if}
</div>

<style>
  .photo {
    width: 100%;
    height: 100%;
    background: linear-gradient(135deg, var(--color-chip), var(--color-border));
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-muted-light);
    font-size: 0.7rem;
    font-weight: 600;
    text-align: center;
    padding: 0.5rem;
    overflow: hidden;
  }

  .photo--rect {
    border-radius: 0;
  }

  .photo--circle {
    border-radius: 50%;
  }
</style>
