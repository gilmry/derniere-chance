<script lang="ts">
  import "leaflet/dist/leaflet.css";
  import { onMount, onDestroy } from "svelte";
  import type { Offer } from "../lib/api";
  import type { Coords } from "../lib/geoloc";

  export let offers: Offer[] = [];
  export let coords: Coords | null = null;

  // Bruxelles par défaut : aucune offre ni position consommateur pour centrer la carte.
  const FALLBACK_CENTER: [number, number] = [50.8503, 4.3517];

  let container: HTMLDivElement;
  let map: import("leaflet").Map | undefined;
  let markersLayer: import("leaflet").LayerGroup | undefined;
  let L: typeof import("leaflet") | undefined;
  let mapError = "";
  let resizeObserver: ResizeObserver | undefined;

  function drawMarkers() {
    if (!map || !L || !markersLayer) return;
    markersLayer.clearLayers();

    const merchants = new Map<string, { lat: number; lon: number; nom: string; offerId: string }>();
    for (const offer of offers) {
      if (
        !merchants.has(offer.marchand_id) &&
        offer.marchand_latitude != null &&
        offer.marchand_longitude != null
      ) {
        merchants.set(offer.marchand_id, {
          lat: offer.marchand_latitude,
          lon: offer.marchand_longitude,
          nom: offer.marchand_nom,
          offerId: offer.id,
        });
      }
    }

    const bounds: [number, number][] = [];

    if (coords) {
      L.circleMarker([coords.lat, coords.lon], {
        radius: 8,
        color: "#fff",
        weight: 2,
        fillColor: "#2e7d5b",
        fillOpacity: 1,
      })
        .addTo(markersLayer)
        .bindPopup("Toi");
      bounds.push([coords.lat, coords.lon]);
    }

    for (const m of merchants.values()) {
      L.marker([m.lat, m.lon])
        .addTo(markersLayer)
        .bindPopup(`<a href="/offre?id=${m.offerId}">${m.nom}</a>`);
      bounds.push([m.lat, m.lon]);
    }

    if (bounds.length > 1) {
      map.fitBounds(bounds, { padding: [32, 32] });
    } else if (bounds.length === 1) {
      map.setView(bounds[0], 14);
    }
  }

  onMount(async () => {
    try {
      L = await import("leaflet");

      // Les icônes par défaut de Leaflet référencent des chemins relatifs au
      // package qui cassent une fois bundlés (problème connu) - on les
      // ré-associe explicitement aux assets importés par Vite. AVANT toute
      // création de marqueur : Leaflet fige les options d'icône au moment où
      // le marqueur est construit, un fix arrivé après (ex. déclenché par
      // markersLayer devenant truthy plus bas) ne rattrape pas les marqueurs
      // déjà créés avec l'icône par défaut cassée.
      const iconRetinaUrl = (await import("leaflet/dist/images/marker-icon-2x.png")).default;
      const iconUrl = (await import("leaflet/dist/images/marker-icon.png")).default;
      const shadowUrl = (await import("leaflet/dist/images/marker-shadow.png")).default;
      L.Icon.Default.mergeOptions({ iconRetinaUrl, iconUrl, shadowUrl });

      map = L.map(container).setView(
        coords ? [coords.lat, coords.lon] : FALLBACK_CENTER,
        14,
      );
      // Tuiles OSM publiques : gratuites, attribution obligatoire (ci-dessous),
      // usage raisonnable seulement (cf. OSM tile usage policy) - à remplacer
      // par un fournisseur dédié (MapTiler, Mapbox...) si le trafic grossit.
      L.tileLayer("https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", {
        attribution: '© <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>',
        maxZoom: 19,
      }).addTo(map);
      markersLayer = L.layerGroup().addTo(map);

      // Le conteneur (flex + min-height) peut continuer à changer de taille
      // après l'init de Leaflet (polices qui finissent de charger, rotation
      // d'écran, apparition de la barre d'adresse mobile...) - sans ce
      // recalcul continu, la carte ne couvre que la taille qu'elle avait au
      // moment de sa création (ex. la moitié de l'écran si le layout n'était
      // pas encore stabilisé).
      resizeObserver = new ResizeObserver(() => map?.invalidateSize());
      resizeObserver.observe(container);
    } catch (err) {
      console.error("OffersMap init failed", err);
      mapError = "La carte n'a pas pu se charger.";
    }
  });

  onDestroy(() => {
    resizeObserver?.disconnect();
    map?.remove();
  });

  // Redessine les marqueurs (sans recréer la carte) quand la liste d'offres
  // ou la position consommateur change (ex. filtre catégorie).
  $: if (map && markersLayer) {
    offers;
    coords;
    drawMarkers();
  }
</script>

{#if mapError}
  <p class="map-error">{mapError}</p>
{/if}
<div bind:this={container} class="leaflet-container-wrap"></div>

<style>
  /* position:absolute + inset:0 plutôt que width/height:100% : le parent
     (.map dans Feed.svelte) est déjà position:relative, donc ceci fixe des
     dimensions en pixels dès le premier layout, sans dépendre de la
     résolution de pourcentages sur un ancêtre flex (source du bug observé :
     la carte ne couvrait qu'une partie de son conteneur). */
  .leaflet-container-wrap {
    position: absolute;
    inset: 0;
  }

  .map-error {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--color-muted);
    font-size: 13px;
    padding: 20px;
    text-align: center;
  }

  :global(.leaflet-container) {
    width: 100%;
    height: 100%;
    font-family: var(--font-body);
  }

  :global(.leaflet-popup-content a) {
    font-weight: 700;
    color: var(--color-primary);
  }
</style>
