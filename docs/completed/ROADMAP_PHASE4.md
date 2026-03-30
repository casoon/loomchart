# Phase 4: Drawing Tools (Weeks 7-8)

**Goal:** Basic drawing tools for chart analysis

**Prerequisites:** Phase 3 complete (panel system working smoothly)

## Week 7: Drawing Infrastructure

### Task 7.1: Drawing Data Model

**Objective:** Define data structures for all drawing types.

**File: `crates/chartcore/src/drawings/mod.rs`**

```rust
pub mod drawing;
pub mod manager;
pub mod hit_test;
pub mod renderer;

pub use drawing::*;
pub use manager::DrawingManager;

use serde::{Serialize, Deserialize};
use crate::utils::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DrawingType {
    TrendLine,
    HorizontalLine,
    VerticalLine,
    Rectangle,
    FibonacciRetracement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub timestamp: i64,  // Logical X (candle timestamp)
    pub price: f64,      // Logical Y (price value)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drawing {
    pub id: String,
    pub drawing_type: DrawingType,
    pub points: Vec<Point>,
    pub style: DrawingStyle,
    pub locked: bool,
    pub visible: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingStyle {
    pub color: Color,
    pub width: f64,
    pub line_style: LineStyle,
    pub extend_left: bool,
    pub extend_right: bool,
    pub fill_color: Option<Color>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
}

impl Drawing {
    pub fn new(drawing_type: DrawingType, points: Vec<Point>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            drawing_type,
            points,
            style: DrawingStyle::default(),
            locked: false,
            visible: true,
            created_at: js_sys::Date::now() as i64,
            updated_at: js_sys::Date::now() as i64,
        }
    }
    
    pub fn update_points(&mut self, points: Vec<Point>) {
        self.points = points;
        self.updated_at = js_sys::Date::now() as i64;
    }
    
    pub fn update_style(&mut self, style: DrawingStyle) {
        self.style = style;
        self.updated_at = js_sys::Date::now() as i64;
    }
}

impl Default for DrawingStyle {
    fn default() -> Self {
        Self {
            color: Color::rgb(33, 150, 243),  // Blue
            width: 2.0,
            line_style: LineStyle::Solid,
            extend_left: false,
            extend_right: false,
            fill_color: None,
        }
    }
}
```

**Deliverable:** Drawing data model defined

---

### Task 7.2: Drawing Manager

**Objective:** Manage all drawings (add, remove, update, hit test).

**File: `crates/chartcore/src/drawings/manager.rs`**

```rust
use std::collections::HashMap;
use super::{Drawing, DrawingType, Point};

pub struct DrawingManager {
    drawings: HashMap<String, Drawing>,
    active_drawing: Option<String>,
    selected_drawing: Option<String>,
    hover_drawing: Option<String>,
}

impl DrawingManager {
    pub fn new() -> Self {
        Self {
            drawings: HashMap::new(),
            active_drawing: None,
            selected_drawing: None,
            hover_drawing: None,
        }
    }
    
    /// Start creating a new drawing
    pub fn start_drawing(&mut self, drawing_type: DrawingType, start_point: Point) -> String {
        let drawing = Drawing::new(drawing_type, vec![start_point]);
        let id = drawing.id.clone();
        
        self.drawings.insert(id.clone(), drawing);
        self.active_drawing = Some(id.clone());
        
        id
    }
    
    /// Update the active drawing with a new point
    pub fn update_active_drawing(&mut self, point: Point) -> Result<(), String> {
        let id = self.active_drawing.as_ref()
            .ok_or("No active drawing")?;
        
        let drawing = self.drawings.get_mut(id)
            .ok_or("Active drawing not found")?;
        
        // For most drawings, we only need 2 points
        match drawing.drawing_type {
            DrawingType::TrendLine | 
            DrawingType::Rectangle => {
                if drawing.points.len() == 1 {
                    drawing.points.push(point);
                } else {
                    drawing.points[1] = point;
                }
            },
            DrawingType::HorizontalLine |
            DrawingType::VerticalLine => {
                if drawing.points.len() == 1 {
                    drawing.points.push(point);
                } else {
                    drawing.points[1] = point;
                }
            },
            DrawingType::FibonacciRetracement => {
                if drawing.points.len() == 1 {
                    drawing.points.push(point);
                } else {
                    drawing.points[1] = point;
                }
            },
        }
        
        Ok(())
    }
    
    /// Finalize the active drawing
    pub fn finalize_drawing(&mut self) -> Result<String, String> {
        let id = self.active_drawing.take()
            .ok_or("No active drawing")?;
        
        // Validate drawing has enough points
        let drawing = self.drawings.get(&id)
            .ok_or("Drawing not found")?;
        
        let min_points = match drawing.drawing_type {
            DrawingType::TrendLine => 2,
            DrawingType::HorizontalLine => 1,
            DrawingType::VerticalLine => 1,
            DrawingType::Rectangle => 2,
            DrawingType::FibonacciRetracement => 2,
        };
        
        if drawing.points.len() < min_points {
            self.drawings.remove(&id);
            return Err("Not enough points".to_string());
        }
        
        Ok(id)
    }
    
    /// Cancel the active drawing
    pub fn cancel_drawing(&mut self) {
        if let Some(id) = self.active_drawing.take() {
            self.drawings.remove(&id);
        }
    }
    
    /// Add a completed drawing
    pub fn add_drawing(&mut self, drawing: Drawing) -> String {
        let id = drawing.id.clone();
        self.drawings.insert(id.clone(), drawing);
        id
    }
    
    /// Remove a drawing
    pub fn remove_drawing(&mut self, id: &str) -> Result<Drawing, String> {
        self.drawings.remove(id)
            .ok_or_else(|| "Drawing not found".to_string())
    }
    
    /// Update drawing points
    pub fn update_drawing(&mut self, id: &str, points: Vec<Point>) -> Result<(), String> {
        let drawing = self.drawings.get_mut(id)
            .ok_or("Drawing not found")?;
        
        if drawing.locked {
            return Err("Drawing is locked".to_string());
        }
        
        drawing.update_points(points);
        Ok(())
    }
    
    /// Hit test: find drawing under cursor
    pub fn hit_test(&self, x: f64, y: f64, tolerance: f64) -> Option<String> {
        // Check all drawings, return first hit
        for (id, drawing) in &self.drawings {
            if !drawing.visible {
                continue;
            }
            
            if self.hit_test_drawing(drawing, x, y, tolerance) {
                return Some(id.clone());
            }
        }
        
        None
    }
    
    fn hit_test_drawing(&self, drawing: &Drawing, x: f64, y: f64, tolerance: f64) -> bool {
        match drawing.drawing_type {
            DrawingType::TrendLine => {
                self.hit_test_line(&drawing.points, x, y, tolerance)
            },
            DrawingType::HorizontalLine => {
                (y - drawing.points[0].price).abs() < tolerance
            },
            DrawingType::VerticalLine => {
                (x - drawing.points[0].timestamp as f64).abs() < tolerance
            },
            DrawingType::Rectangle => {
                self.hit_test_rectangle(&drawing.points, x, y, tolerance)
            },
            DrawingType::FibonacciRetracement => {
                self.hit_test_fibonacci(&drawing.points, x, y, tolerance)
            },
        }
    }
    
    fn hit_test_line(&self, points: &[Point], x: f64, y: f64, tolerance: f64) -> bool {
        if points.len() < 2 {
            return false;
        }
        
        let p1 = &points[0];
        let p2 = &points[1];
        
        let distance = point_to_line_distance(
            x, y,
            p1.timestamp as f64, p1.price,
            p2.timestamp as f64, p2.price
        );
        
        distance < tolerance
    }
    
    fn hit_test_rectangle(&self, points: &[Point], x: f64, y: f64, tolerance: f64) -> bool {
        if points.len() < 2 {
            return false;
        }
        
        let x1 = points[0].timestamp as f64;
        let y1 = points[0].price;
        let x2 = points[1].timestamp as f64;
        let y2 = points[1].price;
        
        let min_x = x1.min(x2);
        let max_x = x1.max(x2);
        let min_y = y1.min(y2);
        let max_y = y1.max(y2);
        
        // Check if point is on any edge
        let on_top = (y - max_y).abs() < tolerance && x >= min_x && x <= max_x;
        let on_bottom = (y - min_y).abs() < tolerance && x >= min_x && x <= max_x;
        let on_left = (x - min_x).abs() < tolerance && y >= min_y && y <= max_y;
        let on_right = (x - max_x).abs() < tolerance && y >= min_y && y <= max_y;
        
        on_top || on_bottom || on_left || on_right
    }
    
    fn hit_test_fibonacci(&self, points: &[Point], x: f64, y: f64, tolerance: f64) -> bool {
        // Hit test on any of the fibonacci levels
        if points.len() < 2 {
            return false;
        }
        
        let levels = [0.0, 0.236, 0.382, 0.5, 0.618, 0.786, 1.0];
        let y1 = points[0].price;
        let y2 = points[1].price;
        
        for level in levels {
            let level_y = y1 + (y2 - y1) * level;
            if (y - level_y).abs() < tolerance {
                return true;
            }
        }
        
        false
    }
    
    /// Select a drawing
    pub fn select(&mut self, id: &str) {
        self.selected_drawing = Some(id.to_string());
    }
    
    /// Deselect current drawing
    pub fn deselect(&mut self) {
        self.selected_drawing = None;
    }
    
    /// Get all drawings
    pub fn drawings(&self) -> &HashMap<String, Drawing> {
        &self.drawings
    }
    
    /// Get selected drawing
    pub fn selected(&self) -> Option<&Drawing> {
        self.selected_drawing.as_ref()
            .and_then(|id| self.drawings.get(id))
    }
}

/// Calculate distance from point to line segment
fn point_to_line_distance(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    
    if dx == 0.0 && dy == 0.0 {
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }
    
    let t = ((px - x1) * dx + (py - y1) * dy) / (dx * dx + dy * dy);
    let t = t.max(0.0).min(1.0);
    
    let proj_x = x1 + t * dx;
    let proj_y = y1 + t * dy;
    
    ((px - proj_x).powi(2) + (py - proj_y).powi(2)).sqrt()
}
```

**Deliverable:** Drawing manager with add/remove/hit-test

---

### Task 7.3: Drawing Renderer

**Objective:** Render drawings using RenderCommands.

**File: `crates/chartcore/src/drawings/renderer.rs`**

```rust
use crate::rendering::commands::{RenderCommand, RenderCommandBuffer};
use crate::drawings::{Drawing, DrawingType, LineStyle};
use crate::core::Viewport;
use crate::utils::Color;

pub struct DrawingRenderer;

impl DrawingRenderer {
    pub fn render(drawing: &Drawing, 
                  viewport: &Viewport,
                  buffer: &mut RenderCommandBuffer,
                  is_selected: bool) {
        if !drawing.visible {
            return;
        }
        
        match drawing.drawing_type {
            DrawingType::TrendLine => {
                Self::render_trend_line(drawing, viewport, buffer, is_selected);
            },
            DrawingType::HorizontalLine => {
                Self::render_horizontal_line(drawing, viewport, buffer, is_selected);
            },
            DrawingType::VerticalLine => {
                Self::render_vertical_line(drawing, viewport, buffer, is_selected);
            },
            DrawingType::Rectangle => {
                Self::render_rectangle(drawing, viewport, buffer, is_selected);
            },
            DrawingType::FibonacciRetracement => {
                Self::render_fibonacci(drawing, viewport, buffer, is_selected);
            },
        }
    }
    
    fn render_trend_line(drawing: &Drawing, 
                        viewport: &Viewport,
                        buffer: &mut RenderCommandBuffer,
                        is_selected: bool) {
        if drawing.points.len() < 2 {
            return;
        }
        
        let p1 = &drawing.points[0];
        let p2 = &drawing.points[1];
        
        let mut x1 = viewport.timestamp_to_x(p1.timestamp);
        let mut y1 = viewport.price_to_y(p1.price);
        let mut x2 = viewport.timestamp_to_x(p2.timestamp);
        let mut y2 = viewport.price_to_y(p2.price);
        
        // Extend line if requested
        if drawing.style.extend_left || drawing.style.extend_right {
            let slope = (y2 - y1) / (x2 - x1);
            
            if drawing.style.extend_left {
                x1 = 0.0;
                y1 = y2 - slope * (x2 - x1);
            }
            
            if drawing.style.extend_right {
                x2 = viewport.width;
                y2 = y1 + slope * (x2 - x1);
            }
        }
        
        let color = if is_selected {
            drawing.style.color.lighten(0.3)
        } else {
            drawing.style.color.clone()
        };
        
        buffer.push(RenderCommand::DrawLine {
            x1, y1, x2, y2,
            color,
            width: drawing.style.width,
        });
        
        // Draw control points if selected
        if is_selected {
            Self::draw_control_point(p1, viewport, buffer);
            Self::draw_control_point(p2, viewport, buffer);
        }
    }
    
    fn render_horizontal_line(drawing: &Drawing,
                             viewport: &Viewport,
                             buffer: &mut RenderCommandBuffer,
                             is_selected: bool) {
        if drawing.points.is_empty() {
            return;
        }
        
        let price = drawing.points[0].price;
        let y = viewport.price_to_y(price);
        
        let color = if is_selected {
            drawing.style.color.lighten(0.3)
        } else {
            drawing.style.color.clone()
        };
        
        buffer.push(RenderCommand::DrawLine {
            x1: 0.0,
            y1: y,
            x2: viewport.width,
            y2: y,
            color,
            width: drawing.style.width,
        });
        
        // Draw price label
        buffer.push(RenderCommand::DrawText {
            text: format!("{:.2}", price),
            x: viewport.width - 60.0,
            y: y - 5.0,
            font: "12px sans-serif".to_string(),
            color: drawing.style.color.clone(),
            align: crate::rendering::commands::TextAlign::Right,
        });
    }
    
    fn render_vertical_line(drawing: &Drawing,
                           viewport: &Viewport,
                           buffer: &mut RenderCommandBuffer,
                           is_selected: bool) {
        if drawing.points.is_empty() {
            return;
        }
        
        let timestamp = drawing.points[0].timestamp;
        let x = viewport.timestamp_to_x(timestamp);
        
        let color = if is_selected {
            drawing.style.color.lighten(0.3)
        } else {
            drawing.style.color.clone()
        };
        
        buffer.push(RenderCommand::DrawLine {
            x1: x,
            y1: 0.0,
            x2: x,
            y2: viewport.height,
            color,
            width: drawing.style.width,
        });
    }
    
    fn render_rectangle(drawing: &Drawing,
                       viewport: &Viewport,
                       buffer: &mut RenderCommandBuffer,
                       is_selected: bool) {
        if drawing.points.len() < 2 {
            return;
        }
        
        let p1 = &drawing.points[0];
        let p2 = &drawing.points[1];
        
        let x1 = viewport.timestamp_to_x(p1.timestamp);
        let y1 = viewport.price_to_y(p1.price);
        let x2 = viewport.timestamp_to_x(p2.timestamp);
        let y2 = viewport.price_to_y(p2.price);
        
        let x = x1.min(x2);
        let y = y1.min(y2);
        let width = (x2 - x1).abs();
        let height = (y2 - y1).abs();
        
        let color = if is_selected {
            drawing.style.color.lighten(0.3)
        } else {
            drawing.style.color.clone()
        };
        
        buffer.push(RenderCommand::DrawRect {
            x, y, width, height,
            fill: drawing.style.fill_color.clone(),
            stroke: Some(color),
            stroke_width: drawing.style.width,
        });
        
        if is_selected {
            Self::draw_control_point(p1, viewport, buffer);
            Self::draw_control_point(p2, viewport, buffer);
        }
    }
    
    fn render_fibonacci(drawing: &Drawing,
                       viewport: &Viewport,
                       buffer: &mut RenderCommandBuffer,
                       is_selected: bool) {
        if drawing.points.len() < 2 {
            return;
        }
        
        let p1 = &drawing.points[0];
        let p2 = &drawing.points[1];
        
        let x1 = viewport.timestamp_to_x(p1.timestamp);
        let x2 = viewport.timestamp_to_x(p2.timestamp);
        let y1 = viewport.price_to_y(p1.price);
        let y2 = viewport.price_to_y(p2.price);
        
        let levels = [
            (0.0, "0%"),
            (0.236, "23.6%"),
            (0.382, "38.2%"),
            (0.5, "50%"),
            (0.618, "61.8%"),
            (0.786, "78.6%"),
            (1.0, "100%"),
        ];
        
        for (ratio, label) in levels {
            let y = y1 + (y2 - y1) * ratio;
            let price = p1.price + (p2.price - p1.price) * ratio;
            
            buffer.push(RenderCommand::DrawLine {
                x1: x1.min(x2),
                y1: y,
                x2: x1.max(x2),
                y2: y,
                color: drawing.style.color.clone().with_alpha(0.6),
                width: 1.0,
            });
            
            buffer.push(RenderCommand::DrawText {
                text: format!("{} ({:.2})", label, price),
                x: x1.max(x2) + 5.0,
                y: y - 5.0,
                font: "11px sans-serif".to_string(),
                color: drawing.style.color.clone(),
                align: crate::rendering::commands::TextAlign::Left,
            });
        }
        
        if is_selected {
            Self::draw_control_point(p1, viewport, buffer);
            Self::draw_control_point(p2, viewport, buffer);
        }
    }
    
    fn draw_control_point(point: &super::Point, 
                         viewport: &Viewport,
                         buffer: &mut RenderCommandBuffer) {
        let x = viewport.timestamp_to_x(point.timestamp);
        let y = viewport.price_to_y(point.price);
        
        buffer.push(RenderCommand::DrawRect {
            x: x - 4.0,
            y: y - 4.0,
            width: 8.0,
            height: 8.0,
            fill: Some(Color::rgb(255, 255, 255)),
            stroke: Some(Color::rgb(33, 150, 243)),
            stroke_width: 2.0,
        });
    }
}
```

**Update ChartCore to render drawings:**

```rust
impl ChartCore {
    fn render_drawings(&self, buffer: &mut RenderCommandBuffer) {
        for (id, drawing) in self.state.drawing_manager.drawings() {
            let is_selected = self.state.drawing_manager.selected()
                .map(|d| d.id == *id)
                .unwrap_or(false);
            
            DrawingRenderer::render(drawing, &self.viewport, buffer, is_selected);
        }
    }
}
```

**Deliverable:** Drawing renderer integrated into chart

---

## Week 8: Drawing UI & Interaction

### Task 8.1: Drawing Toolbar

**File: `apps/frontend/src/components/DrawingToolbar.astro`**

```html
<div 
  x-data="{ activeTool: 'cursor' }"
  class="fixed left-4 top-24 bg-card border border-border rounded-lg shadow-lg p-2 flex flex-col gap-1"
>
  <button
    @click="activeTool = 'cursor'; setDrawingMode('cursor')"
    :class="activeTool === 'cursor' ? 'bg-primary text-primary-foreground' : ''"
    class="p-2 rounded hover:bg-accent"
    title="Select (V)">
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path d="M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z" stroke-width="2"/>
    </svg>
  </button>
  
  <div class="border-t border-border my-1"></div>
  
  <button
    @click="activeTool = 'trendline'; setDrawingMode('trendline')"
    :class="activeTool === 'trendline' ? 'bg-primary text-primary-foreground' : ''"
    class="p-2 rounded hover:bg-accent"
    title="Trend Line (T)">
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path d="M4 20l16-16" stroke-width="2"/>
    </svg>
  </button>
  
  <button
    @click="activeTool = 'horizontal'; setDrawingMode('horizontal')"
    :class="activeTool === 'horizontal' ? 'bg-primary text-primary-foreground' : ''"
    class="p-2 rounded hover:bg-accent"
    title="Horizontal Line (H)">
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path d="M4 12h16" stroke-width="2"/>
    </svg>
  </button>
  
  <button
    @click="activeTool = 'vertical'; setDrawingMode('vertical')"
    :class="activeTool === 'vertical' ? 'bg-primary text-primary-foreground' : ''"
    class="p-2 rounded hover:bg-accent"
    title="Vertical Line (V)">
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path d="M12 4v16" stroke-width="2"/>
    </svg>
  </button>
  
  <button
    @click="activeTool = 'rectangle'; setDrawingMode('rectangle')"
    :class="activeTool === 'rectangle' ? 'bg-primary text-primary-foreground' : ''"
    class="p-2 rounded hover:bg-accent"
    title="Rectangle (R)">
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <rect x="4" y="6" width="16" height="12" stroke-width="2"/>
    </svg>
  </button>
  
  <button
    @click="activeTool = 'fibonacci'; setDrawingMode('fibonacci')"
    :class="activeTool === 'fibonacci' ? 'bg-primary text-primary-foreground' : ''"
    class="p-2 rounded hover:bg-accent"
    title="Fibonacci Retracement (F)">
    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path d="M4 20V4M20 20V4M4 4h16M4 8h16M4 12h16M4 16h16M4 20h16" stroke-width="1"/>
    </svg>
  </button>
</div>

<script>
  function setDrawingMode(mode: string) {
    const wasm = window.getWasm?.();
    if (wasm) {
      wasm.set_drawing_mode(mode);
    }
  }
</script>
```

**Deliverable:** Drawing toolbar UI

---

### Task 8.2: Drawing Interaction (continued in next file due to length...)

**File: `apps/frontend/src/lib/drawing-interaction.ts`**

```typescript
export class DrawingInteraction {
    private canvas: HTMLCanvasElement;
    private mode: string = 'cursor';
    private activeDrawing: string | null = null;
    
    constructor(canvas: HTMLCanvasElement) {
        this.canvas = canvas;
        this.setupEventListeners();
        this.setupKeyboardShortcuts();
    }
    
    setMode(mode: string): void {
        this.mode = mode;
        this.updateCursor();
    }
    
    private setupEventListeners(): void {
        this.canvas.addEventListener('mousedown', (e) => this.handleMouseDown(e));
        this.canvas.addEventListener('mousemove', (e) => this.handleMouseMove(e));
        this.canvas.addEventListener('mouseup', (e) => this.handleMouseUp(e));
        this.canvas.addEventListener('keydown', (e) => this.handleKeyDown(e));
    }
    
    private setupKeyboardShortcuts(): void {
        document.addEventListener('keydown', (e) => {
            if (e.target instanceof HTMLInputElement) return;
            
            switch(e.key.toLowerCase()) {
                case 'v': this.setMode('cursor'); break;
                case 't': this.setMode('trendline'); break;
                case 'h': this.setMode('horizontal'); break;
                case 'r': this.setMode('rectangle'); break;
                case 'f': this.setMode('fibonacci'); break;
                case 'escape': this.cancelDrawing(); break;
                case 'delete': this.deleteSelected(); break;
            }
        });
    }
    
    private handleMouseDown(e: MouseEvent): void {
        const point = this.getChartPoint(e);
        const wasm = window.getWasm?.();
        if (!wasm) return;
        
        if (this.mode === 'cursor') {
            // Select mode
            const hitId = wasm.hit_test_drawing(point.x, point.y, 10.0);
            if (hitId) {
                wasm.select_drawing(hitId);
            } else {
                wasm.deselect_drawing();
            }
        } else {
            // Drawing mode
            this.activeDrawing = wasm.start_drawing(
                this.mode,
                JSON.stringify(point)
            );
        }
    }
    
    private handleMouseMove(e: MouseEvent): void {
        const point = this.getChartPoint(e);
        const wasm = window.getWasm?.();
        if (!wasm) return;
        
        if (this.activeDrawing) {
            wasm.update_active_drawing(JSON.stringify(point));
        }
    }
    
    private handleMouseUp(e: MouseEvent): void {
        const wasm = window.getWasm?.();
        if (!wasm || !this.activeDrawing) return;
        
        try {
            wasm.finalize_drawing();
            this.activeDrawing = null;
            
            // Return to cursor mode after drawing
            this.setMode('cursor');
        } catch (err) {
            console.error('Failed to finalize drawing:', err);
        }
    }
    
    private getChartPoint(e: MouseEvent): { x: number, y: number } {
        const rect = this.canvas.getBoundingClientRect();
        return {
            x: e.clientX - rect.left,
            y: e.clientY - rect.top
        };
    }
    
    private updateCursor(): void {
        if (this.mode === 'cursor') {
            this.canvas.style.cursor = 'default';
        } else {
            this.canvas.style.cursor = 'crosshair';
        }
    }
    
    private cancelDrawing(): void {
        const wasm = window.getWasm?.();
        if (wasm && this.activeDrawing) {
            wasm.cancel_drawing();
            this.activeDrawing = null;
        }
        this.setMode('cursor');
    }
    
    private deleteSelected(): void {
        const wasm = window.getWasm?.();
        if (wasm) {
            const selected = wasm.get_selected_drawing();
            if (selected) {
                wasm.remove_drawing(selected.id);
            }
        }
    }
}
```

**WASM bindings for drawing interaction:**

```rust
#[wasm_bindgen]
impl WasmChart {
    pub fn start_drawing(&mut self, drawing_type: &str, point_json: &str) -> String {
        let point: Point = serde_json::from_str(point_json).unwrap();
        let dtype = match drawing_type {
            "trendline" => DrawingType::TrendLine,
            "horizontal" => DrawingType::HorizontalLine,
            "vertical" => DrawingType::VerticalLine,
            "rectangle" => DrawingType::Rectangle,
            "fibonacci" => DrawingType::FibonacciRetracement,
            _ => return String::new(),
        };
        
        self.state.drawing_manager.start_drawing(dtype, point)
    }
    
    pub fn update_active_drawing(&mut self, point_json: &str) -> Result<(), JsValue> {
        let point: Point = serde_json::from_str(point_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.state.drawing_manager.update_active_drawing(point)
            .map_err(|e| JsValue::from_str(&e))
    }
    
    pub fn finalize_drawing(&mut self) -> Result<String, JsValue> {
        self.state.drawing_manager.finalize_drawing()
            .map_err(|e| JsValue::from_str(&e))
    }
    
    pub fn cancel_drawing(&mut self) {
        self.state.drawing_manager.cancel_drawing();
    }
    
    pub fn hit_test_drawing(&self, x: f64, y: f64, tolerance: f64) -> Option<String> {
        self.state.drawing_manager.hit_test(x, y, tolerance)
    }
    
    pub fn select_drawing(&mut self, id: &str) {
        self.state.drawing_manager.select(id);
    }
    
    pub fn deselect_drawing(&mut self) {
        self.state.drawing_manager.deselect();
    }
    
    pub fn remove_drawing(&mut self, id: &str) -> Result<(), JsValue> {
        self.state.drawing_manager.remove_drawing(id)
            .map(|_| ())
            .map_err(|e| JsValue::from_str(&e))
    }
}
```

**Deliverable:** Full drawing interaction system

---

### Task 8.3: Undo/Redo System

**File: `crates/chartcore/src/commands/mod.rs`**

```rust
use serde::{Serialize, Deserialize};
use crate::drawings::{Drawing, Point, DrawingStyle};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    AddDrawing(Drawing),
    RemoveDrawing { id: String, drawing: Drawing },
    UpdateDrawingPoints { id: String, old_points: Vec<Point>, new_points: Vec<Point> },
    UpdateDrawingStyle { id: String, old_style: DrawingStyle, new_style: DrawingStyle },
}

pub struct CommandHistory {
    undo_stack: Vec<Command>,
    redo_stack: Vec<Command>,
    max_history: usize,
}

impl CommandHistory {
    pub fn new(max_history: usize) -> Self {
        Self {
            undo_stack: Vec::with_capacity(max_history),
            redo_stack: Vec::new(),
            max_history,
        }
    }
    
    pub fn execute(&mut self, cmd: Command) {
        // Clear redo stack when new command is executed
        self.redo_stack.clear();
        
        // Add to undo stack
        self.undo_stack.push(cmd);
        
        // Limit history size
        if self.undo_stack.len() > self.max_history {
            self.undo_stack.remove(0);
        }
    }
    
    pub fn undo(&mut self) -> Option<Command> {
        self.undo_stack.pop().map(|cmd| {
            self.redo_stack.push(cmd.clone());
            cmd
        })
    }
    
    pub fn redo(&mut self) -> Option<Command> {
        self.redo_stack.pop().map(|cmd| {
            self.undo_stack.push(cmd.clone());
            cmd
        })
    }
    
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }
    
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}
```

**Keyboard shortcuts:**

```typescript
// In drawing-interaction.ts
document.addEventListener('keydown', (e) => {
    if (e.ctrlKey || e.metaKey) {
        if (e.key === 'z' && !e.shiftKey) {
            e.preventDefault();
            undo();
        } else if (e.key === 'z' && e.shiftKey) {
            e.preventDefault();
            redo();
        }
    }
});

function undo() {
    const wasm = window.getWasm?.();
    if (wasm) {
        wasm.undo();
    }
}

function redo() {
    const wasm = window.getWasm?.();
    if (wasm) {
        wasm.redo();
    }
}
```

**Deliverable:** Undo/Redo system with keyboard shortcuts

---

## Phase 4 Completion Checklist

- [ ] Drawing data model (DrawingType, Point, Drawing, DrawingStyle)
- [ ] DrawingManager (add, remove, update, hit test)
- [ ] Drawing renderer (all 5 drawing types)
- [ ] Drawing toolbar UI
- [ ] Drawing interaction (mouse events)
- [ ] Keyboard shortcuts (V, T, H, R, F, Esc, Delete)
- [ ] Control points on selected drawings
- [ ] Undo/Redo system (Ctrl+Z, Ctrl+Shift+Z)
- [ ] Drawing properties panel
- [ ] All tests passing

## Success Criteria

At the end of Phase 4:
1. 5+ drawing tools fully functional
2. Smooth drawing interaction
3. Hit testing works accurately
4. Selected drawings show control points
5. Undo/Redo working for all operations
6. Keyboard shortcuts functional

**Time Budget:** 2 weeks  
**Risk Level:** Medium (complex interaction logic)  
**Dependencies:** Phase 3 (panel system)
