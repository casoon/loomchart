# LoomChart

A high-performance, real-time trading chart platform built with Rust, WebAssembly, and Astro. LoomChart delivers 60fps candlestick rendering, 70+ technical indicators, and professional drawing tools — all running natively in the browser via WASM.

**[Live Demo](https://loomchart.pages.dev)** — runs entirely in the browser with synthetic data, no backend required.

---

## Features

- **60fps chart rendering** — Rust chart engine compiled to WebAssembly, viewport culling, hardware-accelerated canvas
- **70+ technical indicators** — RSI, MACD, Bollinger Bands, Ichimoku Cloud, ATR, Stochastic, EMA/SMA families, and more
- **Professional drawing tools** — Trend lines, Fibonacci retracements, rectangles, text annotations
- **Multi-panel layouts** — Draggable, resizable panels; indicators on separate sub-charts
- **Real-time data streaming** — WebSocket via Phoenix Channels; Capital.com data feed integration
- **Backtesting engine** — Full strategy validation with portfolio metrics
- **Signal DSL** — Declarative trading rules (crossovers, thresholds, pattern detection)
- **Risk management** — Position sizing, kill switches, drawdown limits
- **Dark / light themes**, keyboard shortcuts, IndexedDB caching

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  Frontend (Astro + Alpine.js + TypeScript)           │
│  apps/frontend/                                      │
├─────────────────────────────────────────────────────┤
│  WASM Chart Engine (Rust → WebAssembly)              │
│  crates/chartcore/  +  packages/wasm-core/           │
├─────────────────────────────────────────────────────┤
│  Backend (Elixir / Phoenix Channels)                 │
│  apps/phoenix/                                       │
├─────────────────────────────────────────────────────┤
│  Data Feed (Rust / Capital.com API)                  │
│  apps/capital-feed/                                  │
├─────────────────────────────────────────────────────┤
│  Database (PostgreSQL via Supabase)                  │
└─────────────────────────────────────────────────────┘
```

---

## Monorepo Structure

```
loomchart/
├── apps/
│   ├── demo/            # Static demo — Cloudflare Pages, synthetic data
│   ├── frontend/        # Astro web app — main UI
│   ├── phoenix/         # Elixir WebSocket & REST backend
│   └── capital-feed/    # Rust data feed (Capital.com → PostgreSQL)
├── crates/
│   └── chartcore/       # Core Rust chart engine (WASM target)
├── packages/
│   ├── wasm-core/       # WASM bindings (Rust → JS)
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
| Frontend | Astro 6, Alpine.js 3, TypeScript 5, Tailwind CSS 4 |
| Chart Engine | Rust 1.75+, wasm-bindgen, web-sys |
| Backend | Elixir, Phoenix Framework, Phoenix Channels |
| Data Feed | Rust, Tokio, tokio-tungstenite, SQLx |
| Database | PostgreSQL 15+ (Supabase compatible) |
| Package Manager | pnpm 9 |

---

## Getting Started

### Prerequisites

- Rust 1.75+ with `wasm-pack` (`cargo install wasm-pack`)
- Node.js 20+ and pnpm 9+ (`npm install -g pnpm`)
- Elixir 1.15+ and Erlang/OTP 26+
- PostgreSQL 15+ (or a Supabase project)

### 1. Install dependencies

```bash
pnpm install
```

### 2. Build the WASM chart engine

```bash
cd packages/wasm-core
wasm-pack build --target web --out-dir ../../apps/frontend/src/wasm
```

### 3. Configure environment

```bash
cp apps/capital-feed/.env.example apps/capital-feed/.env
# Fill in your Capital.com API credentials and database URL
```

### 4. Start the backend

```bash
cd apps/phoenix
mix deps.get
mix ecto.setup
mix phx.server
```

### 5. Start the frontend

```bash
cd apps/frontend
pnpm dev
```

The app will be available at `http://localhost:4321`.

---

## Development

### Run all Rust tests

```bash
cargo test --workspace
```

### Build WASM (dev mode)

```bash
cd packages/wasm-core
wasm-pack build --target web --dev --out-dir ../../apps/frontend/src/wasm
```

### Lint & type-check frontend

```bash
cd apps/frontend
pnpm check
```

---

## Packages

### `crates/chartcore`

The heart of LoomChart. A pure Rust chart rendering engine that:
- Manages multi-panel layouts with configurable split ratios
- Runs incremental indicator calculations (no full-recalc on each tick)
- Issues typed `RenderCommand`s consumed by the canvas renderer
- Compiles to WASM for in-browser use

### `packages/indicators`

Standalone `no_std`-compatible Rust TA library. Each indicator exposes both a stateless math function and a stateful streaming struct. Groups: trend, momentum, volatility, volume, oscillators, Ichimoku family.

### `packages/backtest`

Event-driven backtesting engine. Feed historical `Candle` data, attach a strategy, receive `Trade` and `PortfolioSnapshot` streams. Optional Rayon parallelism for parameter sweeps.

### `packages/signals`

Declarative signal DSL. Express rules like `crossover(fast_ema, slow_ema)` or `above(rsi, 70.0)` and compose them with `And`, `Or`, `Not`.

### `apps/capital-feed`

Rust microservice that connects to the Capital.com streaming WebSocket, backfills historical candles via REST, and writes to PostgreSQL. Deploy via Fly.io (`deploy/capital-feed/`).

---

## Contributing

1. Fork the repo and create a feature branch
2. Run `cargo test --workspace` and `pnpm check` before opening a PR
3. Keep commits focused; one logical change per commit
4. PRs should include a brief description of *why*, not just *what*

---

## License

- Core chart engine and indicators: see `crates/chartcore/LICENSE-COMMERCIAL`
- All other packages: MIT — see `LICENSE`
