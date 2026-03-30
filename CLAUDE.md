# LoomChart — Claude Code Guide

## Project Overview

LoomChart is a Rust/WASM + Astro monorepo for a real-time trading chart platform. The chart engine is written in Rust and compiled to WebAssembly; the frontend is Astro + Alpine.js; the backend is Elixir/Phoenix.

## Monorepo Layout

```
crates/chartcore/       Core Rust chart engine → compiled to WASM
packages/wasm-core/     WASM bindings layer (Rust, wasm-bindgen)
packages/indicators/    Pure Rust TA library (no_std)
packages/core/          Shared types: Candle, Timeframe, Symbol
packages/backtest/      Backtesting engine
packages/signals/       Signal DSL
packages/risk/          Risk management
packages/data/          Data providers
apps/frontend/          Astro web app
apps/phoenix/           Elixir Phoenix backend
apps/capital-feed/      Rust Capital.com data feed service
```

## Build Commands

### WASM (must rebuild after every Rust change)
```bash
cd packages/wasm-core
wasm-pack build --target web --out-dir ../../apps/frontend/src/wasm
```

### Frontend
```bash
cd apps/frontend && pnpm dev       # dev server (port 4321)
cd apps/frontend && pnpm build     # production build
cd apps/frontend && pnpm check     # TypeScript + Astro type check
```

### Rust
```bash
cargo test --workspace             # all tests
cargo build --workspace            # build all crates
cargo clippy --workspace           # lint
```

### Backend (Phoenix)
```bash
cd apps/phoenix
mix phx.server                     # dev server (port 4000)
mix test                           # run tests
```

## Key Architectural Decisions

- **Rust → WASM**: The chart engine (`crates/chartcore`) never calls browser APIs directly. It produces `RenderCommand` enums that the TypeScript layer executes on Canvas. This keeps the engine testable in pure Rust.
- **Indicator pipeline**: Indicators are registered in `crates/chartcore/src/indicators/registry.rs` and use incremental state — they do NOT recalculate from scratch on each tick.
- **WASM state**: All chart state lives in `packages/wasm-core/src/state.rs` (`AppState`). JavaScript interacts exclusively through the exported WASM functions.
- **Phoenix Channels**: The frontend subscribes to `candle:*` topics over WebSocket. The Elixir backend broadcasts new candles received from the data feed.

## Important Files

| File | Purpose |
|------|---------|
| `crates/chartcore/src/indicators/registry.rs` | Indicator registration & lookup |
| `crates/chartcore/src/indicators/metadata.rs` | Indicator metadata (name, params, groups) |
| `crates/chartcore/src/core/chart_renderer.rs` | Main render loop |
| `packages/wasm-core/src/state.rs` | WASM-exposed AppState |
| `apps/frontend/src/wasm/trading_ui.js` | Generated WASM JS bindings (do not edit manually) |
| `apps/frontend/src/components/IndicatorSelector.astro` | Indicator picker UI |
| `apps/phoenix/lib/loom_web/channels/` | Phoenix WebSocket channels |

## Known Open TODOs

These are documented gaps — do not silently stub them:

- `packages/plugins/loader.rs`: wasmtime integration for third-party indicator plugins
- `apps/phoenix/lib/loom_web/channels/user_socket.ex`: JWT verification for Supabase auth
- `packages/wasm-core/src/websocket/mod.rs`: WebSocket reconnect with exponential backoff
- `crates/chartcore/src/rendering/renderer.rs`: Dashed/dotted line styles
- `crates/chartcore/src/core/indicator_renderer.rs`: DrawCircle, polygon fill, dash patterns

## Credentials & Secrets

- Never commit `.env` files
- `apps/capital-feed/.env.example` contains only placeholder values — keep it that way
- Database URLs, API keys, and passwords must only ever live in `.env` files

## Commit Style

- Imperative subject line: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`
- No `Co-Authored-By` lines
- Do not commit, push, or publish without explicit instruction
