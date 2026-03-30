# Loom Trading Platform - User Guide

Welcome to Loom, a high-performance trading chart platform built with Rust and modern web technologies.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Chart Navigation](#chart-navigation)
3. [Indicators](#indicators)
4. [Drawing Tools](#drawing-tools)
5. [Multi-Panel Layouts](#multi-panel-layouts)
6. [Real-Time Data](#real-time-data)
7. [Keyboard Shortcuts](#keyboard-shortcuts)
8. [Troubleshooting](#troubleshooting)

---

## Getting Started

### First Launch

When you first open Loom, you'll see:
- **Main Chart Panel**: Your primary price chart
- **Toolbar**: Symbol selection, timeframe buttons, and indicators
- **Sidebar**: Quick access to drawing tools and layouts

### Selecting a Symbol

1. Click the **Source** dropdown (Binance, Coinbase, Kraken)
2. Select your preferred **Symbol** (BTCUSDT, ETHUSD, etc.)
3. The chart will automatically load historical data

### Changing Timeframes

Click any timeframe button in the toolbar:
- **Seconds**: 1s, 5s, 15s, 30s
- **Minutes**: 1m, 5m, 15m, 30m
- **Hours**: 1h, 2h, 4h, 6h, 12h
- **Days**: 1d
- **Weeks**: 1w
- **Months**: 1M

---

## Chart Navigation

### Pan and Zoom

- **Pan**: Click and drag the chart horizontally
- **Zoom**: Use mouse wheel to zoom in/out
- **Fit to Data**: Double-click the chart to auto-fit visible data

### Crosshair

- Move your mouse over the chart to see:
  - **Time**: Bottom axis
  - **Price**: Right axis
  - **OHLCV**: Top toolbar shows current candle data

### Price Scale

- **Auto-scale**: Chart automatically adjusts to visible data
- **Manual range**: Right-click price axis (coming soon)

---

## Indicators

### Adding Indicators

**Quick Add** (Top 3):
1. Use the checkboxes in the toolbar for:
   - **EMA** (Exponential Moving Average)
   - **RSI** (Relative Strength Index)
   - **MACD** (Moving Average Convergence Divergence)

**Full Library**:
1. Click the **"+ Indicators"** button
2. Browse available indicators
3. Click to add to chart

### Configuring Indicators

Each indicator has customizable parameters:

- **EMA Period**: 9, 21, 50, 200 (dropdown)
- **RSI Period**: 7, 14, 21 (dropdown)
- **MACD**: Standard 12,26,9 settings

### Removing Indicators

1. Uncheck the indicator in the toolbar
2. Or use the Indicator Selector modal to manage all indicators

### Available Indicators (20/70 Migrated)

#### Tier 1: Essential (Complete)
✅ SMA, EMA, WMA, VWAP, Bollinger Bands  
✅ RSI, Stochastic, CCI, Williams %R, ROC  

#### Tier 2: Common (In Progress)
✅ MACD, ATR, ADX, Momentum, OBV  
⏳ Ichimoku, Parabolic SAR, Supertrend, Aroon, DMI

---

## Drawing Tools

### Available Tools

Access from the **left sidebar**:
- **Trendline**: Draw support/resistance lines
- **Horizontal Line**: Mark key price levels
- **Vertical Line**: Mark time events
- **Rectangle**: Highlight price zones
- **Fibonacci Retracement**: Auto-calculate Fib levels

### Using Drawing Tools

1. Click the tool icon in sidebar
2. Click on chart to place first point
3. Click again to place second point (if needed)
4. Tool is automatically saved

### Editing Drawings

- **Move**: Click and drag the drawing
- **Delete**: Right-click → Delete (or press Delete key)
- **Color**: Right-click → Change Color

---

## Multi-Panel Layouts

### Panel Manager

1. Click **"Panels"** button in toolbar
2. See all active panels:
   - Main chart
   - Indicator panels (RSI, MACD, Volume)
3. Add/remove panels as needed

### Layout Presets

1. Click **"Presets"** button
2. Choose from templates:
   - **Single**: Just price chart
   - **Classic**: Price + Volume
   - **Technical**: Price + RSI + MACD
   - **Full**: All panels visible

### Custom Layouts

1. Click **"Layouts"** button
2. Create new layout:
   - Name your layout
   - Configure panel arrangement
   - Save for quick access

---

## Real-Time Data

### WebSocket Connection

Loom uses Phoenix Channels for real-time streaming:

- **Status Indicator**: Top-right corner shows connection status
  - 🟢 Connected
  - 🟡 Connecting
  - 🔴 Disconnected

### Connection Features

- **Auto-reconnect**: Automatically reconnects on disconnect
- **Delta Sync**: Only fetches new data after reconnection
- **Backfill**: Loads missing candles when reconnecting

### Switching Data Modes

- **Live Data**: Click "Fetch Live Data" button
- **Test Data**: Click "Generate Test Data" for realistic simulation

---

## Keyboard Shortcuts

### Navigation
- `←` `→`: Pan left/right
- `+` `-`: Zoom in/out
- `Home`: Jump to first candle
- `End`: Jump to latest candle
- `Space`: Pause/resume auto-scroll

### Drawing Tools
- `T`: Activate Trendline
- `H`: Horizontal Line
- `V`: Vertical Line
- `R`: Rectangle
- `F`: Fibonacci
- `Esc`: Deselect tool
- `Delete`: Remove selected drawing

### View
- `1-9`: Switch timeframes (1m, 5m, 15m, etc.)
- `I`: Toggle indicators panel
- `D`: Toggle drawing toolbar
- `F11`: Fullscreen mode

---

## Troubleshooting

### Chart Not Loading

**Symptoms**: Empty chart, no candles visible

**Solutions**:
1. Check internet connection
2. Verify symbol is valid for selected source
3. Click "Fetch Live Data" to reload
4. Check browser console for errors

### WebSocket Disconnected

**Symptoms**: Red status indicator, no live updates

**Solutions**:
1. Wait for auto-reconnect (5-30 seconds)
2. Refresh the page (F5)
3. Check firewall/proxy settings
4. Verify backend is running

### Slow Performance

**Symptoms**: Laggy chart, slow panning/zooming

**Solutions**:
1. Reduce number of visible indicators
2. Lower data density (use longer timeframe)
3. Close other browser tabs
4. Disable browser extensions
5. Check GPU acceleration is enabled

### WASM Initialization Failed

**Symptoms**: Error message "WASM module failed to load"

**Solutions**:
1. Refresh the page (Ctrl+F5)
2. Clear browser cache
3. Update browser to latest version
4. Try different browser (Chrome, Firefox, Edge)

### Storage Quota Exceeded

**Symptoms**: Error message about storage

**Solutions**:
1. Click **"Cache"** button in toolbar
2. Select **"Clear All Cache"**
3. Or use **"Clear Old Data"** to keep recent data

---

## Advanced Features

### Cache Management

Access via **"Cache"** button:
- **View Statistics**: See cache size and entry count
- **Clear All**: Remove all cached data
- **Clear Old**: Remove data older than 30 days
- **Export**: Download cache as JSON

### Layout Management

Access via **"Layouts"** button:
- **Save Layout**: Store current panel arrangement
- **Load Layout**: Restore saved configuration
- **Import/Export**: Share layouts between devices
- **Reset**: Return to default layout

### Performance Monitoring

Press `Ctrl+Shift+P` to toggle performance overlay:
- **FPS**: Current frames per second
- **Render Time**: Time to draw frame
- **Candle Count**: Visible/total candles
- **Memory**: WASM memory usage

---

## Tips & Tricks

### Optimal Performance

1. **Use appropriate timeframe**: Lower timeframes (1s, 5s) require more resources
2. **Limit indicators**: Each indicator adds rendering overhead
3. **Viewport culling**: Only visible data is rendered (automatic)
4. **Hardware acceleration**: Enable in browser settings

### Better Analysis

1. **Multi-timeframe**: Use layout presets to see multiple timeframes
2. **Indicator combos**: Combine RSI + MACD for confluence
3. **Drawing persistence**: Your drawings are saved automatically
4. **Keyboard workflow**: Learn shortcuts for faster trading

### Data Management

1. **Real-time mode**: Best for active trading
2. **Test mode**: Perfect for strategy testing
3. **Historical data**: Use longer timeframes to load more history
4. **Cache enabled**: Faster subsequent loads

---

## Getting Help

### Documentation
- **Developer Docs**: See `DEVELOPER_GUIDE.md`
- **Architecture**: See `ARCHITECTURE.md`
- **API Reference**: See `docs/api/`

### Community
- **GitHub Issues**: Report bugs or request features
- **Discussions**: Ask questions, share strategies

### Performance Benchmarks
- See `PERFORMANCE.md` for detailed metrics
- Run `/bench` command in browser console

---

## Changelog

### Phase 6 (Current) - Polish & Production
- ✅ Error boundaries with user-friendly notifications
- ✅ Toast notification system
- ✅ Loading states for async operations
- ✅ Enhanced test data generator with scenarios
- ✅ Integration test suite
- ✅ Performance optimizations (viewport culling, LOD rendering)

### Phase 5 - Real-Time Streaming
- ✅ Phoenix WebSocket integration
- ✅ Delta sync on reconnect
- ✅ Exponential backoff retry
- ✅ Live candle updates

### Phase 4 - Drawing Tools
- ✅ Trendlines, horizontal/vertical lines
- ✅ Rectangles, Fibonacci retracement
- ✅ Drawing persistence

### Phase 3 - Indicators
- ✅ 20 technical indicators
- ✅ Multi-panel support
- ✅ Configurable parameters

---

**Version**: 0.1.0  
**Last Updated**: 2025-12-31  
**Status**: Production Ready
