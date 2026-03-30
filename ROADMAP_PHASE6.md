# Phase 6: Polish & Production Ready (Weeks 11-12)

**Goal:** Production-quality UX and stability

**Prerequisites:** Phase 5 complete (realtime data streaming)

## Week 11: Performance & Error Handling

### Task 11.1: Performance Profiling

**Objective:** Identify and fix performance bottlenecks.

**Profiling Checklist:**

1. **Chrome DevTools Performance Tab**
   - Record 10 seconds of chart interaction
   - Target: 60fps (16.67ms per frame)
   - Identify long tasks (>50ms)
   - Check for layout thrashing

2. **Rust Profiling with cargo-flamegraph**
   ```bash
   # Install flamegraph
   cargo install flamegraph
   
   # Profile WASM build (may need adaptations for WASM target)
   cd packages/wasm-core
   cargo flamegraph --bin trading_ui
   ```

3. **Memory Profiling**
   - Chrome DevTools Memory tab
   - Take heap snapshots before/after operations
   - Check for memory leaks
   - Target: < 100MB for 1000 candles + 10 indicators

4. **WASM Binary Size**
   - Current size: Check with `ls -lh *.wasm`
   - Target: < 500KB gzipped
   - Optimizations:
     ```toml
     # Cargo.toml
     [profile.release]
     opt-level = "z"  # Optimize for size
     lto = true       # Link-time optimization
     codegen-units = 1
     panic = "abort"
     strip = true     # Remove debug symbols
     ```
   - Use `wasm-opt` from binaryen:
     ```bash
     wasm-opt -Oz -o optimized.wasm input.wasm
     ```

**Performance Targets:**

| Metric | Target | Measurement |
|--------|--------|-------------|
| Frame Rate | 60fps | With 10 indicators active |
| Initial Load | < 500ms | WASM + first render |
| Timeframe Switch | < 200ms | Loading + rendering |
| Indicator Add | < 100ms | Calculation + render |
| Drawing Tool Response | < 16ms | Mouse move to render |
| WASM Size | < 500KB | Gzipped |
| Memory Usage | < 100MB | 1000 candles, 10 indicators |
| WebSocket Latency | < 50ms | Server to chart update |

**Optimization Strategy:**

```rust
// File: crates/chartcore/src/rendering/optimizations.rs

/// Viewport culling - only render visible candles
pub fn cull_candles(candles: &[Candle], viewport: &Viewport) -> &[Candle] {
    let start_idx = viewport.first_visible_index();
    let end_idx = viewport.last_visible_index();
    
    &candles[start_idx..end_idx.min(candles.len())]
}

/// Level-of-detail rendering
pub fn should_render_detail(bar_width: f64) -> bool {
    bar_width > 3.0  // Only show wicks when bars are wide enough
}

/// Batch similar drawing commands
pub fn batch_commands(commands: Vec<RenderCommand>) -> Vec<RenderCommand> {
    // Combine consecutive DrawLine commands with same color/width
    let mut batched = Vec::new();
    let mut current_batch: Vec<RenderCommand> = Vec::new();
    
    for cmd in commands {
        match cmd {
            RenderCommand::DrawIndicatorLine { .. } => {
                if current_batch.is_empty() {
                    current_batch.push(cmd);
                } else {
                    // Check if can be batched with previous
                    // ... batching logic
                }
            }
            _ => {
                if !current_batch.is_empty() {
                    batched.extend(current_batch.drain(..));
                }
                batched.push(cmd);
            }
        }
    }
    
    batched
}
```

**Deliverable:** Performance profiling report and optimizations

---

### Task 11.2: Error Boundaries

**Objective:** Catch and handle all errors gracefully.

**File: `apps/frontend/src/lib/error-boundary.ts`**

```typescript
interface ErrorReport {
    message: string;
    stack?: string;
    timestamp: number;
    context: string;
}

class ErrorBoundary {
    private errorLog: ErrorReport[] = [];
    private maxErrors: number = 100;
    
    init(): void {
        // Catch unhandled errors
        window.addEventListener('error', (event) => {
            this.handleError(event.error, 'window.onerror');
        });
        
        // Catch unhandled promise rejections
        window.addEventListener('unhandledrejection', (event) => {
            this.handleError(event.reason, 'unhandledrejection');
        });
        
        // Catch WASM errors
        this.wrapWasmCalls();
    }
    
    private handleError(error: Error, context: string): void {
        console.error(`[ErrorBoundary] ${context}:`, error);
        
        // Log error
        this.errorLog.push({
            message: error.message || String(error),
            stack: error.stack,
            timestamp: Date.now(),
            context
        });
        
        // Limit log size
        if (this.errorLog.length > this.maxErrors) {
            this.errorLog.shift();
        }
        
        // Show user-friendly notification
        this.showErrorNotification(error, context);
        
        // Report to monitoring service (if configured)
        this.reportToMonitoring(error, context);
    }
    
    private showErrorNotification(error: Error, context: string): void {
        let userMessage = 'An error occurred';
        let action = undefined;
        
        // Customize message based on error type
        if (context === 'WASM') {
            userMessage = 'Chart rendering error';
            action = {
                label: 'Reload',
                handler: () => window.location.reload()
            };
        } else if (context === 'WebSocket') {
            userMessage = 'Connection error - reconnecting...';
        } else if (error.message.includes('quota')) {
            userMessage = 'Storage quota exceeded';
            action = {
                label: 'Clear Cache',
                handler: () => this.clearIndexedDB()
            };
        }
        
        window.dispatchEvent(new CustomEvent('showToast', {
            detail: {
                type: 'error',
                message: userMessage,
                duration: 5000,
                action
            }
        }));
    }
    
    private wrapWasmCalls(): void {
        const originalGetWasm = (window as any).getWasm;
        
        (window as any).getWasm = () => {
            const wasm = originalGetWasm?.();
            if (!wasm) return null;
            
            // Wrap all WASM methods with error handling
            return new Proxy(wasm, {
                get: (target, prop) => {
                    const value = target[prop];
                    
                    if (typeof value === 'function') {
                        return (...args: any[]) => {
                            try {
                                return value.apply(target, args);
                            } catch (err) {
                                this.handleError(err as Error, `WASM.${String(prop)}`);
                                return null;
                            }
                        };
                    }
                    
                    return value;
                }
            });
        };
    }
    
    private async clearIndexedDB(): Promise<void> {
        const databases = await indexedDB.databases();
        for (const db of databases) {
            if (db.name) {
                indexedDB.deleteDatabase(db.name);
            }
        }
        window.location.reload();
    }
    
    private reportToMonitoring(error: Error, context: string): void {
        // Send to Sentry, LogRocket, or custom monitoring
        // For now, just console.log
        console.log('[Monitoring] Error reported:', { error, context });
    }
    
    getErrorLog(): ErrorReport[] {
        return [...this.errorLog];
    }
}

// Initialize on page load
const errorBoundary = new ErrorBoundary();
errorBoundary.init();

export { errorBoundary };
```

**Deliverable:** Global error boundary

---

### Task 11.3: Toast Notification System

**Objective:** User feedback for all async operations.

**File: `apps/frontend/src/components/ToastNotifications.astro`**

```html
<div 
  x-data="{
    toasts: [],
    nextId: 0
  }"
  @show-toast.window="
    const toast = {
      id: nextId++,
      type: $event.detail.type || 'info',
      message: $event.detail.message,
      duration: $event.detail.duration || 5000,
      action: $event.detail.action
    };
    toasts.push(toast);
    
    if (toast.duration > 0) {
      setTimeout(() => {
        toasts = toasts.filter(t => t.id !== toast.id);
      }, toast.duration);
    }
  "
  class="fixed bottom-4 right-4 z-50 flex flex-col gap-2"
>
  <template x-for="toast in toasts" :key="toast.id">
    <div
      x-transition:enter="transition ease-out duration-300"
      x-transition:enter-start="opacity-0 translate-y-2"
      x-transition:enter-end="opacity-100 translate-y-0"
      x-transition:leave="transition ease-in duration-200"
      x-transition:leave-start="opacity-100 translate-y-0"
      x-transition:leave-end="opacity-0 translate-y-2"
      :class="{
        'bg-green-500 text-white': toast.type === 'success',
        'bg-blue-500 text-white': toast.type === 'info',
        'bg-yellow-500 text-yellow-950': toast.type === 'warning',
        'bg-red-500 text-white': toast.type === 'error'
      }"
      class="px-4 py-3 rounded-lg shadow-lg flex items-center justify-between gap-4 min-w-[300px]"
    >
      <!-- Icon -->
      <div class="flex-shrink-0">
        <template x-if="toast.type === 'success'">
          <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"/>
          </svg>
        </template>
        <template x-if="toast.type === 'error'">
          <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
          </svg>
        </template>
        <template x-if="toast.type === 'info'">
          <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z" clip-rule="evenodd"/>
          </svg>
        </template>
      </div>
      
      <!-- Message -->
      <div class="flex-1" x-text="toast.message"></div>
      
      <!-- Action Button -->
      <button
        x-show="toast.action"
        @click="toast.action?.handler(); toasts = toasts.filter(t => t.id !== toast.id)"
        class="px-3 py-1 bg-white/20 rounded hover:bg-white/30 text-sm font-medium"
        x-text="toast.action?.label"
      ></button>
      
      <!-- Close Button -->
      <button
        @click="toasts = toasts.filter(t => t.id !== toast.id)"
        class="flex-shrink-0 hover:opacity-80"
      >
        <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd"/>
        </svg>
      </button>
    </div>
  </template>
</div>
```

**Usage examples:**

```typescript
// Success
window.dispatchEvent(new CustomEvent('showToast', {
    detail: {
        type: 'success',
        message: 'Indicator added successfully'
    }
}));

// Error with action
window.dispatchEvent(new CustomEvent('showToast', {
    detail: {
        type: 'error',
        message: 'Failed to load candles',
        action: {
            label: 'Retry',
            handler: () => retryLoadCandles()
        }
    }
}));
```

**Deliverable:** Toast notification system

---

### Task 11.4: Loading States

**Objective:** Show loading indicators for all async operations.

**File: `apps/frontend/src/components/LoadingOverlay.astro`**

```html
<div
  x-data="{ loading: false, message: '' }"
  @show-loading.window="loading = true; message = $event.detail.message"
  @hide-loading.window="loading = false"
  x-show="loading"
  x-transition
  class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center"
>
  <div class="bg-card border border-border rounded-lg p-6 flex flex-col items-center gap-4">
    <!-- Spinner -->
    <svg class="animate-spin h-12 w-12 text-primary" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
      <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
      <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
    </svg>
    
    <!-- Message -->
    <p class="text-lg font-medium" x-text="message"></p>
  </div>
</div>
```

**Skeleton placeholders for charts:**

```html
<!-- File: apps/frontend/src/components/ChartSkeleton.astro -->
<div class="animate-pulse bg-card border border-border rounded-lg p-4">
  <div class="h-8 bg-muted rounded w-1/4 mb-4"></div>
  <div class="h-64 bg-muted rounded"></div>
</div>
```

**Usage:**

```typescript
// Show loading
window.dispatchEvent(new CustomEvent('showLoading', {
    detail: { message: 'Loading candles...' }
}));

// Hide loading
window.dispatchEvent(new CustomEvent('hideLoading'));
```

**Deliverable:** Loading states for all async operations

---

## Week 12: Testing & Documentation

### Task 12.1: Integration Tests

**Objective:** Comprehensive test coverage for critical paths.

**File: `tests/integration/chart_rendering_test.rs`**

```rust
#[cfg(test)]
mod tests {
    use chartcore::*;
    
    #[test]
    fn test_chart_renders_candles() {
        let mut chart = ChartCore::new(800, 600);
        let candles = generate_test_candles(100);
        chart.load_candles(candles);
        
        let buffer = chart.render();
        
        // Verify commands were generated
        assert!(buffer.commands.len() > 0, "No render commands generated");
        
        // Count candle commands
        let candle_count = buffer.commands.iter()
            .filter(|cmd| matches!(cmd, RenderCommand::DrawCandle { .. }))
            .count();
        
        assert_eq!(candle_count, 100, "Should render 100 candles");
    }
    
    #[test]
    fn test_indicator_overlay() {
        let mut chart = ChartCore::new(800, 600);
        chart.load_candles(generate_test_candles(100));
        
        // Add SMA indicator
        chart.add_indicator("sma", json!({"period": 20}), true).unwrap();
        
        let buffer = chart.render();
        
        // Verify SMA line is rendered
        let indicator_lines = buffer.commands.iter()
            .filter(|cmd| matches!(cmd, RenderCommand::DrawIndicatorLine { .. }))
            .count();
        
        assert!(indicator_lines > 0, "No indicator lines rendered");
    }
    
    #[test]
    fn test_panel_system() {
        let mut manager = PanelManager::new();
        
        // Add main chart panel
        let chart_id = manager.add_panel(PanelConfig {
            panel_type: PanelType::Chart,
            stretch_factor: 3.0,
            indicators: vec![],
        }).unwrap();
        
        // Add RSI panel
        let rsi_id = manager.add_panel(PanelConfig {
            panel_type: PanelType::Indicator,
            stretch_factor: 1.0,
            indicators: vec!["rsi".to_string()],
        }).unwrap();
        
        assert_eq!(manager.panels().len(), 2);
        
        // Test removal
        manager.remove_panel(&rsi_id).unwrap();
        assert_eq!(manager.panels().len(), 1);
    }
    
    #[test]
    fn test_drawing_tools() {
        let mut manager = DrawingManager::new();
        
        // Start trend line
        let id = manager.start_drawing(
            DrawingType::TrendLine,
            Point { timestamp: 1000, price: 100.0 }
        );
        
        // Update with second point
        manager.update_active_drawing(
            Point { timestamp: 2000, price: 200.0 }
        ).unwrap();
        
        // Finalize
        manager.finalize_drawing().unwrap();
        
        // Verify drawing was created
        assert_eq!(manager.drawings().len(), 1);
        
        // Test hit testing
        let hit = manager.hit_test(1500.0, 150.0, 10.0);
        assert!(hit.is_some(), "Should hit the trend line");
    }
    
    fn generate_test_candles(count: usize) -> Vec<Candle> {
        (0..count).map(|i| Candle {
            timestamp: i as i64 * 60000,
            open: 100.0 + (i as f64 * 0.5),
            high: 105.0 + (i as f64 * 0.5),
            low: 95.0 + (i as f64 * 0.5),
            close: 102.0 + (i as f64 * 0.5),
            volume: 1000.0,
        }).collect()
    }
}
```

**Frontend tests:**

**File: `apps/frontend/src/tests/realtime-client.test.ts`**

```typescript
import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { RealtimeClient } from '../lib/realtime-client';

describe('RealtimeClient', () => {
    let client: RealtimeClient;
    
    beforeEach(() => {
        client = new RealtimeClient('BTCUSD');
    });
    
    afterEach(() => {
        client.disconnect();
    });
    
    it('should connect to WebSocket', (done) => {
        client.connect();
        
        window.addEventListener('connectionStatusChanged', (e: any) => {
            if (e.detail.status === 'connected') {
                expect(e.detail.status).toBe('connected');
                done();
            }
        });
    });
    
    it('should receive initial candles', (done) => {
        client.connect();
        
        // Mock WASM
        (window as any).getWasm = () => ({
            load_candles: (json: string) => {
                const candles = JSON.parse(json);
                expect(candles.length).toBeGreaterThan(0);
                done();
            }
        });
    });
});
```

**Deliverable:** Integration test suite

---

### Task 12.2: User Documentation

**File: `docs/USER_GUIDE.md`**

```markdown
# Loom Trading Platform - User Guide

## Getting Started

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/loom.git
   cd loom
   ```

2. Install dependencies:
   ```bash
   pnpm install
   ```

3. Build WASM modules:
   ```bash
   ./build-wasm.sh
   ```

4. Start development server:
   ```bash
   pnpm dev
   ```

5. Open http://localhost:4321

### First Chart

When you open Loom, you'll see:
- Main chart area (center)
- Drawing toolbar (left)
- Control panel (top right)
- Connection status (top right)

## Chart Navigation

### Pan and Zoom
- **Pan**: Click and drag on the chart
- **Zoom**: Scroll wheel (up = zoom in, down = zoom out)
- **Reset**: Double-click on the chart

### Axis Controls
- **Y-axis (Price)**: Drag to scale vertically
- **X-axis (Time)**: Drag to scroll horizontally
- **Auto-scale**: Right-click axis → Enable Auto Scale

### Timeframe Switching
- Select timeframe from dropdown (top)
- Options: 1m, 5m, 15m, 1h, 4h, 1d
- Historical candles load automatically

## Indicators

### Adding Indicators

1. Click "Indicators" button (top toolbar)
2. Search or browse categories
3. Select indicator
4. Choose display mode:
   - **Overlay**: Renders on main chart (e.g., SMA, Bollinger Bands)
   - **Separate Panel**: Creates new panel below (e.g., RSI, MACD)
5. Click "Add Indicator"

### Configuring Indicators

1. Open "Panels" sidebar (top right)
2. Find your indicator
3. Click settings gear icon
4. Adjust parameters
5. Changes apply immediately

### Supported Indicators (70+)

**Trend** (10): SMA, EMA, WMA, DEMA, TEMA, HMA, VWMA, SMMA, KAMA, MAMA

**Momentum** (15): RSI, MACD, Stochastic, Williams %R, CCI, ROC, Momentum, TSI, UO, AO, CMO, TRIX, DPO, KST, DMI

**Volatility** (8): Bollinger Bands, ATR, Keltner Channels, Donchian Channels, StdDev, Chaikin Volatility, Historical Volatility, True Range

**Volume** (10): Volume, OBV, CMF, MFI, VWAP, A/D, Force Index, EoM, Volume Oscillator, PVT

**Bill Williams** (5): Alligator, Fractals, Gator, MFI, AO

**Ichimoku** (1): Ichimoku Cloud

## Drawing Tools

### Using Drawing Tools

1. Select tool from drawing toolbar (left):
   - **Cursor (V)**: Select/move drawings
   - **Trend Line (T)**: Draw trend lines
   - **Horizontal Line (H)**: Price levels
   - **Vertical Line**: Time markers
   - **Rectangle (R)**: Price ranges
   - **Fibonacci (F)**: Retracement levels

2. Click on chart to start drawing
3. Click again to finish (or drag for some tools)
4. Press **Esc** to cancel

### Editing Drawings

1. Select cursor tool (V)
2. Click on drawing to select
3. Drag control points to adjust
4. Right-click for more options:
   - Change color
   - Lock/unlock
   - Delete

### Keyboard Shortcuts

- **V**: Cursor (select)
- **T**: Trend line
- **H**: Horizontal line
- **R**: Rectangle
- **F**: Fibonacci
- **Esc**: Cancel drawing
- **Delete**: Remove selected
- **Ctrl+Z**: Undo
- **Ctrl+Shift+Z**: Redo

## Panels

### Multi-Panel Layouts

Create custom layouts with multiple panels:

1. Click "Panels" button (top right)
2. Use layout presets or create custom
3. Drag separator between panels to resize
4. Drag panel headers to reorder

### Layout Presets

- **Default**: Single chart
- **Multi-Timeframe**: 3 charts (1h, 15m, 5m)
- **Indicator Dashboard**: Chart + RSI + MACD + Volume
- **Trading**: Chart + Volume

### Saving Layouts

1. Arrange panels as desired
2. Click "Save Layout" in Panels sidebar
3. Enter name
4. Layout auto-saves to browser storage

## Settings

### Theme
- Light/Dark mode toggle (top right)
- Auto-detects system preference

### Default Timeframe
- Settings → Default Timeframe
- Applies on app startup

### Data Feed
- Settings → Data Feed
- Configure Capital.com API key
- Test connection

## Troubleshooting

### Chart Not Loading
1. Check connection status (top right)
2. Reload page (Ctrl+R)
3. Clear cache: Settings → Clear Cache

### Indicators Not Showing
1. Check if indicator is visible (Panels sidebar)
2. Verify timeframe has enough data
3. Adjust scale (right-click Y-axis → Reset Scale)

### Performance Issues
1. Reduce number of active indicators
2. Use shorter timeframe
3. Close unused panels

## Keyboard Shortcuts Reference

| Shortcut | Action |
|----------|--------|
| V | Select tool |
| T | Trend line |
| H | Horizontal line |
| R | Rectangle |
| F | Fibonacci |
| Esc | Cancel/Deselect |
| Delete | Remove selected |
| Ctrl+Z | Undo |
| Ctrl+Shift+Z | Redo |
| Ctrl+R | Reload |
```

**Deliverable:** Complete user guide

---

### Task 12.3: Developer Documentation

**File: `docs/ARCHITECTURE.md`**

Create comprehensive developer documentation covering:

1. System Overview
2. Chart Engine (Rust chartcore)
3. WASM Bindings
4. Frontend Architecture (Astro + Alpine.js)
5. Realtime Data Flow (Phoenix WebSocket)
6. Adding New Indicators (tutorial)
7. Adding Drawing Tools (tutorial)
8. Testing Strategy

**Deliverable:** Developer architecture documentation

---

### Task 12.4: Performance Benchmarks

**File: `PERFORMANCE.md`**

Document baseline performance metrics:

```markdown
# Performance Benchmarks

Tested on: MacBook Pro M1, 16GB RAM, Chrome 120

## Rendering Performance

| Scenario | Frame Time | FPS | Target |
|----------|-----------|-----|--------|
| 1000 candles, 0 indicators | 8.2ms | 120 | ✅ 60fps |
| 1000 candles, 5 indicators | 14.1ms | 70 | ✅ 60fps |
| 1000 candles, 10 indicators | 22.3ms | 45 | ⚠️ 60fps |
| Pan/scroll | 4.5ms | 220 | ✅ 60fps |
| Drawing tool (active) | 6.1ms | 160 | ✅ 60fps |

## Load Times

| Operation | Time | Target |
|-----------|------|--------|
| Initial page load | 420ms | ✅ < 500ms |
| WASM initialization | 180ms | ✅ < 500ms |
| First render | 90ms | ✅ < 200ms |
| Timeframe switch (1m→1h) | 150ms | ✅ < 200ms |
| Add indicator | 65ms | ✅ < 100ms |

## Memory Usage

| Scenario | Memory | Target |
|----------|--------|--------|
| Initial load | 42MB | ✅ < 100MB |
| 1000 candles loaded | 68MB | ✅ < 100MB |
| 10 indicators active | 89MB | ✅ < 100MB |
| After 1 hour usage | 105MB | ⚠️ < 100MB |

## Bundle Sizes

| Asset | Size (gzipped) | Target |
|-------|---------------|--------|
| trading_ui.wasm | 387KB | ✅ < 500KB |
| main.js | 125KB | ✅ < 200KB |
| styles.css | 18KB | ✅ < 50KB |
| **Total** | **530KB** | ✅ < 1MB |

## Network Performance

| Metric | Measurement | Target |
|--------|-------------|--------|
| WebSocket latency | 32ms | ✅ < 50ms |
| Candle update → render | 45ms | ✅ < 100ms |
| Reconnection time | 1.2s | ✅ < 5s |

## Optimization Opportunities

1. **10 indicators at 22ms**: Implement indicator calculation caching
2. **Memory after 1 hour (105MB)**: Implement candle eviction from memory
3. **WASM size (387KB)**: Apply additional compression with wasm-opt -Oz

## Testing Methodology

- **Rendering**: Chrome DevTools Performance tab, 10-second recording
- **Load times**: Network tab, hard reload (Ctrl+Shift+R)
- **Memory**: Chrome Task Manager, heap snapshots
- **Bundle sizes**: Production build, gzipped
- **Network**: Phoenix logger + browser Network tab
```

**Deliverable:** Performance benchmark document

---

## Phase 6 Completion Checklist

- [ ] Performance profiling completed
- [ ] Bottlenecks identified and fixed
- [ ] 60fps target achieved (with 5 indicators)
- [ ] WASM binary optimized (< 500KB gzipped)
- [ ] Global error boundary implemented
- [ ] Toast notification system working
- [ ] Loading states for all async operations
- [ ] Integration tests (Rust)
- [ ] Frontend tests (Vitest)
- [ ] User guide documentation
- [ ] Developer architecture docs
- [ ] Performance benchmarks documented
- [ ] All console errors eliminated
- [ ] Mobile-responsive (bonus)

## Success Criteria

At the end of Phase 6:
1. 60fps with 5+ indicators active
2. Zero unhandled errors in normal operation
3. Professional UX (loading states, error messages)
4. Comprehensive documentation
5. Test coverage for critical paths
6. Production-ready codebase

**Time Budget:** 2 weeks  
**Risk Level:** Low (polish and documentation)  
**Dependencies:** Phase 5 (realtime streaming)

---

## Post-Phase 6: Production Deployment

After Phase 6, Loom is production-ready. Next steps:

1. **Deploy to production**
   - Setup hosting (Vercel, Netlify, or custom)
   - Configure Phoenix backend (Fly.io, Render)
   - Setup monitoring (Sentry, LogRocket)

2. **User feedback & iteration**
   - Beta testing
   - Bug fixes
   - UX improvements

3. **Begin Phase 7** (from ROADMAP_OVERVIEW.md)
   - Advanced drawing tools
   - WebWorker migration
   - Advanced indicators
   - Trading integration
   - Mobile app (Tauri)
   - AI features
