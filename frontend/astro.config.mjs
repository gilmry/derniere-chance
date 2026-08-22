import { defineConfig } from "astro/config";
import svelte from "@astrojs/svelte";

// Static build: prototype PWA, no backend yet (see ../VISION.md).
export default defineConfig({
  integrations: [svelte()],
  output: "static",
});
