# Architecture Documentation

## Overview

Loom is a real-time trading UI designed for displaying and analyzing candle-based market data. The architecture follows a modular approach with clear separation of concerns.

## Components

### 1. Phoenix Backend (Fly.io)

**Location:** `apps/phoenix/`

**Responsibilities:**
- REST API for historical data
- WebSocket/Channels for real-time streaming
- Data persistence (via Supabase Postgres)
- Optional: Server-side indicator computation

**Key Modules:**
- `Loom.MarketData` - Candle queries, upserts, broadcasting
- `Loom.Indicators` - Indicator definitions and instances
- `LoomWeb.CandlesChannel` - Real-time candle streaming

**Channel Events:**
```elixir
# Topic: "candles:{source}:{symbol}:{tf}"

# On join
push(socket, "candle_snapshot", %{candles: [...], server_time: "..."})

# On candle update (running candle)
push(socket, "candle_update", %{...candle_data, is_final: false})

# On candle close
push(socket, "candle_final", %{...candle_data, is_final: true})
```

### 2. Supabase PostgreSQL

**Location:** `supabase/migrations/`

**Tables:**
- `candles` - OHLCV data with composite primary key
- `indicator_definitions` - Registry of available indicators
- `indicator_instances` - Activated indicators per chart
- `indicator_values` - Computed time series
- `instruments` - Symbol metadata
- `signals` - Trading signals (future)

**Best Practices:**
- Use `ts` rounded to timeframe bucket start
- `is_final` flag distinguishes running vs closed candles
- Efficient indexes for range queries

### 3. Frontend (Astro + Alpine)

**Location:** `apps/frontend/`

**Responsibilities:**
- UI layout and controls
- Load and initialize WASM module
- Alpine.js for reactive UI state

**Key Files:**
- `src/pages/index.astro` - Main page
- `src/lib/app.ts` - Alpine component
- `src/lib/wasm-bridge.ts` - WASM loader

### 4. Chart Wrapper

**Location:** `packages/chart-wrapper/`

**Responsibilities:**
- Thin wrapper around `lightweight-charts`
- Imperative API for WASM control
- No business logic

**API:**
```typescript
class LoomChart {
  create(config: ChartConfig): void
  addCandlestickSeries(id: string, options?): void
  addLineSeries(id: string, options?): void
  setCandles(seriesId: string, candles: ChartCandle[]): void
  updateCandle(seriesId: string, candle: ChartCandle): void
  setLineSeries(seriesId: string, points: ChartLinePoint[]): void
  updateLinePoint(seriesId: string, point: ChartLinePoint): void
  addMarker(seriesId: string, marker: ChartMarker): void
}
```

### 5. Rust WASM Core

**Location:** `packages/wasm-core/`

**Responsibilities:**
- UI state management
- WebSocket client to Phoenix
- REST client for history
- Client-side indicator calculation
- Bridge to JS chart wrapper

**Exports:**
```rust
#[wasm_bindgen]
pub fn init(config_json: &str) -> Result<(), JsValue>
pub fn connect(source: &str, symbol: &str, tf: &str, ws_url: &str) -> Result<(), JsValue>
pub fn disconnect() -> Result<(), JsValue>
pub fn set_symbol(symbol: &str) -> Result<(), JsValue>
pub fn set_timeframe(tf: &str) -> Result<(), JsValue>
pub fn toggle_indicator(name: &str, params_json: &str, enabled: bool) -> Result<(), JsValue>
```

## Data Flow

### Connect & Load History

```
User selects EURUSD/1m
    │
    ▼
WASM.connect("capitalcom", "EURUSD", "1m", wsUrl)
    │
    ├──► REST: GET /api/candles?source=capitalcom&symbol=EURUSD&tf=1m&limit=500
    │       │
    │       ▼
    │    Chart.setCandles("main", candles)
    │
    └──► WS: Join "candles:capitalcom:EURUSD:1m"
            │
            ▼
         On: "candle_snapshot" (if needed)
```

### Live Updates

```
Phoenix receives new tick
    │
    ▼
MarketData.upsert_candle() → broadcasts to PubSub
    │
    ▼
CandlesChannel pushes "candle_update" / "candle_final"
    │
    ▼
WASM receives message
    │
    ├──► Update state.candles
    ├──► Indicator.on_candle() → new values
    └──► Chart.updateCandle() / Chart.updateLinePoint()
```

### Reconnect Strategy

```
WS disconnects
    │
    ▼
Set status = "reconnecting"
    │
    ▼
Exponential backoff: 1s, 2s, 4s, 8s... (max 30s)
    │
    ▼
On reconnect:
    ├──► REST: Fetch candles since last known timestamp
    ├──► Merge with existing data
    └──► WS: Rejoin channel
```

## Indicator System

### Two Modes

1. **Client-computed (WASM)** - MVP approach
   - Faster, no server load
   - Calculated from candle stream in real-time
   - Not persisted

2. **Server-computed (Phoenix)** - Future
   - Source of truth for backtesting
   - Stored in `indicator_values`
   - Streamed via channel

### Indicator Plugin Pattern

```rust
pub trait Indicator {
    fn on_candle(&mut self, candle: &Candle) -> Option<f64>;
    fn reset(&mut self);
    fn recalculate(&mut self, candles: &VecDeque<Candle>) -> Vec<(i64, f64)>;
    fn id(&self) -> &str;
}
```

### Best Practices

- Use rolling window (ring buffer) for efficiency
- Only "commit" values on `is_final=true`
- Preview values for running candle
- Numerical stability: clear rounding policy

## Security Considerations

- JWT verification for Supabase auth (planned)
- Row Level Security (RLS) on database tables
- CORS configuration
- Environment-based secrets
