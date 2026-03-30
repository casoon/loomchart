# Loom Trading Platform - Getting Started

## 🎯 Projekt-Übersicht

Loom ist eine modulare Trading-Plattform mit folgender Architektur:

```
┌─────────────────────────────────────────────────────────────────┐
│                        LOOM ECOSYSTEM                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  loom-core   │  │loom-indicators│  │   loom-signals       │  │
│  │              │  │               │  │                      │  │
│  │ • Timeframes │  │ • 47 Indikat. │  │ • Wyckoff           │  │
│  │ • Candles    │  │ • EMA, RSI... │  │ • Smart Money (SMC) │  │
│  │ • Events     │  │ • Patterns    │  │ • Elliott Waves     │  │
│  │ • Symbols    │  │ • Streaming   │  │ • Signal DSL        │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │  loom-risk   │  │  loom-chart  │  │   loom-backtest      │  │
│  │              │  │              │  │                      │  │
│  │ • Sizing     │  │ • Primitives │  │ • Strategy Trait     │  │
│  │ • Kill Switch│  │ • Plugins    │  │ • Fill Models        │  │
│  │ • Limits     │  │ • WASM Bridge│  │ • Metrics & Reports  │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                     loom-data                             │   │
│  │    Data Providers (CSV, Binance, Mock, Time Ranges)      │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                     @loom/chart-core                       │  │
│  │  JavaScript Chart Library (WebGPU/Canvas, Touch, Export)  │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## 🚀 Schnellstart

### 1. Repository klonen und Dependencies installieren

```bash
git clone <repo>
cd loom

# Node.js Dependencies
pnpm install

# Rust Dependencies werden automatisch geladen
```

### 2. Rust Crates bauen

```bash
# Alle Crates bauen
cd packages/indicators && cargo build --release
cd ../core && cargo build --release
cd ../signals && cargo build --release
cd ../risk && cargo build --release
cd ../chart && cargo build --release

# Tests ausführen
cargo test --all-features
```

### 3. WASM für Frontend bauen

```bash
cd packages/wasm-core
wasm-pack build --target web --out-dir ../shared/wasm
```

## 📦 Crate-Übersicht

### loom-core
Basis-Typen für alle anderen Crates.

```rust
use loom_core::{Timeframe, Candle, Symbol, Event};

// Timeframe mit Bucket-Alignment
let tf = Timeframe::H1;
let bucket = tf.bucket_start(1700000000); // Aligned to hour

// Candle erstellen
let candle = Candle::new(time, open, high, low, close, volume);

// Symbol mit Metadaten
let btc = Symbol::crypto("BTCUSDT", "BTC", "USDT")
    .with_tick_size(0.01)
    .with_lot_size(0.001);
```

### loom-indicators
47+ technische Indikatoren, `no_std` kompatibel.

```rust
use loom_indicators::prelude::*;

// Streaming-Indikatoren
let mut ema = Ema::new(14);
let mut rsi = Rsi::new(14);
let mut macd = Macd::new(12, 26, 9);

for candle in candles {
    if let Some(value) = ema.next(candle.close) {
        println!("EMA: {}", value);
    }
}

// Stateless Funktionen
let sma = loom_indicators::math::sma(&prices, 20);
let atr = loom_indicators::math::true_range(high, low, prev_close);
```

### loom-signals
Trading-Signale und Pattern-Erkennung.

```rust
use loom_signals::{
    cross_over, cross_under, divergence,
    WyckoffAnalyzer, SmcAnalyzer, ElliottAnalyzer,
};

// Signal DSL
if cross_over(&fast_ma, &slow_ma) {
    println!("Bullish crossover!");
}

// Wyckoff
let wyckoff = WyckoffAnalyzer::new();
if let Some(spring) = wyckoff.detect_spring(&candles) {
    println!("Spring detected: strength {}", spring.strength);
}

// Smart Money Concepts
let smc = SmcAnalyzer::new();
let order_blocks = smc.find_order_blocks(&candles);
let fvgs = smc.find_fvgs(&candles);
```

### loom-risk
Position Sizing und Risk Management.

```rust
use loom_risk::{FixedFractional, KillSwitch, Account};

// Position Sizing (2% Risiko pro Trade)
let sizer = FixedFractional::new(0.02);
let size = sizer.calculate_size(10000.0, 100.0, 95.0); // 40 units

// Kill Switch
let mut kill = KillSwitch::new(KillSwitchConfig::conservative(), 10000.0);
kill.on_trade(-200.0, now); // Trackt Verluste

if !kill.can_trade() {
    println!("Trading gestoppt: {:?}", kill.reason());
}
```

### loom-chart
Chart-Zeichnungen und Plugin-System.

```rust
use loom_chart::{ChartPlugin, DrawingContext, HorizontalLine, PriceZone};

struct MyPlugin;

impl ChartPlugin for MyPlugin {
    fn name(&self) -> &str { "My Plugin" }

    fn on_candle(&mut self, ctx: &mut DrawingContext, candle: &Candle) {
        // Support-Level zeichnen
        ctx.draw(HorizontalLine::new(candle.low)
            .color(Color::GREEN)
            .label("Support"));

        // Zone markieren
        ctx.draw(PriceZone::new(resistance, support)
            .fill(Color::BLUE.with_alpha(0.1)));
    }
}
```

## 🧪 Tests ausführen

```bash
# Alle Rust Tests
cargo test --workspace --all-features

# Mit Output
cargo test --workspace -- --nocapture

# Nur ein Crate
cargo test -p loom-indicators

# Benchmark (falls vorhanden)
cargo bench -p loom-indicators
```

## 📊 Datenquellen

Die Plattform unterstützt verschiedene Datenquellen (siehe `loom-data`):

```rust
use loom_data::{BinanceProvider, CsvProvider, MockProvider};

// Binance Live-Daten
let binance = BinanceProvider::new();
let candles = binance.get_history("BTCUSDT", Timeframe::H1, range).await?;

// CSV Import
let csv = CsvProvider::new("data/btcusdt_1h.csv");

// Mock für Tests
let mock = MockProvider::trending(1000, 0.001);
```

## 🎮 Backtesting

```rust
use loom_backtest::{Backtest, Strategy, FillModel};

struct MyStrategy;

impl Strategy for MyStrategy {
    fn on_candle(&mut self, ctx: &mut StrategyContext, candle: &Candle) {
        // Deine Logik hier
    }
}

let backtest = Backtest::new()
    .data(candles)
    .strategy(MyStrategy)
    .initial_capital(10000.0)
    .fill_model(FillModel::Realistic { slippage: 0.0001 })
    .run();

println!("Sharpe: {}", backtest.metrics().sharpe_ratio);
```

## 📁 Projektstruktur

```
loom/
├── packages/
│   ├── core/           # Basis-Typen (Timeframe, Candle, Event, Symbol)
│   ├── indicators/     # 47+ Indikatoren
│   ├── signals/        # Wyckoff, SMC, Elliott, Signal DSL
│   ├── risk/           # Position Sizing, Kill Switch
│   ├── chart/          # Drawing Primitives, Plugins
│   ├── data/           # Datenquellen (TODO)
│   ├── backtest/       # Backtest Engine (TODO)
│   ├── wasm-core/      # WASM Bridge
│   ├── chart-core/     # JS Chart Library
│   └── shared/         # Shared TS Types
├── apps/
│   ├── frontend/       # Astro + Alpine.js
│   └── phoenix/        # Elixir Backend
├── supabase/           # DB Migrations
├── tests/              # Integration Tests (TODO)
│   ├── golden/         # Golden Tests vs. Backtrader/QuantConnect
│   └── integration/    # End-to-End Tests
└── examples/           # Beispiele (TODO)
```

## ⚙️ Konfiguration

### Feature Flags (Cargo.toml)

```toml
[dependencies]
loom-indicators = { version = "0.1", features = ["full"] }
# oder selektiv:
loom-indicators = { version = "0.1", features = ["trend", "momentum"] }
```

Verfügbare Features:
- `std` - Standard Library (default)
- `serde` - Serialisierung
- `full` - Alle Indikatoren
- `trend` - Trend-Indikatoren
- `momentum` - Momentum-Indikatoren
- `volatility` - Volatilitäts-Indikatoren
- `volume` - Volumen-Indikatoren
- `patterns` - Candlestick Patterns

## 🔗 Nächste Schritte

1. **Datenquellen einrichten** - CSV, Binance, oder Mock-Daten
2. **Indikatoren testen** - Golden Tests gegen bekannte Implementierungen
3. **Strategie entwickeln** - Mit Signal DSL und Indikatoren
4. **Backtesten** - Performance validieren
5. **Live gehen** - Mit Risk Management
