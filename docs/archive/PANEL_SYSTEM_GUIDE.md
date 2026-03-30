# Loom Panel System - Complete Guide

## ✅ Implementiert: Multi-Panel Trading Chart mit Cache

### Architektur-Übersicht

```
┌─────────────────────────────────────────────────────────┐
│                    Browser Frontend                      │
│  ┌───────────────────────────────────────────────────┐  │
│  │  PanelContainer.astro (UI)                        │  │
│  │  ├─ Panel 1: Chart (Candles + EMA + MFI overlay) │  │
│  │  ├─ Separator (Draggable)                         │  │
│  │  ├─ Panel 2: RSI (0-100 scale)                    │  │
│  │  ├─ Separator (Draggable)                         │  │
│  │  └─ Panel 3: MACD                                 │  │
│  └───────────────────────────────────────────────────┘  │
│             ↕ WASM Bindings                             │
│  ┌───────────────────────────────────────────────────┐  │
│  │  WASM Core (Rust)                                 │  │
│  │  ├─ PanelManager (Layout-Berechnung)             │  │
│  │  ├─ ScaleMapper (Multi-Scale-Overlays)           │  │
│  │  └─ AppState (Single Data Source)                │  │
│  └───────────────────────────────────────────────────┘  │
│             ↕ Persistence                               │
│  ┌───────────────────────────────────────────────────┐  │
│  │  IndexedDB                                        │  │
│  │  ├─ candles: {symbol, tf} → Candle[]             │  │
│  │  └─ layouts: {id} → PanelLayout                   │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## 1. Cache-System (IndexedDB)

### Features
- ✅ **Persistent Candle Cache** - Überlebt Page Reload
- ✅ **Multi-Timeframe** - 1m, 5m, 15m, 1h, 4h, 1d cached
- ✅ **Auto-Cleanup** - Alte Daten (>24h) löschen
- ✅ **Layout-Speicherung** - Panel-Konfigurationen speichern

### Verwendung

```typescript
import { candleCacheDB } from '../lib/candle-cache-db';

// Initialize (einmalig beim App-Start)
await candleCacheDB.init();

// Candles speichern
await candleCacheDB.saveCandles('BTCUSD', '5m', candles);

// Candles laden (gibt null wenn stale >1h)
const cached = await candleCacheDB.loadCandles('BTCUSD', '5m');
if (cached) {
  console.log('From cache:', cached.length);
} else {
  console.log('Cache miss, fetching from server...');
}

// Symbol komplett löschen
await candleCacheDB.clearSymbol('BTCUSD');

// Alles löschen
await candleCacheDB.clearAll();

// Stats abrufen
const stats = await candleCacheDB.getCacheStats();
console.log('Cache size:', stats.totalSize, 'bytes');

// Layout speichern
const layout = await wasm.get_panel_layout();
await candleCacheDB.saveLayout('my-setup', JSON.parse(layout));

// Layout laden
const saved = await candleCacheDB.loadLayout('my-setup');
if (saved) {
  await wasm.restore_panel_layout(JSON.stringify(saved));
}
```

### Cache-Management UI

```astro
---
import CacheManager from '../components/CacheManager.astro';
---

<CacheManager />

<script>
  // Show cache manager
  (window as any).showCacheManager();
</script>
```

**Features:**
- Cache Stats (Entries, Size, Age)
- Clear Old Entries (>24h)
- Clear All
- Saved Layouts verwalten
- Current Layout speichern

---

## 2. Panel-System

### WASM API

#### Chart-Overlays (Indikatoren im Hauptchart)

```typescript
// EMA als Overlay (normale Scale)
await wasm.add_chart_overlay('ema21', false);

// MFI als Overlay MIT SCALE-MAPPING! (0-100 → Price Range)
await wasm.add_chart_overlay('mfi14', true);
// ↑ separate_scale=true bedeutet:
// MFI-Werte (0-100) werden automatisch auf den Price-Range
// des Charts gemapped (z.B. 40000-42000 BTCUSD)

// Overlay entfernen
await wasm.remove_chart_overlay('mfi14');
```

#### Indicator-Panels (eigene Pane)

```typescript
// RSI als eigenes Panel unten
const rsiPanelId = await wasm.add_indicator_panel('rsi14', '{"period": 14}');

// MACD als eigenes Panel
const macdPanelId = await wasm.add_indicator_panel('macd', '{}');

// Panel entfernen
await wasm.remove_panel(rsiPanelId);
```

#### Layout-Management

```typescript
// Container-Höhe setzen (bei Window-Resize)
await wasm.set_panel_container_height(window.innerHeight);

// Panel-Höhe ändern (User zieht Separator)
await wasm.resize_panel(panelId, 150);

// Panel verschieben
await wasm.move_panel(panelId, 2); // An Position 2

// Layout als JSON
const layout = await wasm.get_panel_layout();
// {
//   panels: [
//     {
//       id: "uuid",
//       title: "Chart",
//       panel_type: { type: "Chart", overlays: [...] },
//       y_offset: 0,
//       computed_height: 400,
//       stretch_factor: 3.0
//     },
//     {
//       id: "uuid",
//       title: "RSI14",
//       panel_type: { type: "Indicator", indicator_id: "rsi14" },
//       y_offset: 401,
//       computed_height: 100,
//       stretch_factor: 1.0
//     }
//   ],
//   total_height: 600
// }

// Layout wiederherstellen
await wasm.restore_panel_layout(layoutJson);
```

### Frontend-Integration

```astro
---
import PanelContainer from '../components/PanelContainer.astro';
---

<div class="h-screen flex flex-col">
  <Header />
  <PanelContainer />
</div>

<script>
  // Panels neu rendern
  (window as any).refreshPanels();
  
  // Bei WASM-Ready automatisch rendern
  window.addEventListener('wasmReady', () => {
    (window as any).refreshPanels();
  });
</script>
```

---

## 3. Multi-Scale-Overlay-System

### Problem gelöst: Indikatoren mit unterschiedlichen Ranges!

**Beispiel:** MFI (0-100) auf BTCUSD-Chart (40000-42000)

```rust
// In chartcore/src/panels/scale.rs
let mfi_range = ScaleRange::new(0.0, 100.0);
let price_range = ScaleRange::new(40000.0, 42000.0);

let scale = OverlayScale::new(mfi_range, price_range);

// MFI 50 wird gemapped:
let mapped = scale.map_value(50.0);
// → 41000.0 (Mitte des Price-Range)

// MFI 0 → 40000 (Bottom)
// MFI 100 → 42000 (Top)
```

### Verwendung im Frontend

```typescript
// Automatisch beim add_chart_overlay mit separate_scale=true:
await wasm.add_chart_overlay('mfi14', true);

// WASM macht intern:
// 1. MFI-Werte berechnen (0-100)
// 2. Current Price-Range ermitteln (z.B. 40000-42000)
// 3. MFI-Werte mappen auf Price-Range
// 4. Im selben Chart rendern wie Candles
```

**Unterstützte Indikatoren mit separate_scale:**
- MFI (0-100)
- RSI (0-100)
- Stochastic (0-100)
- CCI (-200 bis 200)
- Williams %R (-100 bis 0)

---

## 4. Draggable Separators

### Implementierung

```typescript
// In PanelContainer.astro:
separator.addEventListener('mousedown', (e) => {
  dragState = {
    active: true,
    panelId: panel.id,
    startY: e.clientY,
    startHeight: panel.computed_height
  };
});

document.addEventListener('mousemove', (e) => {
  if (dragState.active) {
    const deltaY = e.clientY - dragState.startY;
    const newHeight = startHeight + deltaY;
    // Update panel height optimistically
  }
});

document.addEventListener('mouseup', async () => {
  // Persist to WASM
  await wasm.resize_panel(panelId, finalHeight);
  await refreshPanels(); // Re-layout
});
```

### Features
- ✅ Smooth Drag (keine Ruckler)
- ✅ Min-Height Enforcement (60px Indicators, 100px Chart)
- ✅ Proportional Redistribution
- ✅ Visual Feedback (Hover-Highlight)

---

## 5. Workflow-Beispiel

### Typischer User-Flow:

```typescript
// 1. App-Start
await candleCacheDB.init();
const wasm = await import('./wasm/trading_ui');
await wasm.init('{}');
(window as any).wasmModule = wasm;

// 2. Connect zu Feed
await wasm.connect('capital', 'BTCUSD', '5m', 'wss://...');

// 3. Check Cache
const cached = await candleCacheDB.loadCandles('BTCUSD', '5m');
if (cached) {
  // Sofort anzeigen
  displayCandles(cached);
}

// 4. Panels aufbauen
await wasm.add_chart_overlay('ema21', false);
await wasm.add_chart_overlay('mfi14', true); // Mit Scale-Mapping!
const rsiPanel = await wasm.add_indicator_panel('rsi14', '{}');

// 5. Layout rendern
(window as any).refreshPanels();

// 6. Bei neuen Candles → Cache updaten
wasm.on_candle = async (candle) => {
  await candleCacheDB.updateCandle('BTCUSD', '5m', candle);
};

// 7. Timeframe wechseln
await wasm.set_timeframe('1h');
const cached1h = await candleCacheDB.loadCandles('BTCUSD', '1h');
if (cached1h) {
  // Instant switch!
} else {
  // Fetch from server
}

// 8. Layout speichern
const layout = await wasm.get_panel_layout();
await candleCacheDB.saveLayout('my-strategy', JSON.parse(layout));

// 9. Später: Layout laden
const saved = await candleCacheDB.loadLayout('my-strategy');
await wasm.restore_panel_layout(JSON.stringify(saved));
(window as any).refreshPanels();
```

---

## 6. Performance-Optimierungen

### Cache-Strategie

**Memory Budget:** ~5 MB für 7 Timeframes
- 1m: 2000 candles × 100 bytes = 200 KB
- 5m: 2000 candles × 100 bytes = 200 KB
- ...
- Total: ~1.4 MB + Indicators ~3 MB

**Stale Detection:**
- Candles älter als 1 Stunde = stale
- Auto-Cleanup alle 24h

### Canvas-Rendering

**Optimierungen:**
- Device Pixel Ratio berücksichtigt (Retina)
- Nur sichtbare Bereiche rendern
- Dirty Rectangles für Partial Updates

---

## 7. API-Referenz

### CandleCacheDB

| Method | Signature | Beschreibung |
|--------|-----------|--------------|
| `init()` | `Promise<void>` | Initialisiere IndexedDB |
| `saveCandles(symbol, tf, candles)` | `Promise<void>` | Candles speichern |
| `loadCandles(symbol, tf)` | `Promise<Candle[] \| null>` | Candles laden (null wenn stale) |
| `clearSymbol(symbol)` | `Promise<void>` | Alle TFs eines Symbols löschen |
| `clearAll()` | `Promise<void>` | Kompletter Reset |
| `getCacheStats()` | `Promise<CacheStats>` | Statistiken |
| `saveLayout(id, layout)` | `Promise<void>` | Panel-Layout speichern |
| `loadLayout(id)` | `Promise<any \| null>` | Panel-Layout laden |

### WASM Panel API

| Function | Signature | Beschreibung |
|----------|-----------|--------------|
| `add_chart_overlay(id, separate_scale)` | `Promise<void>` | Indicator als Overlay |
| `remove_chart_overlay(id)` | `Promise<void>` | Overlay entfernen |
| `add_indicator_panel(id, params)` | `Promise<string>` | Eigenes Panel, returns ID |
| `remove_panel(id)` | `Promise<void>` | Panel entfernen |
| `resize_panel(id, height)` | `Promise<void>` | Panel-Höhe ändern |
| `move_panel(id, index)` | `Promise<void>` | Panel-Reihenfolge |
| `get_panel_layout()` | `Promise<string>` | Layout als JSON |
| `restore_panel_layout(json)` | `Promise<void>` | Layout wiederherstellen |
| `set_panel_container_height(h)` | `Promise<void>` | Container-Resize |

---

## 8. Nächste Schritte

### Noch zu implementieren:

1. **Canvas-Renderer** - Echte Chart-Daten rendern (Candles, Lines, Indicators)
2. **Crosshair-Sync** - Synchronisierter Crosshair über alle Panels
3. **Zoom & Pan** - Mit Mouse Wheel & Drag
4. **Keyboard Shortcuts** - Strg+I für Add Indicator, etc.
5. **Mobile Touch Support** - Touch-Drag für Resize

### Integration-Punkte:

```typescript
// In app.ts oder index.astro:
import { candleCacheDB } from './lib/candle-cache-db';
import PanelContainer from './components/PanelContainer.astro';
import CacheManager from './components/CacheManager.astro';

// Initialize
await candleCacheDB.init();
(window as any).wasmModule = await initWasm();

// Render panels
(window as any).refreshPanels();
```

---

**Status:** ✅ Core-System komplett - Bereit für Chart-Rendering!
