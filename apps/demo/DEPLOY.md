# Deploy to Cloudflare Pages

The demo is a fully static Astro app — no server required.

## Cloudflare Pages Setup

1. Connect your `casoon/loomchart` GitHub repo to Cloudflare Pages
2. Use these build settings:

   | Setting | Value |
   |---------|-------|
   | Framework preset | Astro |
   | Build command | `cd apps/demo && npm install && npm run build` |
   | Build output directory | `apps/demo/dist` |
   | Root directory | *(leave blank — repo root)* |

3. Deploy — the WASM binaries are pre-built and committed in `public/wasm/`,
   so no Rust toolchain is needed on the build machine.

## Local Preview

```bash
cd apps/demo
pnpm install
pnpm dev        # http://localhost:4322
pnpm build      # produces dist/
pnpm preview    # serve dist/ locally
```

## Updating the WASM

When the chart engine is rebuilt, copy the fresh binaries:

```bash
cp -r apps/frontend/public/wasm/. apps/demo/public/wasm/
```

Then commit and push — Cloudflare Pages will redeploy automatically.
