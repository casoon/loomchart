import { defineConfig } from "astro/config";
import tailwind from "@astrojs/tailwind";

export default defineConfig({
  output: "static",
  integrations: [tailwind()],
  vite: {
    build: { target: "esnext" },
    assetsInclude: ["**/*.wasm"],
  },
});
