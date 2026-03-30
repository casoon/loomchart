# Frontend Integration - CandleGenerator

Die CandleGenerator Testfunktion ist jetzt in `apps/frontend` integriert!

## Build & Test

### 1. WASM Module bauen

```bash
./build-wasm.sh
```

Dies baut das WASM Modul nach `apps/frontend/public/wasm/`.

### 2. Frontend starten

```bash
cd apps/frontend
pnpm dev
```

### 3. Test Data Generator verwenden

Im Frontend siehst du jetzt einen neuen Bereich im Control Panel:

**Test Data Generator**
- Market: crypto, stock, forex, futures, commodities
- Trend: bullish_strong, bullish_mild, sideways, bearish_mild, bearish_strong
- Volatility: low, normal, high, extreme
- Candles: 10-2000

Klick auf "Generate & Load" um realistische Testdaten zu generieren!

## Was wurde geändert

### 1. ControlPanel.astro
Neuer Bereich mit Formularelementen für:
- Market Type Selection
- Trend Selection
- Volatility Selection
- Candle Count Input
- Generate & Load Button

### 2. app.ts
Erweiterungen:
- `TestDataConfig` Interface
- `testData` State mit Defaults
- `loadTestData()` Funktion die:
  - WASM Modul lädt
  - WASM initialisiert
  - `load_test_data()` aufruft
  - Last candle aktualisiert

### 3. build-wasm.sh
Output-Pfad korrigiert zu `apps/frontend/public/wasm/`

## Funktionsweise

```
User klickt "Generate & Load"
    ↓
Alpine.js: loadTestData()
    ↓
Import WASM: /wasm/trading_ui.js
    ↓
WASM: load_test_data(market, trend, volatility, count)
    ↓
chartcore: CandleGenerator generiert realistische Kerzen
    ↓
AppState: Kerzen werden geladen
    ↓
bridge: set_candles() sendet an Chart
    ↓
Chart zeigt generierte Daten
```

## Features der generierten Daten

- **Realistic Price Movements**: Geometric Brownian Motion
- **Market Hours**: 
  - Stock: 9:30-16:00 EST mit Wochenend-Gaps
  - Crypto: 24/7 ohne Gaps
  - Forex: 24/5 mit Wochenend-Gaps
  - Futures: Fast 24/5
- **Volatility Regimes**: Low (0.3%), Normal (0.8%), High (2.0%), Extreme (5.0%)
- **Trends**: Von -0.2% bis +0.2% drift pro Kerze
- **Realistic Volume**: Korreliert mit Volatilität
- **Wicks**: Basierend auf Liquidität
- **Reproducible**: Seed=42 für konsistente Ergebnisse

## Verwendungsbeispiele

### Bullish Crypto Market
```
Market: crypto
Trend: bullish_strong
Volatility: high
Candles: 1000
```

### Stock Market Crash
```
Market: stock
Trend: bearish_strong
Volatility: extreme
Candles: 500
```

### Sideways Forex
```
Market: forex
Trend: sideways
Volatility: low
Candles: 2000
```

## Troubleshooting

### WASM Module nicht gefunden
```bash
# Prüfe ob WASM gebaut wurde
ls -la apps/frontend/public/wasm/

# Sollte zeigen:
# trading_ui.js
# trading_ui_bg.wasm
# trading_ui.d.ts
```

### Build Fehler
```bash
# Rust & wasm-pack updaten
rustup update
cargo install wasm-pack

# Clean build
cd packages/wasm-core
cargo clean
cd ../..
./build-wasm.sh
```

### Browser Console Errors
Öffne Browser DevTools (F12) und prüfe:
- Network Tab: Wird `trading_ui.js` geladen?
- Console: Werden WASM Fehler angezeigt?
- CORS: Frontend muss auf http://localhost laufen, nicht file://

## Nächste Schritte

Nach erfolgreichem Test kannst du:

1. **Preset Scenarios** hinzufügen (z.B. "Crash", "Rally", "Consolidation")
2. **Switch zwischen Live & Test Data** implementieren
3. **Export Funktion** für generierte Datasets
4. **Backtesting Integration** mit Indicators
5. **Bulk Generation** für Performance-Tests

Viel Spaß beim Testen! 🚀
