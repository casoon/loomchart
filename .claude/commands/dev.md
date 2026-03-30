# Start Development Environment

Start all services needed for local development.

## Order of startup

1. **Database** — ensure PostgreSQL is running locally or that `DATABASE_URL` in `apps/capital-feed/.env` points to a reachable instance

2. **Phoenix backend** (terminal 1):
   ```bash
   cd apps/phoenix
   mix phx.server
   ```
   Runs on `http://localhost:4000`. WebSocket at `ws://localhost:4000/socket`.

3. **Frontend** (terminal 2):
   ```bash
   cd apps/frontend
   pnpm dev
   ```
   Runs on `http://localhost:4321`.

4. **Capital.com data feed** (optional, terminal 3 — requires real API credentials):
   ```bash
   cd apps/capital-feed
   cargo run
   ```

## WASM rebuild

After any change to `crates/chartcore/` or `packages/wasm-core/`:
```bash
cd packages/wasm-core
wasm-pack build --target web --out-dir ../../apps/frontend/src/wasm
```

The frontend dev server picks up the new WASM files automatically on the next page reload.
