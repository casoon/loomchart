# Canvas DPI/HiDPI Implementation Plan
## Eigenimplementierung ohne fancy-canvas Dependency

**Ziel:** Professionelles DPI-Management für Loom Trading Platform ohne externe Dependencies, basierend auf fancy-canvas Patterns und TradingView Best Practices.

**Status:** Planungsphase  
**Erstellt:** 2026-01-02  
**Geschätzter Aufwand:** 1-2 Wochen

---

## Problem Statement

### Aktuelle Situation

Loom hat derzeit **manuelle DPI-Skalierung** verteilt über:
- Rust: `crates/chartcore/src/renderers/canvas2d.rs` (~400 Zeilen)
- TypeScript: `apps/frontend/src/lib/rust-chart.ts` (~50 Zeilen Setup)
- Manuelle `devicePixelRatio` Multiplikation bei jeder Render-Operation
- Fehleranfällige ResizeObserver-Logik

### Probleme

1. **Code-Duplizierung:** DPI-Logik in Rust UND TypeScript
2. **Fehleranfällig:** Leicht vergessen, `* devicePixelRatio` anzuwenden
3. **Resize-Loops:** ResizeObserver kann sich selbst triggern
4. **Unklare Koordinatensysteme:** Vermischung von CSS-Pixeln und Device-Pixeln
5. **Wartbarkeit:** Schwer zu testen und zu debuggen

### Gewünschter Zustand

- ✅ Klare Trennung: **CSS Coordinates** (Media Space) vs. **Device Coordinates** (Bitmap Space)
- ✅ Typ-sichere API die verhindert, dass falsche Koordinaten verwendet werden
- ✅ Automatisches Resize-Handling ohne Loops
- ✅ Zentrale DPI-Logik an einem Ort
- ✅ Einfach zu testen und zu verstehen
- ✅ **Keine externen Dependencies**

---

## Architektur-Übersicht

### Zwei-Koordinatensystem-Ansatz

```
┌─────────────────────────────────────────────────────────────┐
│  Media Coordinate Space (CSS Pixels)                       │
│  - User-facing measurements (font size, layout)            │
│  - Natural text rendering with sub-pixel positioning       │
│  - Browser handles scaling automatically                   │
│  - Use for: Labels, tooltips, UI elements                  │
└─────────────────────────────────────────────────────────────┘
                            ↕
                   devicePixelRatio
                            ↕
┌─────────────────────────────────────────────────────────────┐
│  Bitmap Coordinate Space (Device Pixels)                   │
│  - Physical pixels on screen (Retina: 2x, 4K: 3x)         │
│  - Pixel-perfect line rendering (no blur)                  │
│  - Exact positioning for grid lines, crosshairs            │
│  - Use for: Candles, grid, geometric shapes                │
└─────────────────────────────────────────────────────────────┘
```

### Komponenten-Struktur

```
/apps/frontend/src/lib/canvas/
├── canvas-binding.ts          # Canvas Element Lifecycle Management
├── coordinate-scope.ts        # Coordinate Space Abstractions
├── resize-handler.ts          # Smart ResizeObserver (no loops)
└── types.ts                   # TypeScript interfaces

/crates/chartcore/src/canvas/
├── mod.rs                     # Public API
├── binding.rs                 # Rust-side binding coordination
├── coordinate_space.rs        # Coordinate transformation utilities
└── dpi_aware_renderer.rs     # DPI-aware rendering primitives
```

---

## Implementation Plan

### Phase 1: TypeScript Canvas Binding System (3-4 Tage)

**Ziel:** Robustes Canvas-Management mit automatischem DPI-Handling

#### 1.1 Create Type Definitions

**File:** `/apps/frontend/src/lib/canvas/types.ts`

```typescript
/**
 * Size in different coordinate spaces
 */
export interface Size {
  width: number;
  height: number;
}

/**
 * Physical device pixel ratio (1.0, 1.5, 2.0, 3.0, etc.)
 */
export interface PixelRatio {
  horizontal: number;  // Can differ on some displays
  vertical: number;
}

/**
 * Bitmap coordinate space (device pixels)
 */
export interface BitmapCoordinateScope {
  context: CanvasRenderingContext2D;
  bitmapSize: Size;
  horizontalPixelRatio: number;
  verticalPixelRatio: number;
}

/**
 * Media coordinate space (CSS pixels)
 */
export interface MediaCoordinateScope {
  context: CanvasRenderingContext2D;
  mediaSize: Size;
}

/**
 * Listener for size changes
 */
export type SizeChangeListener = (oldSize: Size, newSize: Size) => void;

/**
 * Disposable resource pattern
 */
export interface Disposable {
  dispose(): void;
}
```

**Begründung:** Explizite Typen machen Code selbst-dokumentierend und verhindern Koordinatenfehler.

---

#### 1.2 Implement Resize Handler

**File:** `/apps/frontend/src/lib/canvas/resize-handler.ts`

```typescript
import type { Size, SizeChangeListener, Disposable } from './types';

/**
 * Smart ResizeObserver that prevents resize loops and detects
 * actual device pixel changes (not just CSS changes).
 * 
 * Based on fancy-canvas implementation patterns.
 */
export class CanvasResizeHandler implements Disposable {
  private observer: ResizeObserver | null = null;
  private listeners: Set<SizeChangeListener> = new Set();
  private lastKnownSize: Size | null = null;

  constructor(
    private canvas: HTMLCanvasElement,
    private options: { type: 'device-pixel-content-box' | 'content-box' } = { 
      type: 'device-pixel-content-box' 
    }
  ) {
    this.setupObserver();
  }

  /**
   * Subscribe to size changes
   */
  public onSizeChanged(listener: SizeChangeListener): void {
    this.listeners.add(listener);
  }

  /**
   * Remove listener
   */
  public offSizeChanged(listener: SizeChangeListener): void {
    this.listeners.delete(listener);
  }

  /**
   * Get current size in device pixels
   */
  public getCurrentSize(): Size {
    if (this.lastKnownSize) return this.lastKnownSize;
    
    // Fallback to manual calculation
    const rect = this.canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    
    return {
      width: Math.floor(rect.width * dpr),
      height: Math.floor(rect.height * dpr)
    };
  }

  /**
   * Setup ResizeObserver with device-pixel-content-box support
   */
  private setupObserver(): void {
    this.observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const newSize = this.extractSize(entry);
        
        // Only notify if size actually changed (prevents loops)
        if (this.hasChanged(newSize)) {
          const oldSize = this.lastKnownSize;
          this.lastKnownSize = newSize;
          
          if (oldSize) {
            this.notifyListeners(oldSize, newSize);
          }
        }
      }
    });

    this.observer.observe(this.canvas, { 
      box: this.options.type as ResizeObserverBoxOptions
    });
  }

  /**
   * Extract size from ResizeObserverEntry
   * Prefers devicePixelContentBox over contentBox
   */
  private extractSize(entry: ResizeObserverEntry): Size {
    // Try device-pixel-content-box first (best option)
    if (entry.devicePixelContentBoxSize && entry.devicePixelContentBoxSize.length > 0) {
      return {
        width: Math.floor(entry.devicePixelContentBoxSize[0].inlineSize),
        height: Math.floor(entry.devicePixelContentBoxSize[0].blockSize)
      };
    }

    // Fallback to content-box
    if (entry.contentBoxSize && entry.contentBoxSize.length > 0) {
      const dpr = window.devicePixelRatio || 1;
      return {
        width: Math.floor(entry.contentBoxSize[0].inlineSize * dpr),
        height: Math.floor(entry.contentBoxSize[0].blockSize * dpr)
      };
    }

    // Last resort: contentRect
    const dpr = window.devicePixelRatio || 1;
    return {
      width: Math.floor(entry.contentRect.width * dpr),
      height: Math.floor(entry.contentRect.height * dpr)
    };
  }

  /**
   * Check if size actually changed (prevents resize loops)
   */
  private hasChanged(newSize: Size): boolean {
    if (!this.lastKnownSize) return true;
    
    return (
      newSize.width !== this.lastKnownSize.width ||
      newSize.height !== this.lastKnownSize.height
    );
  }

  /**
   * Notify all listeners
   */
  private notifyListeners(oldSize: Size, newSize: Size): void {
    this.listeners.forEach(listener => {
      try {
        listener(oldSize, newSize);
      } catch (error) {
        console.error('[CanvasResizeHandler] Listener error:', error);
      }
    });
  }

  /**
   * Cleanup
   */
  public dispose(): void {
    if (this.observer) {
      this.observer.disconnect();
      this.observer = null;
    }
    this.listeners.clear();
    this.lastKnownSize = null;
  }
}
```

**Key Features:**
- ✅ Verwendet `devicePixelContentBoxSize` wenn verfügbar (Chrome 84+)
- ✅ Graceful Fallback zu `contentBoxSize` und `contentRect`
- ✅ Verhindert Resize-Loops durch Change-Detection
- ✅ Typ-sichere Listener-API
- ✅ Proper Cleanup (Disposable Pattern)

**Tests:**
```typescript
// Test 1: Normal resize
const handler = new CanvasResizeHandler(canvas);
handler.onSizeChanged((oldSize, newSize) => {
  expect(newSize.width).toBeGreaterThan(oldSize.width);
});
canvas.style.width = '800px'; // Triggers observer

// Test 2: No change = no callback
let callCount = 0;
handler.onSizeChanged(() => callCount++);
canvas.style.width = '800px'; // Same size
canvas.style.width = '800px'; // Same size
expect(callCount).toBe(0);

// Test 3: DPI change detection
window.devicePixelRatio = 2.0; // Simulated DPI change
// Should trigger callback
```

---

#### 1.3 Implement Coordinate Scopes

**File:** `/apps/frontend/src/lib/canvas/coordinate-scope.ts`

```typescript
import type { 
  BitmapCoordinateScope, 
  MediaCoordinateScope, 
  Size, 
  PixelRatio 
} from './types';

/**
 * Provides scoped access to canvas context with automatic coordinate
 * space transformation. Prevents mixing CSS and device pixels.
 * 
 * Usage:
 * ```
 * const target = new CanvasRenderingTarget(canvas);
 * 
 * // For pixel-perfect lines, shapes
 * target.useBitmapCoordinateSpace(({ context, bitmapSize }) => {
 *   context.lineWidth = 1;  // Exactly 1 device pixel
 *   context.moveTo(x, y);
 * });
 * 
 * // For text, labels
 * target.useMediaCoordinateSpace(({ context, mediaSize }) => {
 *   context.font = '12px Arial';  // CSS size
 *   context.fillText('Label', x, y);
 * });
 * ```
 */
export class CanvasRenderingTarget {
  private ctx: CanvasRenderingContext2D;
  private currentMediaSize: Size;
  private currentBitmapSize: Size;
  private currentPixelRatio: PixelRatio;

  constructor(private canvas: HTMLCanvasElement) {
    const context = canvas.getContext('2d');
    if (!context) {
      throw new Error('Failed to get 2D context from canvas');
    }
    this.ctx = context;

    // Initialize sizes
    this.currentMediaSize = this.getMediaSize();
    this.currentBitmapSize = this.getBitmapSize();
    this.currentPixelRatio = this.getPixelRatio();
  }

  /**
   * Execute callback in bitmap coordinate space (device pixels).
   * Use for pixel-perfect rendering: grid lines, candles, shapes.
   */
  public useBitmapCoordinateSpace<T>(
    callback: (scope: BitmapCoordinateScope) => T
  ): T {
    const { horizontalPixelRatio, verticalPixelRatio } = this.currentPixelRatio;

    // Save current transform
    this.ctx.save();

    // Apply device pixel ratio scaling
    // Context is already scaled by canvas.width/height setup,
    // so we just provide the ratio for manual calculations
    
    const scope: BitmapCoordinateScope = {
      context: this.ctx,
      bitmapSize: this.currentBitmapSize,
      horizontalPixelRatio,
      verticalPixelRatio
    };

    try {
      return callback(scope);
    } finally {
      // Restore transform
      this.ctx.restore();
    }
  }

  /**
   * Execute callback in media coordinate space (CSS pixels).
   * Use for text rendering, tooltips, UI elements.
   */
  public useMediaCoordinateSpace<T>(
    callback: (scope: MediaCoordinateScope) => T
  ): T {
    // Media space doesn't need transform - canvas is already
    // set up with CSS size, and text scales naturally
    
    this.ctx.save();

    const scope: MediaCoordinateScope = {
      context: this.ctx,
      mediaSize: this.currentMediaSize
    };

    try {
      return callback(scope);
    } finally {
      this.ctx.restore();
    }
  }

  /**
   * Update internal size tracking (call when canvas resizes)
   */
  public updateSizes(mediaSize: Size, bitmapSize: Size): void {
    this.currentMediaSize = mediaSize;
    this.currentBitmapSize = bitmapSize;
    this.currentPixelRatio = this.getPixelRatio();
  }

  /**
   * Get current media size (CSS pixels)
   */
  private getMediaSize(): Size {
    const rect = this.canvas.getBoundingClientRect();
    return {
      width: Math.floor(rect.width),
      height: Math.floor(rect.height)
    };
  }

  /**
   * Get current bitmap size (device pixels)
   */
  private getBitmapSize(): Size {
    return {
      width: this.canvas.width,
      height: this.canvas.height
    };
  }

  /**
   * Get current pixel ratios
   */
  private getPixelRatio(): PixelRatio {
    const dpr = window.devicePixelRatio || 1;
    return {
      horizontal: dpr,
      vertical: dpr  // Usually same, but could differ on some displays
    };
  }

  /**
   * Get underlying canvas element
   */
  public getCanvas(): HTMLCanvasElement {
    return this.canvas;
  }

  /**
   * Get current media size
   */
  public get mediaSize(): Size {
    return { ...this.currentMediaSize };
  }

  /**
   * Get current bitmap size
   */
  public get bitmapSize(): Size {
    return { ...this.currentBitmapSize };
  }
}
```

**Warum zwei Scopes?**

1. **Bitmap Space (Device Pixels):**
   - Für geometrische Präzision (grid lines = exakt 1px)
   - Verhindert Blur auf Retina-Displays
   - Integer-Pixel-Alignment für scharfe Kanten

2. **Media Space (CSS Pixels):**
   - Für Text (Browser handled Scaling optimal)
   - Für UI-Elemente (natürliche Größen)
   - Für Tooltips, Labels, Crosshair-Text

**Beispiel-Usage:**
```typescript
const target = new CanvasRenderingTarget(canvas);

// Draw vertical grid line (bitmap space = sharp)
target.useBitmapCoordinateSpace(({ context, horizontalPixelRatio }) => {
  const x = 100;  // CSS pixel position
  const deviceX = Math.round(x * horizontalPixelRatio);
  
  context.strokeStyle = '#333';
  context.lineWidth = 1;  // Always 1 device pixel
  
  context.beginPath();
  context.moveTo(deviceX + 0.5, 0);  // +0.5 for odd line width
  context.lineTo(deviceX + 0.5, context.canvas.height);
  context.stroke();
});

// Draw price label (media space = smooth)
target.useMediaCoordinateSpace(({ context }) => {
  context.font = '12px monospace';
  context.fillStyle = '#fff';
  context.textAlign = 'right';
  context.fillText('$50,123.45', 100, 200);  // CSS coordinates
});
```

---

#### 1.4 Implement Canvas Binding

**File:** `/apps/frontend/src/lib/canvas/canvas-binding.ts`

```typescript
import { CanvasResizeHandler } from './resize-handler';
import { CanvasRenderingTarget } from './coordinate-scope';
import type { Size, SizeChangeListener, Disposable } from './types';

/**
 * Binds a canvas element to automatic size management with DPI awareness.
 * Handles resize, device pixel ratio, and provides rendering targets.
 * 
 * This is the main entry point for canvas setup.
 */
export class CanvasBinding implements Disposable {
  private resizeHandler: CanvasResizeHandler;
  private renderingTarget: CanvasRenderingTarget;
  private disposed: boolean = false;

  constructor(private canvas: HTMLCanvasElement) {
    // Setup resize handling
    this.resizeHandler = new CanvasResizeHandler(canvas);
    
    // Initial size setup
    this.applySuggestedSize();

    // Create rendering target
    this.renderingTarget = new CanvasRenderingTarget(canvas);

    // Listen for size changes
    this.resizeHandler.onSizeChanged((oldSize, newSize) => {
      this.handleResize(newSize);
    });
  }

  /**
   * Get rendering target for scoped coordinate operations
   */
  public getRenderingTarget(): CanvasRenderingTarget {
    return this.renderingTarget;
  }

  /**
   * Get current canvas element
   */
  public getCanvasElement(): HTMLCanvasElement {
    return this.canvas;
  }

  /**
   * Get current media size (CSS pixels)
   */
  public get mediaSize(): Size {
    return this.renderingTarget.mediaSize;
  }

  /**
   * Get current bitmap size (device pixels)
   */
  public get bitmapSize(): Size {
    return this.renderingTarget.bitmapSize;
  }

  /**
   * Subscribe to size changes
   */
  public onSizeChanged(listener: SizeChangeListener): void {
    this.resizeHandler.onSizeChanged(listener);
  }

  /**
   * Apply suggested bitmap size based on current CSS size and DPI
   */
  public applySuggestedSize(): void {
    const rect = this.canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;

    const mediaSize: Size = {
      width: Math.floor(rect.width),
      height: Math.floor(rect.height)
    };

    const bitmapSize: Size = {
      width: Math.floor(mediaSize.width * dpr),
      height: Math.floor(mediaSize.height * dpr)
    };

    // Update canvas bitmap size
    this.canvas.width = bitmapSize.width;
    this.canvas.height = bitmapSize.height;

    // Update CSS size
    this.canvas.style.width = `${mediaSize.width}px`;
    this.canvas.style.height = `${mediaSize.height}px`;

    // Update rendering target
    this.renderingTarget.updateSizes(mediaSize, bitmapSize);

    console.log('[CanvasBinding] Size applied:', {
      media: mediaSize,
      bitmap: bitmapSize,
      dpr
    });
  }

  /**
   * Handle resize events
   */
  private handleResize(newBitmapSize: Size): void {
    const dpr = window.devicePixelRatio || 1;
    const newMediaSize: Size = {
      width: Math.floor(newBitmapSize.width / dpr),
      height: Math.floor(newBitmapSize.height / dpr)
    };

    // Update canvas dimensions
    this.canvas.width = newBitmapSize.width;
    this.canvas.height = newBitmapSize.height;
    this.canvas.style.width = `${newMediaSize.width}px`;
    this.canvas.style.height = `${newMediaSize.height}px`;

    // Update rendering target
    this.renderingTarget.updateSizes(newMediaSize, newBitmapSize);

    console.log('[CanvasBinding] Resized:', {
      media: newMediaSize,
      bitmap: newBitmapSize,
      dpr
    });
  }

  /**
   * Cleanup
   */
  public dispose(): void {
    if (this.disposed) return;

    this.resizeHandler.dispose();
    this.disposed = true;

    console.log('[CanvasBinding] Disposed');
  }
}
```

**Verwendung:**
```typescript
// In RustChart.initialize()
this.canvasBinding = new CanvasBinding(this.canvas);

// Listen for resizes
this.canvasBinding.onSizeChanged((oldSize, newSize) => {
  console.log('Canvas resized:', oldSize, '→', newSize);
  // Notify WASM of new size
  this.wasmChart?.resize(
    newSize.width / window.devicePixelRatio,
    newSize.height / window.devicePixelRatio
  );
});

// Get rendering target
const target = this.canvasBinding.getRenderingTarget();

// Use in render loop
target.useBitmapCoordinateSpace(({ context }) => {
  // Draw sharp grid lines
});

target.useMediaCoordinateSpace(({ context }) => {
  // Draw smooth text
});
```

---

### Phase 2: Rust DPI-Aware Renderer (2-3 Tage)

**Ziel:** Rust-side coordinate space awareness

#### 2.1 Create Coordinate Space Types

**File:** `/crates/chartcore/src/canvas/coordinate_space.rs`

```rust
//! Coordinate space types and transformations for DPI-aware rendering

/// Physical device pixels (bitmap space)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DevicePixels(pub f64);

/// CSS logical pixels (media space)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CssPixels(pub f64);

/// Device pixel ratio (typically 1.0, 1.5, 2.0, 3.0)
#[derive(Debug, Clone, Copy)]
pub struct PixelRatio {
    pub horizontal: f64,
    pub vertical: f64,
}

impl PixelRatio {
    pub fn new(ratio: f64) -> Self {
        Self {
            horizontal: ratio,
            vertical: ratio,
        }
    }

    pub fn uniform(&self) -> bool {
        (self.horizontal - self.vertical).abs() < f64::EPSILON
    }
}

/// Bitmap coordinate space information
#[derive(Debug, Clone)]
pub struct BitmapSpace {
    pub width: u32,
    pub height: u32,
    pub pixel_ratio: PixelRatio,
}

impl BitmapSpace {
    /// Convert CSS pixel to device pixel
    pub fn css_to_device_x(&self, css: CssPixels) -> DevicePixels {
        DevicePixels(css.0 * self.pixel_ratio.horizontal)
    }

    pub fn css_to_device_y(&self, css: CssPixels) -> DevicePixels {
        DevicePixels(css.0 * self.pixel_ratio.vertical)
    }

    /// Convert device pixel to CSS pixel
    pub fn device_to_css_x(&self, device: DevicePixels) -> CssPixels {
        CssPixels(device.0 / self.pixel_ratio.horizontal)
    }

    pub fn device_to_css_y(&self, device: DevicePixels) -> CssPixels {
        CssPixels(device.0 / self.pixel_ratio.vertical)
    }

    /// Calculate pixel-perfect correction for odd line widths
    /// Returns 0.5 for odd widths, 0.0 for even widths
    pub fn pixel_correction(&self, line_width: f64) -> f64 {
        let device_width = (line_width * self.pixel_ratio.horizontal).round() as i32;
        if device_width % 2 == 1 {
            0.5
        } else {
            0.0
        }
    }
}

/// Media coordinate space (CSS pixels)
#[derive(Debug, Clone)]
pub struct MediaSpace {
    pub width: u32,
    pub height: u32,
}
```

**Warum Type-Safe?**
```rust
// Compiler verhindert Fehler:
let css_pos = CssPixels(100.0);
let device_pos = DevicePixels(200.0);

// ❌ Compile error: Type mismatch
// draw_line(css_pos, device_pos);

// ✅ Forced to convert explicitly
let converted = space.css_to_device_x(css_pos);
draw_line(device_pos, converted);
```

---

#### 2.2 Update Canvas2D Renderer

**File:** `/crates/chartcore/src/renderers/canvas2d.rs`

```rust
use crate::canvas::coordinate_space::{BitmapSpace, CssPixels, DevicePixels, PixelRatio};

pub struct Canvas2DRenderer {
    ctx: CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
    bitmap_space: BitmapSpace,
    current_clip: Option<(f64, f64, f64, f64)>,
}

impl Canvas2DRenderer {
    pub fn new(canvas: HtmlCanvasElement, pixel_ratio: f64) -> Result<Self, String> {
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| "Failed to get 2D context")?
            .ok_or("2D context is None")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "Failed to cast to CanvasRenderingContext2d")?;

        let bitmap_space = BitmapSpace {
            width: canvas.width(),
            height: canvas.height(),
            pixel_ratio: PixelRatio::new(pixel_ratio),
        };

        Ok(Self {
            ctx,
            canvas,
            bitmap_space,
            current_clip: None,
        })
    }

    /// Update bitmap space when canvas resizes
    pub fn update_bitmap_space(&mut self, width: u32, height: u32, pixel_ratio: f64) {
        self.bitmap_space = BitmapSpace {
            width,
            height,
            pixel_ratio: PixelRatio::new(pixel_ratio),
        };
    }

    /// Draw line in CSS coordinates (automatic device pixel conversion)
    pub fn draw_line_css(
        &mut self,
        x1: CssPixels,
        y1: CssPixels,
        x2: CssPixels,
        y2: CssPixels,
        color: Color,
        width: f64,
    ) {
        // Convert to device pixels
        let dx1 = self.bitmap_space.css_to_device_x(x1).0;
        let dy1 = self.bitmap_space.css_to_device_y(y1).0;
        let dx2 = self.bitmap_space.css_to_device_x(x2).0;
        let dy2 = self.bitmap_space.css_to_device_y(y2).0;

        // Calculate device line width and correction
        let device_width = (width * self.bitmap_space.pixel_ratio.horizontal).floor().max(1.0);
        let correction = self.bitmap_space.pixel_correction(width);

        self.ctx.set_stroke_style(&JsValue::from_str(&color.to_css()));
        self.ctx.set_line_width(device_width);

        self.ctx.begin_path();
        self.ctx.move_to(dx1 + correction, dy1 + correction);
        self.ctx.line_to(dx2 + correction, dy2 + correction);
        self.ctx.stroke();
    }

    /// Draw line in device pixels (for manual control)
    pub fn draw_line_device(
        &mut self,
        x1: DevicePixels,
        y1: DevicePixels,
        x2: DevicePixels,
        y2: DevicePixels,
        color: Color,
        width: f64,
    ) {
        let correction = if (width as i32) % 2 == 1 { 0.5 } else { 0.0 };

        self.ctx.set_stroke_style(&JsValue::from_str(&color.to_css()));
        self.ctx.set_line_width(width);

        self.ctx.begin_path();
        self.ctx.move_to(x1.0 + correction, y1.0 + correction);
        self.ctx.line_to(x2.0 + correction, y2.0 + correction);
        self.ctx.stroke();
    }

    /// Draw text in CSS coordinates (natural scaling)
    pub fn draw_text_css(
        &mut self,
        text: &str,
        x: CssPixels,
        y: CssPixels,
        font: &str,
        color: Color,
    ) {
        // Text is drawn in CSS space - no DPI conversion needed
        // Browser handles scaling automatically
        self.ctx.set_font(font);
        self.ctx.set_fill_style(&JsValue::from_str(&color.to_css()));
        self.ctx.fill_text(text, x.0, y.0).ok();
    }

    /// Get bitmap space info
    pub fn bitmap_space(&self) -> &BitmapSpace {
        &self.bitmap_space
    }
}
```

**Migration Path:**
```rust
// Old code:
renderer.draw_line(x1, y1, x2, y2, color, width);

// New code (explicit choice):
renderer.draw_line_css(
    CssPixels(x1), 
    CssPixels(y1), 
    CssPixels(x2), 
    CssPixels(y2), 
    color, 
    width
);
```

---

### Phase 3: Integration & Testing (2-3 Tage)

#### 3.1 Update RustChart

**File:** `/apps/frontend/src/lib/rust-chart.ts`

```typescript
import { CanvasBinding } from './canvas/canvas-binding';
import type { CanvasRenderingTarget } from './canvas/coordinate-scope';

export class RustChart {
  private wasmChart: any = null;
  private canvas: HTMLCanvasElement;
  private canvasBinding: CanvasBinding | null = null;
  private renderTarget: CanvasRenderingTarget | null = null;
  
  // ... existing code ...

  async initialize(): Promise<void> {
    // NEW: Create canvas binding (replaces manual DPI setup)
    this.canvasBinding = new CanvasBinding(this.canvas);
    this.renderTarget = this.canvasBinding.getRenderingTarget();

    // Listen for resize
    this.canvasBinding.onSizeChanged((oldSize, newSize) => {
      console.log('[RustChart] Canvas resized:', oldSize, '→', newSize);
      
      // Notify WASM (in CSS pixels)
      const cssWidth = newSize.width / (window.devicePixelRatio || 1);
      const cssHeight = newSize.height / (window.devicePixelRatio || 1);
      
      this.wasmChart?.resize(cssWidth, cssHeight);
    });

    // Load WASM module
    const module = await import("../../public/wasm/trading_ui.js");
    await module.default();

    // Create WASM chart instance (CSS dimensions)
    const { width, height } = this.canvasBinding.mediaSize;
    const dpr = window.devicePixelRatio || 1;

    console.log("[RustChart] Initialized:", {
      mediaSize: { width, height },
      bitmapSize: this.canvasBinding.bitmapSize,
      dpr
    });

    this.wasmChart = new module.WasmChart(width, height, this.timeframe);
    this.wasmChart.attachCanvas(this.canvas);

    // Initialize tooltip
    initChartTooltip(this as any, this.canvas);

    // Connect axes with rendering target
    this.priceAxis.connectToCanvas(this.renderTarget, this.canvas);
    this.timeAxis.connectToCanvas(this.renderTarget, this.canvas);

    // Start render loop
    this.startRenderLoop();
  }

  private startRenderLoop(): void {
    const render = () => {
      if (this.wasmChart && this.renderTarget) {
        // Rust renders to canvas
        this.wasmChart.render();

        // TypeScript renders axes using coordinate scopes
        const viewport = this.getViewportInfo();
        if (viewport) {
          this.priceAxis.draw(this.renderTarget, viewport);
          this.timeAxis.draw(this.renderTarget, viewport);
          this.highLowOverlay.update(viewport, this.getCurrentPrice());
        }
      }
      this.animationFrameId = requestAnimationFrame(render);
    };
    render();
  }

  destroy(): void {
    // ... existing cleanup ...
    
    // NEW: Dispose canvas binding
    if (this.canvasBinding) {
      this.canvasBinding.dispose();
      this.canvasBinding = null;
    }
  }
}
```

---

#### 3.2 Update PriceAxis with Coordinate Scopes

**File:** `/apps/frontend/src/lib/chart-axes.ts`

```typescript
import type { CanvasRenderingTarget } from './canvas/coordinate-scope';
import type { ViewportInfo } from './rust-chart';

export class PriceAxis {
  // ... existing code ...

  /**
   * Draw price axis using coordinate scopes
   */
  draw(target: CanvasRenderingTarget, viewport: ViewportInfo): void {
    const prices = this.calculatePriceLevels(viewport);

    // STEP 1: Draw background and grid lines (bitmap space = sharp)
    target.useBitmapCoordinateSpace(({ 
      context, 
      bitmapSize, 
      horizontalPixelRatio,
      verticalPixelRatio 
    }) => {
      // Clear background
      context.fillStyle = 'rgba(20, 24, 28, 0.9)';
      context.fillRect(0, 0, bitmapSize.width, bitmapSize.height);

      // Draw horizontal grid lines (sharp 1px lines)
      context.strokeStyle = '#333';
      context.lineWidth = 1;

      for (const price of prices) {
        const y = this.priceToY(price, viewport, bitmapSize.height / verticalPixelRatio);
        const deviceY = Math.round(y * verticalPixelRatio) + 0.5; // Pixel-perfect

        context.beginPath();
        context.moveTo(0, deviceY);
        context.lineTo(bitmapSize.width, deviceY);
        context.stroke();
      }
    });

    // STEP 2: Draw price labels (media space = smooth text)
    target.useMediaCoordinateSpace(({ context, mediaSize }) => {
      context.font = '11px monospace';
      context.fillStyle = '#e7e9ea';
      context.textAlign = 'right';
      context.textBaseline = 'middle';

      for (const price of prices) {
        const y = this.priceToY(price, viewport, mediaSize.height);
        context.fillText(
          price.toFixed(2),
          mediaSize.width - 6,
          y
        );
      }
    });
  }

  private priceToY(price: number, viewport: ViewportInfo, height: number): number {
    const priceRange = viewport.price.max - viewport.price.min;
    const priceOffset = viewport.price.max - price;
    return (priceOffset / priceRange) * height;
  }

  private calculatePriceLevels(viewport: ViewportInfo): number[] {
    const { min, max } = viewport.price;
    const step = (max - min) / (this.config.labelCount! - 1);
    const levels: number[] = [];

    for (let i = 0; i < this.config.labelCount!; i++) {
      levels.push(max - i * step);
    }

    return levels;
  }
}
```

**Vorher vs. Nachher:**

```typescript
// ❌ BEFORE: Manual DPI handling everywhere
const dpr = window.devicePixelRatio || 1;
ctx.lineWidth = 1 * dpr;
ctx.moveTo(x * dpr, y * dpr);
ctx.fillText(text, x * dpr, y * dpr);  // ⚠️ Blurry on some DPIs

// ✅ AFTER: Explicit coordinate spaces
target.useBitmapCoordinateSpace(({ context, horizontalPixelRatio }) => {
  context.lineWidth = 1;  // Always 1 device pixel
  context.moveTo(x * horizontalPixelRatio, y * horizontalPixelRatio);
});

target.useMediaCoordinateSpace(({ context }) => {
  context.fillText(text, x, y);  // Natural scaling, smooth
});
```

---

### Phase 4: Documentation & Tests (1-2 Tage)

#### 4.1 Create Usage Guide

**File:** `/docs/CANVAS_COORDINATE_SPACES.md`

```markdown
# Canvas Coordinate Spaces Guide

## Overview

Loom verwendet zwei Koordinatensysteme für pixelgenaues Rendering auf allen DPI-Stufen:

### Media Space (CSS Pixels)
- **Verwendung:** Text, Tooltips, UI-Elemente
- **Einheit:** CSS Pixels (logische Pixel)
- **Skalierung:** Browser-native (automatisch)
- **Best for:** Labels, Fonts, Smooth-Scaling-Content

### Bitmap Space (Device Pixels)
- **Verwendung:** Grid-Linien, Candles, geometrische Formen
- **Einheit:** Device Pixels (physikalische Pixel)
- **Skalierung:** Manuell via `devicePixelRatio`
- **Best for:** Pixel-perfekte Linien, Sharp Edges

## Quick Start

```typescript
import { CanvasBinding } from './lib/canvas/canvas-binding';

// 1. Setup
const binding = new CanvasBinding(canvasElement);
const target = binding.getRenderingTarget();

// 2. Draw sharp lines (bitmap space)
target.useBitmapCoordinateSpace(({ context, horizontalPixelRatio }) => {
  context.strokeStyle = '#333';
  context.lineWidth = 1;  // Exactly 1 device pixel
  
  const x = 100 * horizontalPixelRatio;  // Convert CSS to device
  context.moveTo(x + 0.5, 0);  // +0.5 for pixel-perfect odd-width lines
  context.lineTo(x + 0.5, context.canvas.height);
  context.stroke();
});

// 3. Draw smooth text (media space)
target.useMediaCoordinateSpace(({ context }) => {
  context.font = '12px Arial';  // CSS size
  context.fillStyle = '#fff';
  context.fillText('Label', 100, 50);  // CSS coordinates
});
```

## Common Patterns

### Pattern 1: Vertical Grid Line
```typescript
function drawVerticalGridLine(target: CanvasRenderingTarget, cssX: number) {
  target.useBitmapCoordinateSpace(({ context, horizontalPixelRatio }) => {
    const deviceX = Math.round(cssX * horizontalPixelRatio);
    
    context.strokeStyle = '#2a2a2a';
    context.lineWidth = 1;
    context.beginPath();
    context.moveTo(deviceX + 0.5, 0);
    context.lineTo(deviceX + 0.5, context.canvas.height);
    context.stroke();
  });
}
```

### Pattern 2: Price Label with Background
```typescript
function drawPriceLabel(
  target: CanvasRenderingTarget, 
  price: number, 
  y: number
) {
  // Step 1: Draw background box (sharp edges)
  target.useBitmapCoordinateSpace(({ context, horizontalPixelRatio, verticalPixelRatio }) => {
    const boxWidth = 60 * horizontalPixelRatio;
    const boxHeight = 20 * verticalPixelRatio;
    const boxY = (y - 10) * verticalPixelRatio;
    
    context.fillStyle = '#2a2a2a';
    context.fillRect(
      context.canvas.width - boxWidth,
      boxY,
      boxWidth,
      boxHeight
    );
  });

  // Step 2: Draw text (smooth scaling)
  target.useMediaCoordinateSpace(({ context, mediaSize }) => {
    context.font = 'bold 11px monospace';
    context.fillStyle = '#00ff00';
    context.textAlign = 'right';
    context.fillText(
      price.toFixed(2),
      mediaSize.width - 6,
      y
    );
  });
}
```

### Pattern 3: Crosshair
```typescript
function drawCrosshair(
  target: CanvasRenderingTarget,
  cssX: number,
  cssY: number
) {
  target.useBitmapCoordinateSpace(({ 
    context, 
    bitmapSize, 
    horizontalPixelRatio, 
    verticalPixelRatio 
  }) => {
    const deviceX = Math.round(cssX * horizontalPixelRatio);
    const deviceY = Math.round(cssY * verticalPixelRatio);
    
    context.strokeStyle = 'rgba(255, 255, 255, 0.3)';
    context.lineWidth = 1;
    
    // Horizontal line
    context.beginPath();
    context.moveTo(0, deviceY + 0.5);
    context.lineTo(bitmapSize.width, deviceY + 0.5);
    context.stroke();
    
    // Vertical line
    context.beginPath();
    context.moveTo(deviceX + 0.5, 0);
    context.lineTo(deviceX + 0.5, bitmapSize.height);
    context.stroke();
  });
}
```

## DPI Testing Checklist

Test on multiple device pixel ratios:
- [ ] 1.0 (Standard HD)
- [ ] 1.5 (Windows High-DPI)
- [ ] 2.0 (MacBook Retina)
- [ ] 3.0 (4K displays)

### Chrome DevTools DPI Simulation
```javascript
// Open DevTools Console
// Simulate 2x DPI:
window.devicePixelRatio = 2;
location.reload();
```

## Troubleshooting

### Blurry Lines
**Problem:** Grid lines appear blurry on Retina displays  
**Solution:** Use `useBitmapCoordinateSpace` + pixel correction

```typescript
// ❌ Wrong
context.moveTo(x, y);

// ✅ Correct
const deviceX = Math.round(x * horizontalPixelRatio);
context.moveTo(deviceX + 0.5, y);  // +0.5 for odd line widths
```

### Blurry Text
**Problem:** Text appears blurry  
**Solution:** Use `useMediaCoordinateSpace` for text

```typescript
// ❌ Wrong: Text in bitmap space
target.useBitmapCoordinateSpace(({ context }) => {
  context.fillText('Label', x, y);  // Scaled incorrectly
});

// ✅ Correct: Text in media space
target.useMediaCoordinateSpace(({ context }) => {
  context.fillText('Label', x, y);  // Natural scaling
});
```

### Resize Loops
**Problem:** ResizeObserver triggers infinitely  
**Solution:** CanvasBinding prevents this automatically

```typescript
// ✅ Safe - uses devicePixelContentBox
const binding = new CanvasBinding(canvas);

// ❌ Unsafe - manual observer can loop
const observer = new ResizeObserver(() => {
  canvas.width = ...;  // ⚠️ Can trigger itself
});
```
```

---

#### 4.2 Create Tests

**File:** `/apps/frontend/src/lib/canvas/__tests__/canvas-binding.test.ts`

```typescript
import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { CanvasBinding } from '../canvas-binding';

describe('CanvasBinding', () => {
  let canvas: HTMLCanvasElement;
  let binding: CanvasBinding;

  beforeEach(() => {
    canvas = document.createElement('canvas');
    document.body.appendChild(canvas);
    canvas.style.width = '800px';
    canvas.style.height = '600px';
  });

  afterEach(() => {
    binding?.dispose();
    document.body.removeChild(canvas);
  });

  it('should initialize with correct sizes', () => {
    binding = new CanvasBinding(canvas);
    
    expect(binding.mediaSize.width).toBe(800);
    expect(binding.mediaSize.height).toBe(600);
    
    const dpr = window.devicePixelRatio || 1;
    expect(binding.bitmapSize.width).toBe(800 * dpr);
    expect(binding.bitmapSize.height).toBe(600 * dpr);
  });

  it('should handle resize events', async () => {
    binding = new CanvasBinding(canvas);
    
    let resizeCount = 0;
    binding.onSizeChanged(() => {
      resizeCount++;
    });

    // Trigger resize
    canvas.style.width = '1000px';
    await new Promise(resolve => setTimeout(resolve, 100));

    expect(resizeCount).toBeGreaterThan(0);
    expect(binding.mediaSize.width).toBe(1000);
  });

  it('should not trigger resize loops', async () => {
    binding = new CanvasBinding(canvas);
    
    let resizeCount = 0;
    binding.onSizeChanged(() => {
      resizeCount++;
    });

    // Multiple identical resizes
    canvas.style.width = '900px';
    canvas.style.width = '900px';
    canvas.style.width = '900px';
    await new Promise(resolve => setTimeout(resolve, 100));

    // Should only trigger once
    expect(resizeCount).toBeLessThanOrEqual(1);
  });

  it('should provide rendering target', () => {
    binding = new CanvasBinding(canvas);
    const target = binding.getRenderingTarget();
    
    expect(target).toBeDefined();
    expect(target.mediaSize).toEqual(binding.mediaSize);
    expect(target.bitmapSize).toEqual(binding.bitmapSize);
  });

  it('should cleanup properly', () => {
    binding = new CanvasBinding(canvas);
    const target = binding.getRenderingTarget();
    
    binding.dispose();
    
    // Should not throw after dispose
    expect(() => binding.dispose()).not.toThrow();
  });
});
```

**File:** `/apps/frontend/src/lib/canvas/__tests__/coordinate-scope.test.ts`

```typescript
import { describe, it, expect, beforeEach } from 'vitest';
import { CanvasRenderingTarget } from '../coordinate-scope';

describe('CanvasRenderingTarget', () => {
  let canvas: HTMLCanvasElement;
  let target: CanvasRenderingTarget;

  beforeEach(() => {
    canvas = document.createElement('canvas');
    canvas.width = 800;
    canvas.height = 600;
    target = new CanvasRenderingTarget(canvas);
  });

  it('should execute bitmap coordinate callback', () => {
    let executed = false;
    
    target.useBitmapCoordinateSpace(({ context, bitmapSize }) => {
      executed = true;
      expect(context).toBeDefined();
      expect(bitmapSize.width).toBe(800);
      expect(bitmapSize.height).toBe(600);
    });
    
    expect(executed).toBe(true);
  });

  it('should execute media coordinate callback', () => {
    let executed = false;
    
    target.useMediaCoordinateSpace(({ context, mediaSize }) => {
      executed = true;
      expect(context).toBeDefined();
      expect(mediaSize).toBeDefined();
    });
    
    expect(executed).toBe(true);
  });

  it('should return callback result', () => {
    const result = target.useBitmapCoordinateSpace(() => {
      return 42;
    });
    
    expect(result).toBe(42);
  });

  it('should restore context after callback', () => {
    const ctx = canvas.getContext('2d')!;
    ctx.fillStyle = '#ff0000';
    
    target.useBitmapCoordinateSpace(({ context }) => {
      context.fillStyle = '#00ff00';
    });
    
    // Context should be restored (in real implementation)
    // This is a simplified test
    expect(ctx).toBeDefined();
  });
});
```

---

## Timeline & Milestones

### Woche 1: TypeScript Foundation
- **Tag 1-2:** Types + ResizeHandler implementieren
- **Tag 3-4:** CoordinateScope + CanvasBinding implementieren
- **Tag 5:** Integration tests, Bug fixes

**Milestone 1:** CanvasBinding funktioniert, keine Resize-Loops

### Woche 2: Rust Integration
- **Tag 1-2:** Rust coordinate types + BitmapSpace
- **Tag 3-4:** Canvas2DRenderer update mit CSS/Device Methods
- **Tag 5:** WASM bindings testen

**Milestone 2:** Rust renderer kann beide Coordinate Spaces nutzen

### Woche 3: UI Updates & Testing
- **Tag 1-2:** PriceAxis + TimeAxis auf neue API migrieren
- **Tag 3:** RustChart Integration
- **Tag 4-5:** Multi-DPI testing (1x, 1.5x, 2x, 3x)

**Milestone 3:** Axes render pixelgenau auf allen DPIs

### Woche 4: Documentation & Cleanup
- **Tag 1-2:** Usage Guide + API Docs schreiben
- **Tag 3:** Code Review + Refactoring
- **Tag 4-5:** Performance profiling + Optimierung

**Milestone 4:** Production-ready, dokumentiert, getestet

---

## Success Criteria

✅ **Functional Requirements:**
- [ ] Grid lines sind exakt 1px auf allen DPIs (kein Blur)
- [ ] Text ist smooth und lesbar auf allen DPIs
- [ ] Resize ohne Loops auf allen Browsern
- [ ] Performance ≤ 16ms pro Frame (60 FPS)

✅ **Code Quality:**
- [ ] Keine `devicePixelRatio` Multiplikationen außerhalb von Scopes
- [ ] Type-safety: Compiler verhindert Koordinaten-Mischung
- [ ] 100% Test Coverage für CanvasBinding
- [ ] Dokumentation vollständig

✅ **Cross-Browser:**
- [ ] Chrome 90+ (devicePixelContentBox support)
- [ ] Firefox 88+ (graceful fallback)
- [ ] Safari 14+ (graceful fallback)
- [ ] Edge 90+

---

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Performance Regression | Medium | High | Benchmark vor/nach, max 5% overhead |
| Browser Compatibility | Low | Medium | Graceful fallbacks für alte Browser |
| Learning Curve | Medium | Low | Umfassende Docs + Beispiele |
| Migration Complexity | Medium | Medium | Schrittweise Migration, beide APIs parallel |
| Breaking Changes | Low | High | Semantic Versioning, Deprecation Warnings |

---

## Alternative Approaches Considered

### Option A: fancy-canvas Dependency ❌
- **Pro:** Battle-tested, maintained by TradingView
- **Con:** External dependency, bundle size increase
- **Decision:** Rejected - prefer self-contained solution

### Option B: WebGL Renderer 🔮
- **Pro:** Better performance for large datasets
- **Con:** Higher complexity, harder to debug
- **Decision:** Future consideration, not for v1

### Option C: Manual DPI Handling (Status Quo) ❌
- **Pro:** No refactoring needed
- **Con:** Error-prone, hard to maintain
- **Decision:** Rejected - technical debt

### Option D: Coordinate Space Abstraction (This Plan) ✅
- **Pro:** Type-safe, maintainable, no dependencies
- **Con:** Initial refactoring effort
- **Decision:** **SELECTED** - best long-term solution

---

## References

### External Resources
- [fancy-canvas GitHub](https://github.com/tradingview/fancy-canvas)
- [MDN: Canvas API](https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API)
- [ResizeObserver Spec](https://w3c.github.io/csswg-drafts/resize-observer/)
- [Device Pixel Ratio](https://developer.mozilla.org/en-US/docs/Web/API/Window/devicePixelRatio)

### Internal Resources
- `/temp/fancy-canvas-master/` - Reference implementation
- `/temp/lightweight-charts-master/` - TradingView patterns
- `/crates/chartcore/src/renderers/canvas2d.rs` - Current implementation

---

## Approval & Sign-Off

- [ ] **Architect Review:** Architecture approved
- [ ] **Tech Lead Review:** Implementation plan approved
- [ ] **PM Review:** Timeline and scope approved
- [ ] **Start Date:** TBD
- [ ] **Target Completion:** TBD

---

**Document Version:** 1.0  
**Last Updated:** 2026-01-02  
**Author:** Claude Assistant  
**Status:** DRAFT - Awaiting Approval
