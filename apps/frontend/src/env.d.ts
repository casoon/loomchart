/// <reference path="../.astro/types.d.ts" />
/// <reference types="astro/client" />

interface ImportMetaEnv {
  readonly PUBLIC_API_URL: string;
  readonly PUBLIC_WS_URL: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare global {
  interface Window {
    Alpine: import('alpinejs').Alpine;
    loomChart: import('@loom/chart-wrapper').LoomChart;
    loomWasm: typeof import('@loom/wasm-core');
  }
}
