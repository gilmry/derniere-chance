import { defineConfig } from "astro/config";
import svelte from "@astrojs/svelte";

// Static build: pages with a dynamic id (offre/marchand/reservation) read it
// from a query param and fetch client-side (src/lib/api.ts) rather than via
// getStaticPaths, since the catalogue is runtime data, not known at build
// time. See ../VISION.md.
export default defineConfig({
  integrations: [svelte()],
  output: "static",
});
