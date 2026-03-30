# Phase 1: Fundament Stabilisieren (Weeks 1-2)

**Goal:** Clean architecture foundation, single source of truth

## Week 1: Chart Engine Consolidation

### Task 1.1: Audit Both Engines

**Objective:** Document all features in both chart engines to make informed consolidation decision.

**Actions:**

1. **Audit chartcore (Rust)**
   - Location: `crates/chartcore/`
   - Document features:
     - Panel system (✓ implemented)
     - Rendering pipeline (Canvas2D via web_sys)
     - Indicator support (70+ indicators)
     - Data structures (Candle, OHLCV)
   - List strengths: Performance, memory safety, native speed
   - List weaknesses: WASM overhead, limited browser APIs

2. **Audit chart-core (TypeScript)**
   - Location: `packages/chart-core/`
   - Document features:
     - Series management
     - Renderer abstraction (Canvas/WebGPU)
     - Color utilities
     - Browser integration
   - List strengths: Easy debugging, full browser API access
   - List weaknesses: Performance ceiling, memory management

3. **Create Feature Comparison Matrix**
   
   Create file: `CHART_ENGINE_COMPARISON.md`
   
   | Feature | chartcore (Rust) | chart-core (TypeScript) |
   |---------|-----------------|-------------------------|
   | Panel System | ✓ Full | ✗ None |
   | 70+ Indicators | ✓ Yes | ✗ None |
   | Canvas2D | ✓ Direct | ✓ Abstracted |
   | WebGPU | ✗ No | ✓ Partial |
   | Performance | ⚡ Excellent | ⚠️ Good |

**Deliverable:** CHART_ENGINE_COMPARISON.md with complete feature matrix

---

### Task 1.2: Decision & Migration Plan

**Objective:** Choose primary chart engine and plan migration.

**Recommendation: chartcore (Rust) as primary**

**Rationale:**
- 70+ indicators already implemented
- Panel system already working
- Performance critical for trading platform
- TypeScript chart-core has minimal features

**Migration Plan:**

1. **Keep from chartcore:**
   - All indicator implementations
   - Panel system (panels/mod.rs, panel.rs, manager.rs, scale.rs)
   - AppState management
   - Rendering pipeline

2. **Port from chart-core to chartcore:**
   - WebGPU renderer (future work, not in 12-week scope)
   - Color utilities → `crates/chartcore/src/utils/color.rs`
   - Adaptive FPS system → `crates/chartcore/src/core/adaptive_fps.rs`

3. **Delete from chart-core:**
   - Duplicate rendering code
   - Unused series management
   - Duplicate data structures

**Action Items:**
- [ ] Move useful utilities from chart-core to chartcore
- [ ] Update all imports to use chartcore only
- [ ] Delete packages/chart-core after migration
- [ ] Update documentation

**Deliverable:** Migration checklist document

---

### Task 1.3: State Consolidation

**Objective:** Make chartcore AppState the single source of truth.

**Current Problem:**
- State duplication between crates
- Multiple modules managing same data
- Unclear ownership

**Solution:**

**File: `crates/chartcore/src/state.rs`**

```rust
use crate::panels::PanelManager;
use crate::drawings::DrawingManager;
use crate::data::CandleStore;

/// Single source of truth for all chart state
pub struct AppState {
    // Data
    pub candle_store: CandleStore,
    
    // Layout
    pub panel_manager: PanelManager,
    
    // Drawings
    pub drawing_manager: DrawingManager,
    
    // Viewport
    pub viewport: Viewport,
    
    // Settings
    pub theme: Theme,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            candle_store: CandleStore::new(),
            panel_manager: PanelManager::new(),
            drawing_manager: DrawingManager::new(),
            viewport: Viewport::default(),
            theme: Theme::dark(),
        }
    }
    
    // Data operations
    pub fn load_candles(&mut self, candles: Vec<Candle>) {
        self.candle_store.load(candles);
        self.invalidate_all();
    }
    
    pub fn update_candle(&mut self, candle: Candle) {
        self.candle_store.update(candle);
        self.invalidate_light();
    }
    
    // Panel operations
    pub fn add_panel(&mut self, config: PanelConfig) -> Result<PanelId, String> {
        self.panel_manager.add_panel(config)
    }
    
    pub fn remove_panel(&mut self, id: PanelId) -> Result<(), String> {
        self.panel_manager.remove_panel(id)
    }
    
    // Invalidation
    fn invalidate_all(&self) {
        // Trigger full re-render
    }
    
    fn invalidate_light(&self) {
        // Trigger light update (new candle only)
    }
}
```

**Update WASM bindings:**

**File: `packages/wasm-core/src/lib.rs`**

```rust
use chartcore::AppState;

#[wasm_bindgen]
pub struct WasmChart {
    state: AppState,
}

#[wasm_bindgen]
impl WasmChart {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            state: AppState::new(),
        }
    }
    
    pub fn load_candles(&mut self, json: &str) -> Result<(), JsValue> {
        let candles: Vec<Candle> = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.state.load_candles(candles);
        Ok(())
    }
    
    pub fn add_panel(&mut self, config_json: &str) -> Result<String, JsValue> {
        let config: PanelConfig = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let id = self.state.add_panel(config)
            .map_err(|e| JsValue::from_str(&e))?;
        Ok(id.to_string())
    }
}
```

**Remove duplicate state from:**
- `packages/wasm-core/src/state.rs` (merge into chartcore)
- Any other state management modules

**Deliverable:** Consolidated AppState with single source of truth

---

## Week 2: Render Command Pattern

### Task 2.1: Define Render Commands

**Objective:** Create data-driven rendering commands (preparation for Worker migration).

**File: `crates/chartcore/src/rendering/commands.rs`**

```rust
use serde::{Serialize, Deserialize};
use crate::utils::Color;

/// All possible drawing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderCommand {
    /// Draw a line from (x1,y1) to (x2,y2)
    DrawLine {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: Color,
        width: f64,
    },
    
    /// Draw a rectangle
    DrawRect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill: Option<Color>,
        stroke: Option<Color>,
        stroke_width: f64,
    },
    
    /// Draw text
    DrawText {
        text: String,
        x: f64,
        y: f64,
        font: String,
        color: Color,
        align: TextAlign,
    },
    
    /// Draw a candlestick
    DrawCandle {
        x: f64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        body_width: f64,
        wick_width: f64,
        bullish_color: Color,
        bearish_color: Color,
    },
    
    /// Draw indicator line (optimized for many points)
    DrawIndicatorLine {
        points: Vec<(f64, f64)>,
        color: Color,
        width: f64,
        style: LineStyle,
    },
    
    /// Clear the canvas
    Clear {
        color: Color,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineStyle {
    Solid,
    Dashed { dash_length: f64, gap_length: f64 },
    Dotted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Buffer of render commands for a single frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderCommandBuffer {
    pub commands: Vec<RenderCommand>,
    pub frame_id: u64,
    pub timestamp: f64,
}

impl RenderCommandBuffer {
    pub fn new(frame_id: u64) -> Self {
        Self {
            commands: Vec::with_capacity(1000),
            frame_id,
            timestamp: js_sys::Date::now(),
        }
    }
    
    pub fn push(&mut self, cmd: RenderCommand) {
        self.commands.push(cmd);
    }
    
    pub fn clear(&mut self) {
        self.commands.clear();
    }
    
    pub fn len(&self) -> usize {
        self.commands.len()
    }
}
```

**Benefits:**
- Commands are pure data (can be serialized)
- Testable without browser (commands are just Vec)
- Worker-ready (send commands via postMessage)
- Replayable (save/restore render frames)

**Deliverable:** RenderCommand types with complete coverage of drawing operations

---

### Task 2.2: Update Canvas2D Renderer

**Objective:** Refactor Canvas2D renderer to execute commands instead of direct drawing.

**File: `crates/chartcore/src/rendering/canvas2d.rs`**

```rust
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use crate::rendering::commands::*;
use crate::utils::Color;

pub struct Canvas2DRenderer {
    ctx: CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
    pixel_ratio: f64,
}

impl Canvas2DRenderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, String> {
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| "Failed to get 2d context")?
            .ok_or("2d context is None")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "Failed to cast to CanvasRenderingContext2d")?;
            
        let pixel_ratio = web_sys::window()
            .and_then(|w| w.device_pixel_ratio())
            .unwrap_or(1.0);
            
        Ok(Self { ctx, canvas, pixel_ratio })
    }
    
    /// Execute a buffer of render commands
    pub fn execute_commands(&mut self, buffer: &RenderCommandBuffer) {
        for cmd in &buffer.commands {
            self.execute_command(cmd);
        }
    }
    
    fn execute_command(&mut self, cmd: &RenderCommand) {
        match cmd {
            RenderCommand::Clear { color } => {
                self.ctx.set_fill_style(&color.to_js_value());
                self.ctx.fill_rect(
                    0.0, 
                    0.0, 
                    self.canvas.width() as f64, 
                    self.canvas.height() as f64
                );
            },
            
            RenderCommand::DrawLine { x1, y1, x2, y2, color, width } => {
                self.ctx.set_stroke_style(&color.to_js_value());
                self.ctx.set_line_width(*width * self.pixel_ratio);
                self.ctx.begin_path();
                self.ctx.move_to(*x1 * self.pixel_ratio, *y1 * self.pixel_ratio);
                self.ctx.line_to(*x2 * self.pixel_ratio, *y2 * self.pixel_ratio);
                self.ctx.stroke();
            },
            
            RenderCommand::DrawRect { x, y, width, height, fill, stroke, stroke_width } => {
                let px = x * self.pixel_ratio;
                let py = y * self.pixel_ratio;
                let pw = width * self.pixel_ratio;
                let ph = height * self.pixel_ratio;
                
                if let Some(fill_color) = fill {
                    self.ctx.set_fill_style(&fill_color.to_js_value());
                    self.ctx.fill_rect(px, py, pw, ph);
                }
                
                if let Some(stroke_color) = stroke {
                    self.ctx.set_stroke_style(&stroke_color.to_js_value());
                    self.ctx.set_line_width(*stroke_width * self.pixel_ratio);
                    self.ctx.stroke_rect(px, py, pw, ph);
                }
            },
            
            RenderCommand::DrawText { text, x, y, font, color, align } => {
                self.ctx.set_font(font);
                self.ctx.set_fill_style(&color.to_js_value());
                self.ctx.set_text_align(match align {
                    TextAlign::Left => "left",
                    TextAlign::Center => "center",
                    TextAlign::Right => "right",
                });
                self.ctx.fill_text(
                    text, 
                    *x * self.pixel_ratio, 
                    *y * self.pixel_ratio
                ).ok();
            },
            
            RenderCommand::DrawCandle { x, open, high, low, close, body_width, wick_width, bullish_color, bearish_color } => {
                let is_bullish = close >= open;
                let color = if is_bullish { bullish_color } else { bearish_color };
                
                // Draw wick
                self.execute_command(&RenderCommand::DrawLine {
                    x1: *x,
                    y1: *high,
                    x2: *x,
                    y2: *low,
                    color: color.clone(),
                    width: *wick_width,
                });
                
                // Draw body
                let body_top = open.max(*close);
                let body_bottom = open.min(*close);
                let body_height = (body_top - body_bottom).abs();
                
                self.execute_command(&RenderCommand::DrawRect {
                    x: *x - body_width / 2.0,
                    y: body_top,
                    width: *body_width,
                    height: body_height,
                    fill: Some(color.clone()),
                    stroke: None,
                    stroke_width: 0.0,
                });
            },
            
            RenderCommand::DrawIndicatorLine { points, color, width, style } => {
                if points.is_empty() {
                    return;
                }
                
                self.ctx.set_stroke_style(&color.to_js_value());
                self.ctx.set_line_width(*width * self.pixel_ratio);
                
                // Set line style
                match style {
                    LineStyle::Solid => {
                        self.ctx.set_line_dash(&js_sys::Array::new()).ok();
                    },
                    LineStyle::Dashed { dash_length, gap_length } => {
                        let dash_array = js_sys::Array::new();
                        dash_array.push(&(*dash_length * self.pixel_ratio).into());
                        dash_array.push(&(*gap_length * self.pixel_ratio).into());
                        self.ctx.set_line_dash(&dash_array).ok();
                    },
                    LineStyle::Dotted => {
                        let dash_array = js_sys::Array::new();
                        dash_array.push(&(2.0 * self.pixel_ratio).into());
                        dash_array.push(&(2.0 * self.pixel_ratio).into());
                        self.ctx.set_line_dash(&dash_array).ok();
                    },
                }
                
                self.ctx.begin_path();
                let first = &points[0];
                self.ctx.move_to(first.0 * self.pixel_ratio, first.1 * self.pixel_ratio);
                
                for point in &points[1..] {
                    self.ctx.line_to(point.0 * self.pixel_ratio, point.1 * self.pixel_ratio);
                }
                
                self.ctx.stroke();
                
                // Reset line dash
                self.ctx.set_line_dash(&js_sys::Array::new()).ok();
            },
        }
    }
}
```

**Deliverable:** Canvas2D renderer executing commands

---

### Task 2.3: Update Chart Rendering Pipeline

**Objective:** Refactor ChartCore to generate commands instead of direct rendering.

**File: `crates/chartcore/src/lib.rs`**

```rust
use crate::rendering::commands::{RenderCommand, RenderCommandBuffer};

impl ChartCore {
    /// Generate render commands for the current frame
    pub fn render(&mut self) -> RenderCommandBuffer {
        let mut buffer = RenderCommandBuffer::new(self.frame_id);
        self.frame_id += 1;
        
        // Clear canvas
        buffer.push(RenderCommand::Clear {
            color: self.state.theme.background,
        });
        
        // Render grid
        self.render_grid(&mut buffer);
        
        // Render candles
        self.render_candles(&mut buffer);
        
        // Render indicators
        self.render_indicators(&mut buffer);
        
        // Render drawings
        self.render_drawings(&mut buffer);
        
        // Render axes
        self.render_axes(&mut buffer);
        
        // Render crosshair
        if let Some(cursor) = self.cursor_pos {
            self.render_crosshair(&mut buffer, cursor);
        }
        
        buffer
    }
    
    fn render_candles(&self, buffer: &mut RenderCommandBuffer) {
        let visible_range = self.get_visible_range();
        
        for (i, candle) in self.state.candle_store.iter_range(visible_range) {
            let x = self.index_to_x(i);
            
            buffer.push(RenderCommand::DrawCandle {
                x,
                open: candle.open,
                high: candle.high,
                low: candle.low,
                close: candle.close,
                body_width: self.bar_width * 0.8,
                wick_width: 1.0,
                bullish_color: self.state.theme.bullish,
                bearish_color: self.state.theme.bearish,
            });
        }
    }
    
    fn render_grid(&self, buffer: &mut RenderCommandBuffer) {
        // Horizontal grid lines
        for price in self.get_price_grid_levels() {
            let y = self.price_to_y(price);
            
            buffer.push(RenderCommand::DrawLine {
                x1: 0.0,
                y1: y,
                x2: self.width,
                y2: y,
                color: self.state.theme.grid_line,
                width: 1.0,
            });
        }
    }
    
    // ... more rendering methods
}
```

**Update WASM API:**

**File: `packages/wasm-core/src/lib.rs`**

```rust
#[wasm_bindgen]
impl WasmChart {
    pub fn render(&mut self) -> JsValue {
        let buffer = self.chart.render();
        
        // Serialize to JSON for JavaScript
        serde_wasm_bindgen::to_value(&buffer).unwrap()
    }
}
```

**Update frontend to execute commands:**

**File: `apps/frontend/src/lib/rust-chart.ts`**

```typescript
private renderFrame(): void {
    if (!this.wasmChart) return;
    
    // Get render commands from Rust
    const buffer = this.wasmChart.render();
    
    // Execute commands (currently in main thread, future: send to worker)
    this.executeCommands(buffer);
}

private executeCommands(buffer: RenderCommandBuffer): void {
    const ctx = this.canvas.getContext('2d');
    if (!ctx) return;
    
    for (const cmd of buffer.commands) {
        switch (cmd.type) {
            case 'DrawLine':
                ctx.strokeStyle = cmd.color;
                ctx.lineWidth = cmd.width;
                ctx.beginPath();
                ctx.moveTo(cmd.x1, cmd.y1);
                ctx.lineTo(cmd.x2, cmd.y2);
                ctx.stroke();
                break;
            // ... other command types
        }
    }
}
```

**Deliverable:** Complete render command pipeline from Rust to Canvas

---

## Phase 1 Completion Checklist

- [ ] CHART_ENGINE_COMPARISON.md created with feature matrix
- [ ] Migration plan documented
- [ ] Consolidated AppState (single source of truth)
- [ ] Duplicate state removed from packages/wasm-core
- [ ] RenderCommand enum with all drawing operations
- [ ] RenderCommandBuffer implementation
- [ ] Canvas2D renderer executes commands
- [ ] ChartCore generates commands (not direct draws)
- [ ] WASM bindings updated
- [ ] Frontend executes render commands
- [ ] All tests passing
- [ ] No console errors

## Success Criteria

At the end of Phase 1:
1. Only one chart engine (chartcore)
2. Clean state management (single AppState)
3. Render commands flowing: Rust → WASM → JS → Canvas
4. Foundation ready for Phase 2 (indicator integration)

**Time Budget:** 2 weeks  
**Risk Level:** Medium (architectural changes)  
**Dependencies:** None (can start immediately)
