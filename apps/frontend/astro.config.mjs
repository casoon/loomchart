import { defineConfig } from "astro/config";
import tailwind from "@astrojs/tailwind";

export default defineConfig({
  output: "static",
  integrations: [tailwind()],
  server: {
    port: 4321,
    host: true,
  },
  vite: {
    optimizeDeps: {
      exclude: ["@loom/wasm-core"],
    },
    build: {
      target: "esnext",
    },
    assetsInclude: ["**/*.wasm"],
  },
});
