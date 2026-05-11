# LoomChart

> **Alpha** — actively developed, APIs and features may change without notice.

A high-performance, real-time trading chart platform built with Rust, WebAssembly, and Astro. LoomChart delivers 60fps candlestick rendering, a Rust indicator library, and professional drawing tools — all running natively in the browser via WASM.

**[Live Demo](https://loomchart-demo.casoon.workers.dev)** — runs entirely in the browser with synthetic data, no backend required.

---

## Features

### Chart engine
- **60fps rendering** — Rust chart engine compiled to WebAssembly, viewport culling, pixel-perfect alignment
- **7 chart types** — Candlestick · OHLC · Hollow · Heikin Ashi · Renko · Line · Area
- **6 timeframes** — 1m · 5m · 15m · 1h · 4h · 1d
- **Log / linear price scale** with logarithmically-spaced grid
- **Axis lock** to keep the price range fixed across timeframe switches and fit
- **Dark / light themes** with full token system
- **State export / import** (candles + viewport) for save/load layouts

### Drawing tools
- Trend lines, horizontal lines, vertical lines, clear-all
- Price alerts as draggable horizontal lines
- WASM-side tool manager — tools are first-class chart objects, not canvas decorations

### Analytics (live in demo)
- **Volume Profile** — VPVR (visible range) and VPSV (last 24h session) with POC highlighting
- **Strategy overlay** — SMA(5) / SMA(20) crossover with ▲ buy / ▼ sell markers
- **Divergence detection** — Wilder RSI(14) + pivot detection, bull/bear divergence between consecutive pivots
- **Pattern detection** — Double tops and double bottoms with arc + neckline annotation
- **Multi-timeframe indicator** — SMA(20) from two timeframes higher (e.g. 1h SMA on 5m chart)
- **News overlay** — Synthetic macro events with severity-colored markers

### Playback
- **Bar-by-bar Replay** — Scrubber, play/pause, step, 0.5×–10× speed
- **Live Simulate** — Generates synthetic candles with deterministic seed
- **Static** — Show the full historical dataset

### Alerts
- **Price alerts** (above / below) with browser notifications + visual flash
- **Indicator alerts** when SMA crossovers fire during replay / simulate

### Indicators library
- **70+ technical indicators** — RSI, MACD, Bollinger Bands, Ichimoku Cloud, ATR, Stochastic, EMA/SMA families, and more
- Incremental state — no full recalculation per tick
- Pure Rust, `no_std`-compatible

### Backend (`apps/phoenix`)
- **Phoenix Channels** WebSocket for real-time candle streams
- **JWT auth** (HS256, Supabase-compatible) for production sockets
- Channel test suite

### Data feed (`apps/capital-feed`)
- Rust microservice — Capital.com WebSocket → PostgreSQL
- Designed for Fly.io deployment

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  Frontend (Astro + Alpine.js + TypeScript)          │
│  apps/frontend/  ·  apps/demo/                       │
├─────────────────────────────────────────────────────┤
│  WASM Chart Engine (Rust → WebAssembly)             │
│  crates/chartcore/  +  packages/wasm-core/           │
├─────────────────────────────────────────────────────┤
│  Backend (Elixir / Phoenix Channels, JWT auth)      │
│  apps/phoenix/                                      │
├─────────────────────────────────────────────────────┤
│  Data Feed (Rust / Capital.com API)                 │
│  apps/capital-feed/                                 │
├─────────────────────────────────────────────────────┤
│  Database (PostgreSQL via Supabase)                 │
└─────────────────────────────────────────────────────┘
```

---

## Monorepo Structure

```
loomchart/
├── apps/
│   ├── demo/            # Static demo — Cloudflare Workers, synthetic data
│   ├── frontend/        # Astro web app — main UI
│   ├── phoenix/         # Elixir WebSocket & REST backend
│   └── capital-feed/    # Rust data feed (Capital.com → PostgreSQL)
├── crates/
│   └── chartcore/       # Core Rust chart engine (WASM target)
├── packages/
│   ├── wasm-core/       # WASM bindings (Rust → JS, exports WasmChart)
│   ├── core/            # Shared trading types (Candle, Timeframe, Symbol)
│   ├── indicators/      # Pure Rust TA library (no_std compatible)
│   ├── chart/           # Drawing primitives & plugin system
│   ├── data/            # Data providers (Binance, CSV, async)
│   ├── backtest/        # Backtesting engine
│   ├── risk/            # Risk management
│   ├── signals/         # Signal DSL
│   └── shared/          # TypeScript shared types
├── examples/            # Strategy and indicator examples
├── tests/               # Integration tests
└── deploy/              # Fly.io deployment configs
```

---

## Tech Stack

| Layer | Technology |
|-------|------------|
| `apps/frontend` | Astro 4, Alpine.js 3, TypeScript 5, Tailwind CSS 3 |
| `apps/demo` | Astro 6, Alpine.js 3, TypeScript 5, Tailwind CSS 4 |
| Chart Engine | Rust 1.75+, wasm-bindgen, web-sys, Canvas 2D |
| Backend | Elixir, Phoenix Framework, Phoenix Channels, JWT (HS256) |
| Data Feed | Rust, Tokio, tokio-tungstenite, SQLx |
| Database | PostgreSQL 15+ (Supabase compatible) |
| Hosting (demo) | Cloudflare Workers (wrangler 4) |
| Package Manager | pnpm 9 |

---

## Getting Started

### Prerequisites

- Rust 1.75+ with `wasm-pack` (`cargo install wasm-pack`)
- Node.js 20+ and pnpm 9+ (`npm install -g pnpm`)
- Elixir 1.15+ and Erlang/OTP 26+ (only if running the Phoenix backend)
- PostgreSQL 15+ or a Supabase project (only if running the backend)

### 1. Install dependencies

```bash
pnpm install
```

### 2. Build the WASM chart engine

```bash
cd packages/wasm-core && bash build.sh
```

This outputs to `apps/frontend/public/wasm/` (and you can copy to `apps/demo/public/wasm/` if iterating on the demo).

### 3. Run the demo (no backend required)

```bash
cd apps/demo && pnpm dev
```

Open `http://localhost:4321`. The demo runs purely on synthetic candle data shipped with the bundle.

### 4. Run the full app (with backend)

```bash
# Configure backend
cp apps/capital-feed/.env.example apps/capital-feed/.env
# Fill in API credentials and DATABASE_URL

# Start Phoenix
cd apps/phoenix && mix deps.get && mix ecto.setup && mix phx.server

# In another terminal, start the frontend
pnpm dev
```

---

## Development

### Rust tests

```bash
cargo test --workspace
```

### Build WASM (dev mode)

```bash
cd packages/wasm-core
wasm-pack build --target web --dev --out-dir ../../apps/frontend/public/wasm
```

### Type-check frontend

```bash
pnpm --filter @loom/frontend typecheck
```

### Frontend tests

```bash
pnpm --filter @loom/frontend test
```

### Phoenix tests

```bash
cd apps/phoenix && mix test
```

### Deploy the demo

```bash
cd apps/demo && pnpm build && npx wrangler deploy
```

---

## Packages

### `crates/chartcore`

The heart of LoomChart. A pure Rust chart rendering engine that:
- Manages viewport, candle storage, drawing tools, crosshair, and rendering options
- Supports 5 candle styles (candlestick / OHLC / hollow / line / area), log+linear scale, dark+light themes
- Runs incremental indicator calculations (no full-recalc on each tick)
- Exposes a Canvas 2D renderer that draws directly to an `HtmlCanvasElement`
- Compiles to WASM via `packages/wasm-core` which produces the `WasmChart` JS class

### `packages/indicators`

Standalone `no_std`-compatible Rust TA library. Each indicator exposes both a stateless math function and a stateful streaming struct. Groups: trend, momentum, volatility, volume, oscillators, Ichimoku family.

### `packages/backtest`

Event-driven backtesting engine. Feed historical `Candle` data, attach a strategy, receive `Trade` and `PortfolioSnapshot` streams. Optional Rayon parallelism for parameter sweeps.

### `packages/signals`

Declarative signal DSL. Express rules like `crossover(fast_ema, slow_ema)` or `above(rsi, 70.0)` and compose them with `And`, `Or`, `Not`.

### `apps/capital-feed`

Rust microservice that connects to the Capital.com streaming WebSocket, backfills historical candles via REST, and writes to PostgreSQL. Deploy via Fly.io (`deploy/capital-feed/`).

---

## Roadmap

LoomChart is actively being extended via GitHub Issues and the [LoomChart Roadmap project](https://github.com/users/casoon/projects/6).

### In flight

- **[Rust Plugin Architecture](https://github.com/casoon/loomchart/issues/43)** — Third-party Rust crates compiled to WASM, loaded at runtime via `wasmtime`. Plugin types: indicator, strategy, renderer, data source. Sandboxed by default (no FS / net unless declared in the manifest). Replaces a Pine-Script-style DSL with native Rust + `cargo`. 12 tracked child issues.

### Open feature tracks

- Volume — Footprint candles ([#20](https://github.com/casoon/loomchart/issues/20))
- Symbol management — Multi-symbol overlay ([#33](https://github.com/casoon/loomchart/issues/33))
- UX — Multi-chart grid layout ([#35](https://github.com/casoon/loomchart/issues/35))
- Alerts — Email / push / webhook delivery ([#30](https://github.com/casoon/loomchart/issues/30))

---

## Contributing

1. Fork the repo and create a feature branch
2. Run `cargo test --workspace` and `pnpm --filter @loom/frontend typecheck` before opening a PR
3. Keep commits focused; one logical change per commit
4. PRs should include a brief description of *why*, not just *what*

---

## License

- Core chart engine and indicators: see `crates/chartcore/LICENSE-COMMERCIAL`
- All other packages: MIT — see `LICENSE`
