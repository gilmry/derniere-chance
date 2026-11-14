<script lang="ts">
  // Champ d'adresse avec suggestions pendant la frappe (Photon/OpenStreetMap,
  // via le backend). Choisir une suggestion remplit le champ avec une adresse
  // complète, que le géocodeur retrouvera ensuite sans peine.
  import { createEventDispatcher, onDestroy } from "svelte";
  import { suggestAddresses } from "../lib/api";

  export let value = "";
  /** Code ISO du pays : les suggestions s'y limitent. */
  export let pays: string;
  /** « dark » sur l'écran d'inscription, « light » dans le tableau de bord. */
  export let variant: "dark" | "light" = "dark";

  const dispatch = createEventDispatcher<{ input: string }>();
  const DELAI_MS = 300;
  const id = `adresses-${Math.random().toString(36).slice(2)}`;

  let suggestions: string[] = [];
  let open = false;
  let active = -1;
  let timer: ReturnType<typeof setTimeout> | undefined;
  // Numéro de la dernière requête : une réponse plus ancienne, arrivée en
  // retard, ne doit pas écraser les suggestions de la saisie en cours.
  let requete = 0;
  // Une réponse arrivée après que le commerçant a quitté le champ ne doit pas
  // rouvrir la liste par-dessus le champ suivant.
  let focused = false;

  function onInput() {
    dispatch("input", value);
    active = -1;
    clearTimeout(timer);
    if (value.trim().length < 3) {
      suggestions = [];
      open = false;
      return;
    }
    timer = setTimeout(chercher, DELAI_MS);
  }

  async function chercher() {
    const numero = ++requete;
    try {
      const resultat = await suggestAddresses(value, pays);
      if (numero !== requete) return;
      suggestions = resultat;
      open = focused && suggestions.length > 0;
    } catch {
      // Sans suggestions, le champ reste un champ texte ordinaire.
      if (numero === requete) {
        suggestions = [];
        open = false;
      }
    }
  }

  function choisir(suggestion: string) {
    value = suggestion;
    dispatch("input", value);
    suggestions = [];
    open = false;
    active = -1;
  }

  function onKeydown(event: KeyboardEvent) {
    if (!open) return;
    if (event.key === "ArrowDown") {
      event.preventDefault();
      active = (active + 1) % suggestions.length;
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      active = (active - 1 + suggestions.length) % suggestions.length;
    } else if (event.key === "Enter" && active >= 0) {
      event.preventDefault();
      choisir(suggestions[active]);
    } else if (event.key === "Escape") {
      open = false;
    }
  }

  // Changer de pays rend les suggestions affichées caduques.
  $: pays, (suggestions = []), (open = false);

  onDestroy(() => clearTimeout(timer));
</script>

<div class="address {variant}">
  <input
    placeholder="Adresse"
    autocomplete="off"
    role="combobox"
    aria-label="Adresse"
    aria-autocomplete="list"
    aria-expanded={open}
    aria-controls={id}
    aria-activedescendant={active >= 0 ? `${id}-${active}` : undefined}
    bind:value
    on:input={onInput}
    on:keydown={onKeydown}
    on:focus={() => (focused = true)}
    on:blur={() => {
      focused = false;
      open = false;
    }}
    required
  />
  {#if open}
    <ul class="suggestions" id={id} role="listbox">
      {#each suggestions as suggestion, i}
        <!-- mousedown plutôt que click : il passe avant le blur du champ, qui
             fermerait la liste avant que le choix ne soit pris en compte. -->
        <li
          id={`${id}-${i}`}
          role="option"
          aria-selected={i === active}
          class:active={i === active}
          on:mousedown|preventDefault={() => choisir(suggestion)}
        >
          {suggestion}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .address {
    position: relative;
    display: flex;
    flex-direction: column;
  }

  input {
    font-size: 14px;
    font-family: var(--font-body);
  }

  .dark input {
    height: 50px;
    border-radius: 14px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
    padding: 0 16px;
  }

  .dark input::placeholder {
    color: rgba(255, 255, 255, 0.5);
  }

  .light input {
    height: 46px;
    border-radius: 12px;
    border: 1px solid var(--color-border);
    padding: 0 14px;
  }

  .suggestions {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    z-index: 1000;
    margin: 0;
    padding: 4px 0;
    list-style: none;
    background: #fff;
    color: var(--color-ink);
    border-radius: 12px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.18);
    max-height: 240px;
    overflow-y: auto;
  }

  li {
    padding: 10px 14px;
    font-size: 14px;
    line-height: 1.3;
    cursor: pointer;
  }

  li.active,
  li:hover {
    background: rgba(0, 0, 0, 0.06);
  }
</style>
