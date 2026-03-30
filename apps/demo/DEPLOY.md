# Deploy to Cloudflare Workers

The demo is deployed as a Cloudflare Worker with static asset binding.
Deployment is manual via `wrangler`.

## First-time setup

```bash
cd apps/demo
pnpm install

# Log in with joern.seidel@casoon.de
npx wrangler login
```

## Deploy

```bash
cd apps/demo
pnpm cf:deploy    # runs: astro build && wrangler deploy
```

The Worker is named `loomchart-demo` (see `wrangler.toml`).
After deploy, it is live at `https://loomchart-demo.<your-subdomain>.workers.dev`.

To use a custom domain, add a route in the Cloudflare dashboard:
**Workers & Pages → loomchart-demo → Settings → Domains & Routes**

## Local preview

```bash
cd apps/demo
pnpm dev          # Astro dev server — http://localhost:4322
pnpm build        # produces dist/
pnpm preview      # serve dist/ locally via Astro
```

## Updating the WASM

When the chart engine is rebuilt, copy the fresh binaries and redeploy:

```bash
cp -r apps/frontend/public/wasm/. apps/demo/public/wasm/
cd apps/demo && pnpm deploy
```
