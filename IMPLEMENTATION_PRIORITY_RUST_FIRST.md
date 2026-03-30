# Implementation Priority: Rust-First Approach
## Loom Trading Platform Feature Roadmap

**Prinzip:** Alle Features primär in Rust/WASM implementieren  
**Ökosystem:** Rust (Core) + TypeScript (UI) + WASM (Bridge)  
**Keine neuen Sprachen:** Python nur als Referenz, nicht in Production  

**Erstellt:** 2026-01-02  
**Status:** APPROVED - Ready for Implementation

---

## Architektur-Prinzipien

### Layer-Verantwortlichkeiten

```
┌─────────────────────────────────────────────────────────────┐
│  TypeScript Layer (UI Only)                                 │
│  ✅ User Interaction (Mouse, Keyboard, Touch)              │
│  ✅ DOM Manipulation (Toolbar, Modals, Tooltips)           │
│  ✅ Event Dispatching                                       │
│  ❌ NO Business Logic                                       │
│  ❌ NO Calculations                                         │
└──────────────────────┬──────────────────────────────────────┘
                       │ WASM Bindings
                       ↓
┌─────────────────────────────────────────────────────────────┐
│  Rust/WASM Core (ALL Business Logic)                       │
│  ✅ Indicators (All Calculations)                          │
│  ✅ Drawing Tools (Geometry, Rendering)                    │
│  ✅ Canvas Rendering (DPI, Coordinates)                    │
│  ✅ State Management (Undo/Redo, Serialization)           │
│  ✅ Performance-Critical Code                              │
└─────────────────────────────────────────────────────────────┘
```

### Warum Rust-First?

1. **Performance:** WASM läuft mit near-native Speed
2. **Type Safety:** Rust's Compiler verhindert Fehler zur Compile-Time
3. **Memory Safety:** Keine Segfaults, kein GC
4. **Konsistenz:** Eine Language für Core Logic
5. **WebAssembly:** Native Browser-Integration
6. **Testability:** Rust's Test-Framework ist exzellent

---

## Feature Categorization

### ✅ Category A: Pure Rust Implementation (WASM)
**Characteristics:**
- Performance-critical
- Mathematical calculations
- State management
- Rendering primitives

**Implementation:**
- Rust crate in `/crates/chartcore/`
- WASM bindings in `wasm.rs`
- TypeScript types auto-generated
- Zero JavaScript business logic

---

### 🔶 Category B: Rust Core + TypeScript UI
**Characteristics:**
- Requires DOM manipulation
- User interaction heavy
- Visual feedback needed

**Implementation:**
- Rust: Business logic, validation, state
- TypeScript: Event handling, DOM updates
- Clear separation of concerns

---

### ❌ Category C: External Reference Only
**Characteristics:**
- Requires external language (Python, etc.)
- Not critical for core functionality
- Research/analysis tools

**Implementation:**
- Keep as external reference
- May inspire Rust ports later
- NOT in production pipeline

---

## Priority Matrix

### Priority 1: Canvas DPI System (Pure Rust ✅)

**Goal:** Pixel-perfect rendering on all DPI scales

**Rust Implementation:**
```
/crates/chartcore/src/canvas/
├── mod.rs                    # Public API
├── coordinate_space.rs       # CssPixels, DevicePixels types
├── bitmap_space.rs           # Bitmap coordinate calculations
├── media_space.rs            # Media coordinate calculations
└── dpi_renderer.rs           # DPI-aware rendering primitives
```

**TypeScript Layer:** Minimal wrapper only
```typescript
// apps/frontend/src/lib/canvas/
├── canvas-binding.ts         # Thin wrapper around WASM
└── types.ts                  # TypeScript type definitions
```

**Tasks:**
1. ✅ **[RUST]** Create coordinate space types (CssPixels, DevicePixels, PixelRatio)
2. ✅ **[RUST]** Implement BitmapSpace with conversion methods
3. ✅ **[RUST]** Implement MediaSpace for text rendering
4. ✅ **[RUST]** Update Canvas2DRenderer with coordinate-aware methods
5. ✅ **[RUST]** Add pixel-perfect correction for odd line widths
6. 🔶 **[TS+RUST]** Create ResizeObserver wrapper (TS) calling WASM resize methods
7. 🔶 **[TS]** Update RustChart to use new coordinate system
8. ✅ **[RUST]** Add WASM bindings for coordinate transformations
9. ✅ **[TEST]** Multi-DPI testing (1x, 1.5x, 2x, 3x)

**Estimated Effort:** 2 weeks  
**Complexity:** Medium  
**Value:** 🔥 VERY HIGH (Fundamental quality improvement)

**Files to Create/Modify:**
- `crates/chartcore/src/canvas/` (new module)
- `crates/chartcore/src/renderers/canvas2d.rs` (update)
- `crates/chartcore/src/wasm.rs` (add bindings)
- `apps/frontend/src/lib/rust-chart.ts` (update initialization)

---

### Priority 2: Scientific Indicators (Pure Rust ✅)

**Goal:** Advanced market analysis tools

**Rust Implementation:**
```
/crates/chartcore/src/indicators/scientific/
├── mod.rs
├── entropy.rs                # Shannon Entropy
├── complexity.rs             # Lempel-Ziv Complexity
├── permutation_entropy.rs    # Permutation Entropy
├── approximate_entropy.rs    # Approximate Entropy
├── fractal.rs               # Fractal Dimension
├── lyapunov.rs              # Lyapunov Exponent (advanced)
└── recurrence.rs            # RQA (research-grade)
```

**Each indicator implements:**
```rust
pub trait Next<T> {
    type Output;
    fn next(&mut self, input: T) -> Self::Output;
}

impl Next<f64> for ShannonEntropy {
    type Output = f64;
    
    fn next(&mut self, input: f64) -> Self::Output {
        // Pure Rust calculation
        // No JavaScript
        // Return single value
    }
}
```

**Tasks - Tier 1 (Quick Wins):**
1. ✅ **[RUST]** Implement Shannon Entropy
   - Rolling window buffer
   - Histogram binning
   - Probability calculation
   - Entropy formula: H = -∑(pk * log₂(pk))
   - **Effort:** 2 days

2. ✅ **[RUST]** Implement Lempel-Ziv Complexity
   - Pattern detection algorithm
   - Binary sequence conversion
   - Complexity calculation
   - **Effort:** 2 days

3. ✅ **[RUST]** Implement Permutation Entropy
   - Ordinal pattern extraction
   - Permutation counting
   - Entropy from distribution
   - **Effort:** 2 days

**Tasks - Tier 2 (Advanced):**
4. ✅ **[RUST]** Implement Approximate Entropy
   - Embedding dimension handling
   - Distance matrix calculation
   - Correlation sum computation
   - **Effort:** 3 days

5. ✅ **[RUST]** Implement Fractal Dimension (Box-Counting)
   - Grid overlay algorithm
   - Box counting at multiple scales
   - Log-log regression
   - **Effort:** 4 days

**Tasks - Tier 3 (Research Grade):**
6. ✅ **[RUST]** Implement Lyapunov Exponent
   - Phase space reconstruction
   - Nearest neighbor search
   - Divergence tracking
   - Linear regression on log(divergence)
   - **Effort:** 6 days

7. ✅ **[RUST]** Implement RQA (Recurrence Quantification Analysis)
   - Distance matrix (optimized)
   - Recurrence matrix generation
   - RR, DET, LAM metrics
   - **Effort:** 8 days

**UI Integration:**
8. 🔶 **[TS]** Add indicators to IndicatorSelector component
9. 🔶 **[TS]** Create visualization templates for new indicators
10. 🔶 **[TS]** Add parameter configuration UI

**Estimated Effort:** 3-4 weeks total  
**Complexity:** Medium-High (mathematical rigor required)  
**Value:** ⭐⭐⭐⭐ HIGH (Unique differentiation, not in TradingView)

**Dependencies:**
- `libm` for mathematical functions (log, exp)
- `ndarray` for matrix operations (RQA only)
- Custom rolling buffer implementation

---

### Priority 3: Drawing System Consolidation (Rust Core ✅)

**Goal:** Professional drawing tools with undo/redo

**Current State:**
- ✅ Rust tool implementations exist (`tools/`)
- ✅ Rust DrawingManager exists (`drawings/`)
- ❌ WASM bindings missing
- ❌ TypeScript using old API

**Rust Implementation:**
```
/crates/chartcore/src/
├── tools/                    # Old system (3 tools)
│   ├── trendline.rs
│   ├── horizontal_line.rs
│   └── vertical_line.rs
└── drawings/                 # New system (feature-rich)
    ├── mod.rs
    ├── drawing.rs           # Drawing struct
    ├── manager.rs           # DrawingManager with undo/redo
    ├── renderer.rs          # Rendering
    └── commands.rs          # Command pattern for undo/redo
```

**Tasks - Phase 1: Consolidation:**
1. ✅ **[RUST]** Expose DrawingManager WASM bindings
   ```rust
   #[wasm_bindgen(js_name = startDrawing)]
   pub fn start_drawing(&mut self, drawing_type: &str, x: f64, y: f64) -> String
   
   #[wasm_bindgen(js_name = updateActiveDrawing)]
   pub fn update_active_drawing(&mut self, x: f64, y: f64) -> Result<(), JsValue>
   
   #[wasm_bindgen(js_name = finalizeDrawing)]
   pub fn finalize_drawing(&mut self) -> Result<String, JsValue>
   
   #[wasm_bindgen(js_name = cancelDrawing)]
   pub fn cancel_drawing(&mut self)
   ```
   **Effort:** 2 days

2. ✅ **[RUST]** Add selection/editing WASM methods
   ```rust
   #[wasm_bindgen(js_name = selectDrawing)]
   pub fn select_drawing(&mut self, id: &str)
   
   #[wasm_bindgen(js_name = hitTestDrawing)]
   pub fn hit_test_drawing(&self, x: f64, y: f64, tolerance: f64) -> Option<String>
   
   #[wasm_bindgen(js_name = moveDrawing)]
   pub fn move_drawing(&mut self, id: &str, dx: f64, dy: f64)
   
   #[wasm_bindgen(js_name = deleteDrawing)]
   pub fn delete_drawing(&mut self, id: &str)
   ```
   **Effort:** 1 day

3. ✅ **[RUST]** Add undo/redo WASM bindings
   ```rust
   #[wasm_bindgen(js_name = undoDrawing)]
   pub fn undo_drawing(&mut self) -> bool
   
   #[wasm_bindgen(js_name = redoDrawing)]
   pub fn redo_drawing(&mut self) -> bool
   
   #[wasm_bindgen(js_name = canUndo)]
   pub fn can_undo(&self) -> bool
   
   #[wasm_bindgen(js_name = canRedo)]
   pub fn can_redo(&self) -> bool
   ```
   **Effort:** 1 day

4. ✅ **[RUST]** Update ChartState to use DrawingManager
   **Effort:** 1 day

5. 🔶 **[TS]** Migrate ToolController to use new WASM API
   **Effort:** 2 days

6. 🔶 **[TS]** Test existing tools with new system
   **Effort:** 1 day

**Tasks - Phase 2: New Tools:**
7. ✅ **[RUST]** Implement Rectangle drawing tool
   ```rust
   pub struct Rectangle {
       id: String,
       top_left: ToolNode,
       bottom_right: ToolNode,
       color: Color,
       fill_color: Option<Color>,
       stroke_width: f64,
   }
   
   impl ChartTool for Rectangle {
       fn render(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport) {
           // Draw rectangle in Rust
       }
       
       fn hit_test(&self, x: f64, y: f64, viewport: &Viewport) -> bool {
           // Point-in-rectangle test
       }
   }
   ```
   **Effort:** 2 days

8. ✅ **[RUST]** Implement Fibonacci Retracement tool
   ```rust
   pub struct FibonacciRetracement {
       id: String,
       start: ToolNode,
       end: ToolNode,
       levels: Vec<(f64, Color)>,  // (0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0)
   }
   
   impl ChartTool for FibonacciRetracement {
       fn render(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport) {
           // Calculate and draw Fibonacci levels
           for (level, color) in &self.levels {
               let price = self.start.price + (self.end.price - self.start.price) * level;
               // Draw horizontal line at price with label
           }
       }
   }
   ```
   **Effort:** 3 days

9. 🔶 **[TS]** Add tools to DrawingToolbar UI
   **Effort:** 1 day

**Tasks - Phase 3: Advanced Features:**
10. ✅ **[RUST]** Implement preview/ghost rendering
    ```rust
    // In DrawingManager
    pub struct DrawingManager {
        active_drawing: Option<Drawing>,  // Drawing in progress
        preview_drawing: Option<Drawing>, // Ghost/preview
    }
    
    // Render both committed + preview drawings
    pub fn render_all(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport) {
        for drawing in &self.drawings {
            drawing.render(renderer, viewport);
        }
        
        // Render preview with transparency
        if let Some(preview) = &self.preview_drawing {
            renderer.set_global_alpha(0.5);
            preview.render(renderer, viewport);
            renderer.set_global_alpha(1.0);
        }
    }
    ```
    **Effort:** 2 days

11. ✅ **[RUST]** Add multi-selection support
    ```rust
    pub struct DrawingManager {
        selected_drawings: Vec<String>,  // Multiple IDs
    }
    
    #[wasm_bindgen(js_name = selectMultiple)]
    pub fn select_multiple(&mut self, ids: Vec<String>)
    
    #[wasm_bindgen(js_name = moveSelected)]
    pub fn move_selected(&mut self, dx: f64, dy: f64)
    
    #[wasm_bindgen(js_name = deleteSelected)]
    pub fn delete_selected(&mut self)
    ```
    **Effort:** 2 days

12. ✅ **[RUST]** Style customization in DrawingManager
    ```rust
    #[wasm_bindgen(js_name = setDrawingStyle)]
    pub fn set_drawing_style(&mut self, id: &str, style: JsValue) -> Result<(), JsValue>
    
    pub struct DrawingStyle {
        pub color: Color,
        pub stroke_width: f64,
        pub line_style: LineStyle,  // Solid, Dashed, Dotted
        pub fill_color: Option<Color>,
    }
    ```
    **Effort:** 1 day

13. 🔶 **[TS]** Right-click context menu for style editing
    **Effort:** 2 days

**Estimated Effort:** 3-4 weeks  
**Complexity:** Medium (architecture already exists)  
**Value:** ⭐⭐⭐⭐ HIGH (Professional feature set)

---

### Priority 4: TradingChart.js Features (Rust Core ✅)

**Goal:** Industry best practices

**4.1 Multi-Grid Layout (Rust Core ✅)**

**Rust Implementation:**
```rust
// crates/chartcore/src/layout/multi_grid.rs

pub struct GridLayout {
    main_grid: GridConfig,
    indicator_grids: Vec<GridConfig>,
}

pub struct GridConfig {
    height_ratio: f64,    // Percentage of total height
    y_offset: f64,        // Calculated Y position
    viewport: Viewport,   // Independent viewport per grid
}

impl GridLayout {
    pub fn calculate_layout(&mut self, total_height: u32) {
        // Weighted height distribution
        let n = self.indicator_grids.len();
        let indicator_height = (2.0 * (n as f64).sqrt() / 7.0) / (n as f64);
        let main_height = 1.0 - indicator_height * n as f64;
        
        self.main_grid.height_ratio = main_height;
        for grid in &mut self.indicator_grids {
            grid.height_ratio = indicator_height;
        }
    }
}
```

**WASM Bindings:**
```rust
#[wasm_bindgen(js_name = addIndicatorGrid)]
pub fn add_indicator_grid(&mut self, name: &str) -> String

#[wasm_bindgen(js_name = removeIndicatorGrid)]
pub fn remove_indicator_grid(&mut self, id: &str)

#[wasm_bindgen(js_name = setGridRatio)]
pub fn set_grid_ratio(&mut self, id: &str, ratio: f64)
```

**Tasks:**
1. ✅ **[RUST]** Implement GridLayout in Rust
2. ✅ **[RUST]** Multi-viewport rendering
3. ✅ **[RUST]** Independent Y-axis per grid
4. 🔶 **[TS]** Grid management UI
**Effort:** 4 days

---

**4.2 Legend System (Rust Data + TS Rendering 🔶)**

**Rust Implementation:**
```rust
pub trait Indicator {
    fn legend(&self, values: &[f64]) -> Vec<LegendEntry>;
}

pub struct LegendEntry {
    pub label: String,
    pub value: String,
    pub color: Color,
}

impl Next<f64> for EMA {
    fn next(&mut self, input: f64) -> f64 {
        // ... calculation
    }
}

impl Indicator for EMA {
    fn legend(&self, values: &[f64]) -> Vec<LegendEntry> {
        vec![
            LegendEntry {
                label: format!("EMA({})", self.period),
                value: format!("{:.2}", values[0]),
                color: self.color,
            }
        ]
    }
}
```

**WASM Bindings:**
```rust
#[wasm_bindgen(js_name = getIndicatorLegend)]
pub fn get_indicator_legend(&self, indicator_id: &str) -> String  // JSON
```

**TypeScript Rendering:**
```typescript
// Simple DOM update, no logic
function renderLegend(legendData: LegendEntry[]) {
    legendData.forEach(entry => {
        const el = document.createElement('div');
        el.style.color = entry.color;
        el.textContent = `${entry.label}: ${entry.value}`;
        legendContainer.appendChild(el);
    });
}
```

**Tasks:**
1. ✅ **[RUST]** Add Indicator trait with legend method
2. ✅ **[RUST]** Implement legend for all existing indicators
3. ✅ **[RUST]** Add WASM binding to get legend data
4. 🔶 **[TS]** Render legend in DOM
**Effort:** 3 days

---

**4.3 Logarithmic Price Scale (Pure Rust ✅)**

**Rust Implementation:**
```rust
pub enum PriceScale {
    Linear,
    Logarithmic,
}

impl Viewport {
    pub fn price_to_y_log(&self, price: f64) -> f64 {
        let log_price = price.ln();
        let log_min = self.price.min.ln();
        let log_max = self.price.max.ln();
        
        let normalized = (log_max - log_price) / (log_max - log_min);
        normalized * self.dimensions.height as f64
    }
    
    pub fn y_to_price_log(&self, y: f64) -> f64 {
        let log_min = self.price.min.ln();
        let log_max = self.price.max.ln();
        
        let normalized = y / self.dimensions.height as f64;
        let log_price = log_max - normalized * (log_max - log_min);
        log_price.exp()
    }
}
```

**Tasks:**
1. ✅ **[RUST]** Add PriceScale enum to Viewport
2. ✅ **[RUST]** Implement logarithmic transformation methods
3. ✅ **[RUST]** Update rendering to use selected scale
4. 🔶 **[TS]** Add scale toggle button
**Effort:** 2 days

---

**4.4 Tick Aggregation (Pure Rust ✅)**

**Rust Implementation:**
```rust
pub struct TickAggregator {
    buffer: VecDeque<Candle>,
    last_update: i64,
    merge_threshold_ms: i64,
}

impl TickAggregator {
    pub fn push(&mut self, candle: Candle) -> Option<Candle> {
        let time_diff = candle.time - self.last_update;
        
        if time_diff < self.merge_threshold_ms && !self.buffer.is_empty() {
            // Merge with last candle
            if let Some(last) = self.buffer.back_mut() {
                last.h = last.h.max(candle.h);
                last.l = last.l.min(candle.l);
                last.c = candle.c;
                last.v += candle.v;
                return None;
            }
        }
        
        // New candle
        self.buffer.push_back(candle);
        self.last_update = candle.time;
        Some(candle)
    }
}
```

**Tasks:**
1. ✅ **[RUST]** Implement TickAggregator
2. ✅ **[RUST]** Add to ChartState
3. ✅ **[RUST]** WASM binding for configuration
**Effort:** 2 days

---

**4.5 Range/Measurement Tool (Rust Core ✅)**

**Rust Implementation:**
```rust
pub struct MeasurementTool {
    id: String,
    start: ToolNode,
    end: ToolNode,
}

impl MeasurementTool {
    pub fn calculate_metrics(&self) -> MeasurementMetrics {
        let price_delta = self.end.price - self.start.price;
        let price_percent = (price_delta / self.start.price) * 100.0;
        let time_delta = self.end.time - self.start.time;
        let bar_count = time_delta / self.timeframe_seconds;
        
        MeasurementMetrics {
            price_delta,
            price_percent,
            time_delta,
            bar_count,
            time_formatted: format_time_span(time_delta),
        }
    }
}

impl ChartTool for MeasurementTool {
    fn render(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport) {
        // Draw box from start to end
        // Render metrics in label
        let metrics = self.calculate_metrics();
        
        renderer.draw_rect(/* ... */);
        renderer.draw_text(&format!(
            "Δ${:.2} ({:.2}%)\n{}\n{} bars",
            metrics.price_delta,
            metrics.price_percent,
            metrics.time_formatted,
            metrics.bar_count
        ), /* ... */);
    }
}
```

**Tasks:**
1. ✅ **[RUST]** Implement MeasurementTool
2. ✅ **[RUST]** Add metric calculations
3. ✅ **[RUST]** Format time spans (1D 2h 30m style)
4. 🔶 **[TS]** Add to toolbar
**Effort:** 3 days

---

**4.6 Pin-Based Draggable Points (Rust + TS 🔶)**

**Rust Implementation:**
```rust
pub struct ToolNode {
    pub time: i64,
    pub price: f64,
    pub node_type: NodeType,  // Start, End, Control
}

pub trait ChartTool {
    fn nodes(&self) -> &[ToolNode];
    fn nodes_mut(&mut self) -> &mut [ToolNode];
    
    fn hit_test_node(&self, x: f64, y: f64, viewport: &Viewport) -> Option<usize> {
        const RADIUS: f64 = 5.0;
        
        for (i, node) in self.nodes().iter().enumerate() {
            let node_x = viewport.time_to_x(node.time);
            let node_y = viewport.price_to_y(node.price);
            
            let dx = x - node_x;
            let dy = y - node_y;
            
            if (dx * dx + dy * dy).sqrt() < RADIUS {
                return Some(i);
            }
        }
        
        None
    }
}
```

**WASM Bindings:**
```rust
#[wasm_bindgen(js_name = startDragNode)]
pub fn start_drag_node(&mut self, drawing_id: &str, x: f64, y: f64) -> Option<usize>

#[wasm_bindgen(js_name = dragNodeTo)]
pub fn drag_node_to(&mut self, drawing_id: &str, node_idx: usize, x: f64, y: f64)

#[wasm_bindgen(js_name = endDragNode)]
pub fn end_drag_node(&mut self)
```

**TypeScript (Events Only):**
```typescript
canvas.addEventListener('mousedown', (e) => {
    const nodeIdx = wasmChart.startDragNode(drawingId, e.x, e.y);
    if (nodeIdx !== null) {
        isDragging = true;
        draggedNode = nodeIdx;
    }
});

canvas.addEventListener('mousemove', (e) => {
    if (isDragging) {
        wasmChart.dragNodeTo(drawingId, draggedNode, e.x, e.y);
    }
});
```

**Tasks:**
1. ✅ **[RUST]** Implement node hit testing
2. ✅ **[RUST]** Add drag state to DrawingManager
3. ✅ **[RUST]** WASM bindings for drag operations
4. 🔶 **[TS]** Mouse event handlers (delegation only)
**Effort:** 3 days

---

**4.7 Touch Gestures (TS Event → Rust Logic 🔶)**

**TypeScript (Event Capture):**
```typescript
// Using native Touch API (no HammerJS dependency)
let touchStart: { x: number, y: number, distance: number } | null = null;

canvas.addEventListener('touchstart', (e) => {
    if (e.touches.length === 1) {
        touchStart = { x: e.touches[0].clientX, y: e.touches[0].clientY };
        wasmChart.startPan(touchStart.x, touchStart.y);
    } else if (e.touches.length === 2) {
        const distance = getTouchDistance(e.touches);
        touchStart = { distance };
        wasmChart.startPinch(distance);
    }
});

canvas.addEventListener('touchmove', (e) => {
    if (e.touches.length === 1 && touchStart) {
        wasmChart.updatePan(e.touches[0].clientX, e.touches[0].clientY);
    } else if (e.touches.length === 2) {
        const newDistance = getTouchDistance(e.touches);
        wasmChart.updatePinch(newDistance);
    }
});
```

**Rust Implementation:**
```rust
pub enum TouchGesture {
    None,
    Pan { start_x: f64, start_y: f64 },
    Pinch { start_distance: f64 },
}

impl ChartState {
    pub fn start_pan(&mut self, x: f64, y: f64) {
        self.gesture = TouchGesture::Pan { start_x: x, start_y: y };
    }
    
    pub fn update_pan(&mut self, x: f64, y: f64) {
        if let TouchGesture::Pan { start_x, start_y } = self.gesture {
            let dx = x - start_x;
            let dy = y - start_y;
            self.viewport.pan(dx as i32, dy as i32);
        }
    }
    
    pub fn start_pinch(&mut self, distance: f64) {
        self.gesture = TouchGesture::Pinch { start_distance: distance };
    }
    
    pub fn update_pinch(&mut self, distance: f64) {
        if let TouchGesture::Pinch { start_distance } = self.gesture {
            let scale = distance / start_distance;
            self.viewport.zoom_by_factor(scale);
        }
    }
}
```

**Tasks:**
1. ✅ **[RUST]** Implement TouchGesture enum and handlers
2. ✅ **[RUST]** WASM bindings for gesture methods
3. 🔶 **[TS]** Native touch event listeners (no dependency)
4. ✅ **[RUST]** Momentum calculation for fling
**Effort:** 3 days

---

**4.8 Tool Modifiers (Pure Rust ✅)**

**Rust Implementation:**
```rust
pub struct TrendLine {
    id: String,
    start: ToolNode,
    end: ToolNode,
    extended: bool,      // Modifier
    ray: bool,          // Modifier
}

impl ChartTool for TrendLine {
    fn render(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport) {
        let start_x = viewport.time_to_x(self.start.time);
        let start_y = viewport.price_to_y(self.start.price);
        let end_x = viewport.time_to_x(self.end.time);
        let end_y = viewport.price_to_y(self.end.price);
        
        if self.extended {
            // Extend line to viewport edges
            let (x1, y1, x2, y2) = extend_line_to_edges(
                start_x, start_y, end_x, end_y, viewport
            );
            renderer.draw_line(x1, y1, x2, y2, self.color, 1.0);
        } else if self.ray {
            // Extend only in one direction
            let (x2, y2) = extend_ray(start_x, start_y, end_x, end_y, viewport);
            renderer.draw_line(start_x, start_y, x2, y2, self.color, 1.0);
        } else {
            // Normal segment
            renderer.draw_line(start_x, start_y, end_x, end_y, self.color, 1.0);
        }
    }
}
```

**WASM Bindings:**
```rust
#[wasm_bindgen(js_name = setToolModifier)]
pub fn set_tool_modifier(&mut self, tool_id: &str, modifier: &str, enabled: bool)
```

**Tasks:**
1. ✅ **[RUST]** Add modifier fields to tools
2. ✅ **[RUST]** Implement rendering variations
3. ✅ **[RUST]** WASM binding for modifier toggle
4. 🔶 **[TS]** Toolbar modifier buttons
**Effort:** 2 days

---

**4.9 Channel Tool (Pure Rust ✅)**

**Rust Implementation:**
```rust
pub struct Channel {
    id: String,
    start: ToolNode,
    end: ToolNode,
    width: f64,  // Channel width in price units
    fill_color: Option<Color>,
}

impl ChartTool for Channel {
    fn render(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport) {
        // Calculate parallel lines
        let angle = calculate_angle(self.start, self.end);
        let perpendicular = angle + std::f64::consts::PI / 2.0;
        
        let offset_x = (self.width * perpendicular.cos());
        let offset_y = (self.width * perpendicular.sin());
        
        // Draw upper line
        renderer.draw_line(
            viewport.time_to_x(self.start.time) + offset_x,
            viewport.price_to_y(self.start.price) + offset_y,
            viewport.time_to_x(self.end.time) + offset_x,
            viewport.price_to_y(self.end.price) + offset_y,
            self.color,
            1.0
        );
        
        // Draw lower line
        renderer.draw_line(
            viewport.time_to_x(self.start.time) - offset_x,
            viewport.price_to_y(self.start.price) - offset_y,
            viewport.time_to_x(self.end.time) - offset_x,
            viewport.price_to_y(self.end.price) - offset_y,
            self.color,
            1.0
        );
        
        // Optional fill
        if let Some(fill) = self.fill_color {
            renderer.fill_polygon(/* calculate polygon points */, fill);
        }
    }
}
```

**Tasks:**
1. ✅ **[RUST]** Implement Channel tool
2. ✅ **[RUST]** Parallel line calculation
3. ✅ **[RUST]** Optional fill rendering
4. 🔶 **[TS]** Add to toolbar
**Effort:** 3 days

---

**4.10 Overlay/Indicator Plugin System (Pure Rust ✅)**

**Rust Implementation:**
```rust
// Plugin trait
pub trait IndicatorPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn calculate(&mut self, candles: &[Candle]) -> Vec<f64>;
    fn render(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport, values: &[f64]);
    fn legend(&self, current_value: f64) -> Vec<LegendEntry>;
}

// Plugin registry
pub struct IndicatorRegistry {
    plugins: HashMap<String, Box<dyn IndicatorPlugin>>,
}

impl IndicatorRegistry {
    pub fn register(&mut self, plugin: Box<dyn IndicatorPlugin>) {
        self.plugins.insert(plugin.name().to_string(), plugin);
    }
    
    pub fn get(&self, name: &str) -> Option<&Box<dyn IndicatorPlugin>> {
        self.plugins.get(name)
    }
}

// Example plugin
pub struct CustomMACD {
    fast: usize,
    slow: usize,
    signal: usize,
}

impl IndicatorPlugin for CustomMACD {
    fn name(&self) -> &str { "CustomMACD" }
    
    fn calculate(&mut self, candles: &[Candle]) -> Vec<f64> {
        // Custom MACD calculation
        vec![]
    }
    
    fn render(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport, values: &[f64]) {
        // Custom rendering
    }
    
    fn legend(&self, current_value: f64) -> Vec<LegendEntry> {
        vec![LegendEntry {
            label: format!("MACD({},{},{})", self.fast, self.slow, self.signal),
            value: format!("{:.2}", current_value),
            color: Color::rgb(255, 0, 0),
        }]
    }
}
```

**WASM Bindings:**
```rust
#[wasm_bindgen(js_name = registerIndicatorPlugin)]
pub fn register_indicator_plugin(&mut self, config: JsValue) -> Result<(), JsValue>
```

**Tasks:**
1. ✅ **[RUST]** Define IndicatorPlugin trait
2. ✅ **[RUST]** Implement IndicatorRegistry
3. ✅ **[RUST]** WASM binding for plugin registration
4. ✅ **[RUST]** Example custom indicators
5. 🔶 **[TS]** Plugin configuration UI
**Effort:** 5 days

---

## Summary: Rust vs TypeScript Split

### Pure Rust (WASM) - 85% of Code ✅

**All Business Logic:**
- Scientific Indicators (100% Rust)
- Drawing Tools (100% Rust)
- Canvas DPI Management (95% Rust)
- Grid Layout (100% Rust)
- Tick Aggregation (100% Rust)
- Tool Modifiers (100% Rust)
- Plugin System (100% Rust)

**Performance-Critical:**
- All calculations
- All rendering
- All state management
- All coordinate transformations

---

### TypeScript - 15% of Code 🔶

**UI Layer Only:**
- Event listeners (mouse, touch, keyboard)
- DOM manipulation (modals, tooltips, toolbars)
- WebSocket client
- Chart initialization wrapper
- CSS styling

**No Business Logic:**
- ❌ NO calculations
- ❌ NO state management
- ❌ NO rendering logic
- ❌ NO coordinate math

---

## Implementation Phases

### Phase 1: Foundation (Weeks 1-2)
✅ Canvas DPI System (Pure Rust)
- 2 weeks
- High impact, medium complexity
- Unlocks pixel-perfect rendering

### Phase 2: Differentiation (Weeks 3-5)
✅ Scientific Indicators (Pure Rust)
- Tier 1: Shannon, Lempel-Ziv, Permutation (Week 3)
- Tier 2: ApEn, Fractal Dimension (Week 4-5)
- Unique features, not in competitors

### Phase 3: Professional Tools (Weeks 6-8)
✅ Drawing System Consolidation (Rust Core)
- Phase 1: WASM Bindings (Week 6)
- Phase 2: New Tools (Week 7)
- Phase 3: Advanced Features (Week 8)

### Phase 4: Best Practices (Weeks 9-12)
✅ TradingChart.js Features (Rust Core)
- Multi-Grid, Legend, Log Scale (Week 9)
- Measurement Tool, Pin System (Week 10)
- Touch Gestures, Modifiers (Week 11)
- Plugin System (Week 12)

---

## Success Metrics

**Code Quality:**
- [ ] ≥95% Rust code coverage
- [ ] ≤5% TypeScript business logic
- [ ] Zero Python in production
- [ ] All tests in Rust

**Performance:**
- [ ] 60 FPS on all features
- [ ] ≤1ms per indicator calculation
- [ ] WASM binary ≤2MB gzipped

**Functionality:**
- [ ] All 40 tasks completed
- [ ] Feature parity with competitors
- [ ] Unique scientific indicators

---

**Document Version:** 1.0  
**Last Updated:** 2026-01-02  
**Author:** Claude Assistant  
**Status:** APPROVED - Ready for Implementation
