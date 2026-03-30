# Phase 2: Indikatoren Funktional Machen (Weeks 3-4)

**Goal:** Bridge 70+ indicators to actual chart rendering

**Prerequisites:** Phase 1 complete (render command pattern working)

## Week 3: Indicator Output System

### Task 3.1: Indicator Output Interface

**Objective:** Define standard interface for all indicators to output render-ready data.

**File: `crates/chartcore/src/indicators/output.rs`**

```rust
use crate::utils::Color;

/// Standard output format for all indicators
#[derive(Debug, Clone)]
pub enum IndicatorOutput {
    /// Single line (e.g., SMA, EMA)
    SingleLine {
        values: Vec<Option<f64>>,  // None for NaN/missing
        color: Color,
        width: f64,
        style: LineStyle,
    },
    
    /// Multiple lines (e.g., Bollinger Bands, Ichimoku)
    MultiLine {
        lines: Vec<LineData>,
    },
    
    /// Histogram (e.g., MACD histogram, Volume)
    Histogram {
        values: Vec<f64>,
        positive_color: Color,
        negative_color: Color,
        zero_line: bool,
    },
    
    /// Cloud/Area between two lines (e.g., Ichimoku cloud)
    CloudArea {
        upper: Vec<f64>,
        lower: Vec<f64>,
        bullish_color: Color,
        bearish_color: Color,
    },
    
    /// Scatter plot (e.g., Pivot Points)
    Scatter {
        points: Vec<(usize, f64)>,  // (index, value)
        color: Color,
        size: f64,
        shape: MarkerShape,
    },
}

#[derive(Debug, Clone)]
pub struct LineData {
    pub values: Vec<Option<f64>>,
    pub color: Color,
    pub width: f64,
    pub style: LineStyle,
    pub label: String,
}

#[derive(Debug, Clone)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone)]
pub enum MarkerShape {
    Circle,
    Square,
    Triangle,
    Cross,
}

/// Standard interface all indicators must implement
pub trait Indicator: Send + Sync {
    /// Calculate indicator values from candle data
    fn calculate(&self, candles: &[Candle]) -> IndicatorOutput;
    
    /// Get the scale range for this indicator (None = use price scale)
    fn get_scale_range(&self) -> Option<(f64, f64)>;
    
    /// Can this indicator be overlaid on the price chart?
    fn supports_overlay(&self) -> bool;
    
    /// Indicator name for display
    fn name(&self) -> &str;
    
    /// Indicator parameters for serialization
    fn get_params(&self) -> serde_json::Value;
    
    /// Update parameters (returns error if invalid)
    fn set_params(&mut self, params: serde_json::Value) -> Result<(), String>;
}
```

**Deliverable:** Indicator output interface defined

---

### Task 3.2: Update Existing Indicators

**Objective:** Refactor all 70+ indicators to implement new Indicator trait.

**Example: RSI (Relative Strength Index)**

**File: `crates/chartcore/src/indicators/rsi.rs`**

```rust
use super::output::*;
use crate::data::Candle;
use crate::utils::Color;

pub struct RSI {
    period: usize,
    color: Color,
}

impl RSI {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            color: Color::rgb(255, 152, 0),  // Orange
        }
    }
    
    fn compute_rsi(&self, candles: &[Candle]) -> Vec<Option<f64>> {
        if candles.len() < self.period {
            return vec![None; candles.len()];
        }
        
        let mut rsi_values = vec![None; candles.len()];
        let mut gains = Vec::new();
        let mut losses = Vec::new();
        
        // Calculate price changes
        for i in 1..candles.len() {
            let change = candles[i].close - candles[i - 1].close;
            if change > 0.0 {
                gains.push(change);
                losses.push(0.0);
            } else {
                gains.push(0.0);
                losses.push(change.abs());
            }
        }
        
        // Calculate RSI
        for i in self.period..candles.len() {
            let avg_gain: f64 = gains[i - self.period..i].iter().sum::<f64>() / self.period as f64;
            let avg_loss: f64 = losses[i - self.period..i].iter().sum::<f64>() / self.period as f64;
            
            let rs = if avg_loss == 0.0 {
                100.0
            } else {
                avg_gain / avg_loss
            };
            
            rsi_values[i] = Some(100.0 - (100.0 / (1.0 + rs)));
        }
        
        rsi_values
    }
}

impl Indicator for RSI {
    fn calculate(&self, candles: &[Candle]) -> IndicatorOutput {
        IndicatorOutput::SingleLine {
            values: self.compute_rsi(candles),
            color: self.color.clone(),
            width: 2.0,
            style: LineStyle::Solid,
        }
    }
    
    fn get_scale_range(&self) -> Option<(f64, f64)> {
        Some((0.0, 100.0))  // RSI always 0-100
    }
    
    fn supports_overlay(&self) -> bool {
        false  // RSI requires separate panel
    }
    
    fn name(&self) -> &str {
        "RSI"
    }
    
    fn get_params(&self) -> serde_json::Value {
        serde_json::json!({
            "period": self.period,
            "color": self.color.to_hex(),
        })
    }
    
    fn set_params(&mut self, params: serde_json::Value) -> Result<(), String> {
        if let Some(period) = params["period"].as_u64() {
            if period < 2 || period > 200 {
                return Err("Period must be between 2 and 200".to_string());
            }
            self.period = period as usize;
        }
        
        if let Some(color_hex) = params["color"].as_str() {
            self.color = Color::from_hex(color_hex)?;
        }
        
        Ok(())
    }
}
```

**Example: MACD (Multi-line + Histogram)**

**File: `crates/chartcore/src/indicators/macd.rs`**

```rust
impl Indicator for MACD {
    fn calculate(&self, candles: &[Candle]) -> IndicatorOutput {
        let (macd_line, signal_line, histogram) = self.compute_macd(candles);
        
        // Return multi-component output
        // We'll use MultiLine for MACD + Signal, and overlay Histogram
        IndicatorOutput::MultiLine {
            lines: vec![
                LineData {
                    values: macd_line,
                    color: Color::rgb(33, 150, 243),  // Blue
                    width: 2.0,
                    style: LineStyle::Solid,
                    label: "MACD".to_string(),
                },
                LineData {
                    values: signal_line,
                    color: Color::rgb(255, 87, 34),  // Orange
                    width: 2.0,
                    style: LineStyle::Solid,
                    label: "Signal".to_string(),
                },
            ],
        }
    }
    
    fn get_scale_range(&self) -> Option<(f64, f64)> {
        None  // Auto-scale based on data
    }
    
    fn supports_overlay(&self) -> bool {
        false  // Requires separate panel
    }
}
```

**Migration Checklist for All 70+ Indicators:**

Create file: `INDICATOR_MIGRATION_CHECKLIST.md`

- [ ] Trend Indicators (10):
  - [ ] SMA (Simple Moving Average)
  - [ ] EMA (Exponential Moving Average)
  - [ ] WMA (Weighted Moving Average)
  - [ ] DEMA (Double EMA)
  - [ ] TEMA (Triple EMA)
  - [ ] HMA (Hull Moving Average)
  - [ ] VWMA (Volume Weighted MA)
  - [ ] SMMA (Smoothed MA)
  - [ ] KAMA (Kaufman Adaptive MA)
  - [ ] MAMA (MESA Adaptive MA)

- [ ] Momentum Indicators (15):
  - [ ] RSI
  - [ ] MACD
  - [ ] Stochastic
  - [ ] Williams %R
  - [ ] CCI (Commodity Channel Index)
  - [ ] ROC (Rate of Change)
  - [ ] Momentum
  - [ ] TSI (True Strength Index)
  - [ ] UO (Ultimate Oscillator)
  - [ ] AO (Awesome Oscillator)
  - [ ] CMO (Chande Momentum Oscillator)
  - [ ] TRIX
  - [ ] DPO (Detrended Price Oscillator)
  - [ ] KST (Know Sure Thing)
  - [ ] DMI (Directional Movement Index)

- [ ] Volatility Indicators (8):
  - [ ] Bollinger Bands
  - [ ] ATR (Average True Range)
  - [ ] Keltner Channels
  - [ ] Donchian Channels
  - [ ] Standard Deviation
  - [ ] Chaikin Volatility
  - [ ] Historical Volatility
  - [ ] True Range

- [ ] Volume Indicators (10):
  - [ ] Volume
  - [ ] OBV (On Balance Volume)
  - [ ] Chaikin Money Flow
  - [ ] Money Flow Index
  - [ ] VWAP (Volume Weighted Average Price)
  - [ ] Accumulation/Distribution
  - [ ] Force Index
  - [ ] Ease of Movement
  - [ ] Volume Oscillator
  - [ ] PVT (Price Volume Trend)

- [ ] Bill Williams (5):
  - [ ] Alligator
  - [ ] Fractals
  - [ ] Gator Oscillator
  - [ ] Market Facilitation Index
  - [ ] Awesome Oscillator

- [ ] Ichimoku (1):
  - [ ] Ichimoku Cloud (5 lines)

- [ ] Others (21+):
  - [ ] Fibonacci Retracement
  - [ ] Pivot Points
  - [ ] Parabolic SAR
  - [ ] Supertrend
  - [ ] Aroon
  - [ ] ... (continue for all 70+)

**Deliverable:** All indicators implement Indicator trait

---

### Task 3.3: Indicator Rendering

**Objective:** Create renderer that converts IndicatorOutput to RenderCommands.

**File: `crates/chartcore/src/rendering/indicator_renderer.rs`**

```rust
use crate::rendering::commands::{RenderCommand, RenderCommandBuffer};
use crate::indicators::output::*;
use crate::core::Viewport;

pub struct IndicatorRenderer {
    viewport: Viewport,
}

impl IndicatorRenderer {
    pub fn render(&self,
                  output: &IndicatorOutput,
                  buffer: &mut RenderCommandBuffer) {
        match output {
            IndicatorOutput::SingleLine { values, color, width, style } => {
                self.render_line(values, color, *width, style, buffer);
            },
            
            IndicatorOutput::MultiLine { lines } => {
                for line in lines {
                    self.render_line(&line.values, &line.color, line.width, &line.style, buffer);
                }
            },
            
            IndicatorOutput::Histogram { values, positive_color, negative_color, zero_line } => {
                self.render_histogram(values, positive_color, negative_color, *zero_line, buffer);
            },
            
            IndicatorOutput::CloudArea { upper, lower, bullish_color, bearish_color } => {
                self.render_cloud(upper, lower, bullish_color, bearish_color, buffer);
            },
            
            IndicatorOutput::Scatter { points, color, size, shape } => {
                self.render_scatter(points, color, *size, shape, buffer);
            },
        }
    }
    
    fn render_line(&self,
                   values: &[Option<f64>],
                   color: &Color,
                   width: f64,
                   style: &LineStyle,
                   buffer: &mut RenderCommandBuffer) {
        let mut points = Vec::new();
        
        for (i, value_opt) in values.iter().enumerate() {
            if let Some(value) = value_opt {
                let x = self.viewport.index_to_x(i);
                let y = self.viewport.value_to_y(*value);
                points.push((x, y));
            } else {
                // Break line on missing data
                if !points.is_empty() {
                    buffer.push(RenderCommand::DrawIndicatorLine {
                        points: points.clone(),
                        color: color.clone(),
                        width,
                        style: self.convert_line_style(style),
                    });
                    points.clear();
                }
            }
        }
        
        // Draw remaining points
        if !points.is_empty() {
            buffer.push(RenderCommand::DrawIndicatorLine {
                points,
                color: color.clone(),
                width,
                style: self.convert_line_style(style),
            });
        }
    }
    
    fn render_histogram(&self,
                        values: &[f64],
                        pos_color: &Color,
                        neg_color: &Color,
                        zero_line: bool,
                        buffer: &mut RenderCommandBuffer) {
        let zero_y = self.viewport.value_to_y(0.0);
        
        for (i, &value) in values.iter().enumerate() {
            let x = self.viewport.index_to_x(i);
            let value_y = self.viewport.value_to_y(value);
            let color = if value >= 0.0 { pos_color } else { neg_color };
            
            let bar_width = self.viewport.bar_width * 0.6;
            let height = (value_y - zero_y).abs();
            let y = value_y.min(zero_y);
            
            buffer.push(RenderCommand::DrawRect {
                x: x - bar_width / 2.0,
                y,
                width: bar_width,
                height,
                fill: Some(color.clone()),
                stroke: None,
                stroke_width: 0.0,
            });
        }
        
        // Draw zero line
        if zero_line {
            buffer.push(RenderCommand::DrawLine {
                x1: 0.0,
                y1: zero_y,
                x2: self.viewport.width,
                y2: zero_y,
                color: Color::rgba(255, 255, 255, 0.3),
                width: 1.0,
            });
        }
    }
    
    fn render_cloud(&self,
                    upper: &[f64],
                    lower: &[f64],
                    bullish_color: &Color,
                    bearish_color: &Color,
                    buffer: &mut RenderCommandBuffer) {
        for i in 1..upper.len().min(lower.len()) {
            let is_bullish = upper[i] > lower[i];
            let color = if is_bullish {
                bullish_color.with_alpha(0.2)
            } else {
                bearish_color.with_alpha(0.2)
            };
            
            // Draw cloud segment as polygon
            let x1 = self.viewport.index_to_x(i - 1);
            let x2 = self.viewport.index_to_x(i);
            let upper1 = self.viewport.value_to_y(upper[i - 1]);
            let upper2 = self.viewport.value_to_y(upper[i]);
            let lower1 = self.viewport.value_to_y(lower[i - 1]);
            let lower2 = self.viewport.value_to_y(lower[i]);
            
            // For simplicity, draw as filled rectangle
            // TODO: Implement polygon rendering for smoother clouds
            buffer.push(RenderCommand::DrawRect {
                x: x1,
                y: upper1.min(lower1),
                width: x2 - x1,
                height: (upper1 - lower1).abs(),
                fill: Some(color),
                stroke: None,
                stroke_width: 0.0,
            });
        }
    }
}
```

**Update ChartCore to use IndicatorRenderer:**

**File: `crates/chartcore/src/lib.rs`**

```rust
impl ChartCore {
    fn render_indicators(&self, buffer: &mut RenderCommandBuffer) {
        let renderer = IndicatorRenderer::new(self.viewport.clone());
        
        for panel in self.state.panel_manager.panels() {
            for indicator in panel.indicators() {
                let output = indicator.calculate(&self.state.candle_store.candles);
                renderer.render(&output, buffer);
            }
        }
    }
}
```

**Deliverable:** Indicators rendering to chart via command pattern

---

## Week 4: Indicator UI Integration

### Task 4.1: Enhanced Indicator Selector

**Objective:** Improve IndicatorSelector with categories, search, and previews.

**File: `apps/frontend/src/components/IndicatorSelector.astro`**

```html
<div 
  x-show="showIndicatorSelector" 
  @click.self="showIndicatorSelector = false"
  x-cloak
  class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center"
  x-data="{
    searchQuery: '',
    selectedCategory: 'all',
    selectedIndicator: null,
    displayMode: 'panel'
  }"
>
  <div class="bg-card border border-border rounded-lg shadow-xl w-[800px] max-h-[600px] flex flex-col">
    <!-- Header -->
    <div class="p-4 border-b border-border">
      <h2 class="text-lg font-semibold">Add Indicator</h2>
      
      <!-- Search -->
      <input 
        type="text" 
        x-model="searchQuery"
        placeholder="Search indicators..."
        class="mt-2 w-full px-3 py-2 bg-background border border-border rounded"
      />
    </div>
    
    <!-- Category Tabs -->
    <div class="flex border-b border-border overflow-x-auto">
      <button @click="selectedCategory = 'all'" 
              :class="selectedCategory === 'all' ? 'border-b-2 border-primary' : ''"
              class="px-4 py-2 whitespace-nowrap">
        All
      </button>
      <button @click="selectedCategory = 'trend'"
              :class="selectedCategory === 'trend' ? 'border-b-2 border-primary' : ''"
              class="px-4 py-2 whitespace-nowrap">
        Trend
      </button>
      <button @click="selectedCategory = 'momentum'"
              :class="selectedCategory === 'momentum' ? 'border-b-2 border-primary' : ''"
              class="px-4 py-2 whitespace-nowrap">
        Momentum
      </button>
      <button @click="selectedCategory = 'volatility'"
              :class="selectedCategory = 'volatility' ? 'border-b-2 border-primary' : ''"
              class="px-4 py-2 whitespace-nowrap">
        Volatility
      </button>
      <button @click="selectedCategory = 'volume'"
              :class="selectedCategory === 'volume' ? 'border-b-2 border-primary' : ''"
              class="px-4 py-2 whitespace-nowrap">
        Volume
      </button>
    </div>
    
    <!-- Indicator List -->
    <div class="flex-1 overflow-y-auto p-4 grid grid-cols-3 gap-3">
      <template x-for="indicator in filteredIndicators" :key="indicator.id">
        <button
          @click="selectedIndicator = indicator"
          :class="selectedIndicator?.id === indicator.id ? 'ring-2 ring-primary' : ''"
          class="p-3 bg-background border border-border rounded hover:bg-accent transition text-left"
        >
          <div class="font-medium" x-text="indicator.name"></div>
          <div class="text-xs text-muted-foreground mt-1" x-text="indicator.description"></div>
        </button>
      </template>
    </div>
    
    <!-- Display Mode Selection -->
    <div class="p-4 border-t border-border">
      <div class="mb-3">
        <label class="text-sm font-medium">Display Mode</label>
        <div class="flex gap-2 mt-2">
          <button
            @click="displayMode = 'panel'"
            :class="displayMode === 'panel' ? 'bg-primary text-primary-foreground' : 'bg-background'"
            class="flex-1 px-4 py-2 rounded border border-border">
            Separate Panel
          </button>
          <button
            @click="displayMode = 'overlay'"
            :class="displayMode === 'overlay' ? 'bg-primary text-primary-foreground' : 'bg-background'"
            :disabled="selectedIndicator && !selectedIndicator.supportsOverlay"
            class="flex-1 px-4 py-2 rounded border border-border disabled:opacity-50">
            Overlay on Chart
          </button>
        </div>
      </div>
      
      <!-- Add Button -->
      <button
        @click="addIndicator(selectedIndicator, displayMode)"
        :disabled="!selectedIndicator"
        class="w-full px-4 py-2 bg-primary text-primary-foreground rounded disabled:opacity-50">
        Add Indicator
      </button>
    </div>
  </div>
</div>

<script>
  function addIndicator(indicator, displayMode) {
    const wasm = window.getWasm?.();
    if (!wasm) {
      alert('Chart not initialized');
      return;
    }
    
    try {
      wasm.add_indicator(
        indicator.id,
        displayMode === 'panel' ? null : 'main',
        JSON.stringify(indicator.defaultParams)
      );
      
      // Close modal
      Alpine.store('trading').showIndicatorSelector = false;
      
      // Trigger re-render
      window.dispatchEvent(new CustomEvent('indicatorAdded'));
    } catch (err) {
      console.error('Failed to add indicator:', err);
      alert('Failed to add indicator: ' + err);
    }
  }
</script>
```

**Deliverable:** Enhanced indicator selector with search and categories

---

### Task 4.2: Active Indicator Management

**File: `apps/frontend/src/components/PanelManager.astro`**

Add per-indicator controls in the panel list.

**Deliverable:** Indicator management UI (show/hide, color, settings, remove)

---

### Task 4.3: WASM Bindings for Indicators

**File: `packages/wasm-core/src/lib.rs`**

```rust
#[wasm_bindgen]
impl WasmChart {
    pub fn add_indicator(&mut self,
                        indicator_type: &str,
                        panel_id: Option<String>,
                        config_json: &str) -> Result<String, JsValue> {
        let config: serde_json::Value = serde_json::from_str(config_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        // Create indicator based on type
        let indicator: Box<dyn Indicator> = match indicator_type {
            "rsi" => Box::new(RSI::new(config["period"].as_u64().unwrap_or(14) as usize)),
            "macd" => Box::new(MACD::new(12, 26, 9)),
            "sma" => Box::new(SMA::new(config["period"].as_u64().unwrap_or(20) as usize)),
            // ... all 70+ indicators
            _ => return Err(JsValue::from_str(&format!("Unknown indicator: {}", indicator_type))),
        };
        
        let id = self.state.panel_manager.add_indicator(indicator, panel_id)
            .map_err(|e| JsValue::from_str(&e))?;
        
        Ok(id)
    }
    
    pub fn update_indicator_params(&mut self,
                                   indicator_id: &str,
                                   params_json: &str) -> Result<(), JsValue> {
        let params: serde_json::Value = serde_json::from_str(params_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        self.state.panel_manager.get_indicator_mut(indicator_id)
            .ok_or_else(|| JsValue::from_str("Indicator not found"))?
            .set_params(params)
            .map_err(|e| JsValue::from_str(&e))?;
        
        Ok(())
    }
    
    pub fn remove_indicator(&mut self, indicator_id: &str) -> Result<(), JsValue> {
        self.state.panel_manager.remove_indicator(indicator_id)
            .map_err(|e| JsValue::from_str(&e))
    }
    
    pub fn get_active_indicators(&self) -> JsValue {
        let indicators = self.state.panel_manager.get_all_indicators();
        serde_wasm_bindgen::to_value(&indicators).unwrap()
    }
}
```

**Deliverable:** Complete WASM API for indicator management

---

## Phase 2 Completion Checklist

- [ ] IndicatorOutput interface defined
- [ ] All 70+ indicators implement Indicator trait
- [ ] IndicatorRenderer converts outputs to commands
- [ ] Indicators rendering correctly on chart
- [ ] Enhanced IndicatorSelector UI
- [ ] Active indicator management in PanelManager
- [ ] WASM bindings for add/update/remove indicators
- [ ] Parameter changes update in real-time
- [ ] Color customization working
- [ ] All tests passing
- [ ] Documentation updated

## Success Criteria

At the end of Phase 2:
1. All 70+ indicators visible and rendering correctly
2. Users can add/remove/configure indicators via UI
3. Real-time parameter updates working
4. No performance regression (still 60fps target)

**Time Budget:** 2 weeks  
**Risk Level:** Medium (lots of indicators to migrate)  
**Dependencies:** Phase 1 (render command pattern)
