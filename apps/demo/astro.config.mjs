import { defineConfig } from "astro/config";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  trailingSlash: 'always',
  output: "static",
  vite: {
    plugins: [tailwindcss()],
    build: { target: "esnext" },
    assetsInclude: ["**/*.wasm"],
  },
});
