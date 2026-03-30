# Chart Engine Migration Plan

**Date:** 2025-12-31  
**Decision:** Use chartcore (Rust) as sole chart engine  
**Status:** ✅ Decision Made, Migration In Progress

---

## Executive Summary

Based on comprehensive analysis (see CHART_ENGINE_COMPARISON.md), we are consolidating on **chartcore (Rust)** as the single chart engine for Loom.

**Key Facts:**
- ✅ chartcore has **70 indicators** vs 0 in TypeScript
- ✅ chartcore has **panel system** vs none in TypeScript
- ✅ Porting TypeScript → Rust: **1-2 weeks**
- ❌ Porting Rust → TypeScript: **3-4 months**

**Action:** Keep Rust, port optimizations from TypeScript, delete TypeScript engine.

---

## Migration Strategy

### Phase 1: Port TypeScript Features to Rust (Week 1-2)

#### Task 1.1: Adaptive FPS System ✅

**Source:** `packages/chart-core/src/core/adaptive-fps.ts`

**Target:** `crates/chartcore/src/core/adaptive_fps.rs`

**Features to Port:**
- Battery-aware frame scheduling
- Complexity-based FPS adjustment
- Idle detection (reduce FPS when no interaction)
- Statistics tracking (avg FPS, dropped frames)
- Auto-adjustment based on performance

**Implementation:**

```rust
// File: crates/chartcore/src/core/adaptive_fps.rs

use wasm_bindgen::prelude::*;
use web_sys::js_sys;

pub struct AdaptiveFPSConfig {
    pub max_fps: f64,
    pub min_fps: f64,
    pub idle_fps: f64,
    pub battery_aware: bool,
    pub auto_adjust: bool,
}

impl Default for AdaptiveFPSConfig {
    fn default() -> Self {
        Self {
            max_fps: 60.0,
            min_fps: 10.0,
            idle_fps: 1.0,
            battery_aware: true,
            auto_adjust: true,
        }
    }
}

pub struct AdaptiveFrameScheduler {
    config: AdaptiveFPSConfig,
    current_fps: f64,
    target_fps: f64,
    last_frame_time: f64,
    last_interaction_time: f64,
    frame_times: Vec<f64>,
    dropped_frames: usize,
}

impl AdaptiveFrameScheduler {
    pub fn new(config: AdaptiveFPSConfig) -> Self {
        Self {
            target_fps: config.max_fps,
            current_fps: config.max_fps,
            config,
            last_frame_time: js_sys::Date::now(),
            last_interaction_time: js_sys::Date::now(),
            frame_times: Vec::with_capacity(60),
            dropped_frames: 0,
        }
    }
    
    pub fn should_render(&self) -> bool {
        let now = js_sys::Date::now();
        let target_interval = 1000.0 / self.target_fps;
        
        (now - self.last_frame_time) >= target_interval
    }
    
    pub fn start_frame(&mut self) -> f64 {
        js_sys::Date::now()
    }
    
    pub fn end_frame(&mut self, start_time: f64) {
        let frame_time = js_sys::Date::now() - start_time;
        
        // Track frame times (rolling window of 60 frames)
        self.frame_times.push(frame_time);
        if self.frame_times.len() > 60 {
            self.frame_times.remove(0);
        }
        
        // Update current FPS
        if frame_time > 0.0 {
            let instant_fps = 1000.0 / frame_time;
            self.current_fps = self.current_fps * 0.9 + instant_fps * 0.1; // Smooth
        }
        
        // Auto-adjust target FPS
        if self.config.auto_adjust {
            self.auto_adjust_target();
        }
        
        self.last_frame_time = js_sys::Date::now();
    }
    
    pub fn record_interaction(&mut self) {
        self.last_interaction_time = js_sys::Date::now();
        self.target_fps = self.config.max_fps; // Boost FPS on interaction
    }
    
    pub fn get_time_since_interaction(&self) -> f64 {
        js_sys::Date::now() - self.last_interaction_time
    }
    
    fn auto_adjust_target(&mut self) {
        let time_since_interaction = self.get_time_since_interaction();
        
        // Reduce FPS when idle
        if time_since_interaction > 5000.0 {
            // Idle for 5+ seconds
            self.target_fps = self.config.idle_fps;
        } else if time_since_interaction > 1000.0 {
            // Idle for 1-5 seconds
            self.target_fps = self.config.min_fps;
        } else {
            // Active
            self.target_fps = self.config.max_fps;
        }
        
        // Battery-aware: reduce FPS when on battery
        if self.config.battery_aware {
            if let Some(battery) = get_battery_status() {
                if !battery.charging && battery.level < 0.3 {
                    // Low battery, reduce FPS
                    self.target_fps = self.target_fps.min(30.0);
                }
            }
        }
        
        // If we're dropping frames, reduce target
        let avg_frame_time = self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64;
        let target_frame_time = 1000.0 / self.target_fps;
        
        if avg_frame_time > target_frame_time * 1.5 {
            // Can't keep up, reduce target
            self.target_fps = (self.target_fps * 0.9).max(self.config.min_fps);
        }
    }
    
    pub fn get_stats(&self) -> FPSStats {
        let avg_frame_time = if self.frame_times.is_empty() {
            0.0
        } else {
            self.frame_times.iter().sum::<f64>() / self.frame_times.len() as f64
        };
        
        FPSStats {
            current_fps: self.current_fps,
            target_fps: self.target_fps,
            avg_frame_time_ms: avg_frame_time,
            dropped_frames: self.dropped_frames,
        }
    }
}

pub struct FPSStats {
    pub current_fps: f64,
    pub target_fps: f64,
    pub avg_frame_time_ms: f64,
    pub dropped_frames: usize,
}

// Battery API (optional, may not be supported)
struct BatteryStatus {
    charging: bool,
    level: f64,
}

fn get_battery_status() -> Option<BatteryStatus> {
    // WASM battery API access (if available)
    // This is a stub - actual implementation would use web_sys
    None
}
```

**Estimated Time:** 1 day

---

#### Task 1.2: Advanced Invalidation System ✅

**Source:** `packages/chart-core/src/core/invalidation.ts`

**Target:** `crates/chartcore/src/core/invalidation.rs`

**Features:**
- Full invalidation (complete re-render)
- Light invalidation (only new data)
- Cursor invalidation (crosshair only)
- Dirty region tracking

**Implementation:**

```rust
// File: crates/chartcore/src/core/invalidation.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidationLevel {
    None,       // No changes
    Cursor,     // Only crosshair moved
    Light,      // New candle added (incremental update)
    Full,       // Complete re-render needed
}

pub struct InvalidationMask {
    candles: InvalidationLevel,
    indicators: InvalidationLevel,
    drawings: InvalidationLevel,
    axes: InvalidationLevel,
    crosshair: InvalidationLevel,
}

impl InvalidationMask {
    pub fn new() -> Self {
        Self {
            candles: InvalidationLevel::None,
            indicators: InvalidationLevel::None,
            drawings: InvalidationLevel::None,
            axes: InvalidationLevel::None,
            crosshair: InvalidationLevel::None,
        }
    }
    
    /// Mark all components for full re-render
    pub fn invalidate_all(&mut self, level: InvalidationLevel) {
        self.candles = level;
        self.indicators = level;
        self.drawings = level;
        self.axes = level;
        if level != InvalidationLevel::None {
            self.crosshair = level;
        }
    }
    
    /// Mark specific component
    pub fn invalidate_candles(&mut self, level: InvalidationLevel) {
        self.candles = self.candles.max(level);
    }
    
    pub fn invalidate_indicators(&mut self, level: InvalidationLevel) {
        self.indicators = self.indicators.max(level);
    }
    
    pub fn invalidate_drawings(&mut self, level: InvalidationLevel) {
        self.drawings = self.drawings.max(level);
    }
    
    pub fn invalidate_axes(&mut self, level: InvalidationLevel) {
        self.axes = self.axes.max(level);
    }
    
    pub fn invalidate_crosshair(&mut self) {
        self.crosshair = InvalidationLevel::Cursor;
    }
    
    /// Check if any rendering is needed
    pub fn needs_render(&self) -> bool {
        self.candles != InvalidationLevel::None ||
        self.indicators != InvalidationLevel::None ||
        self.drawings != InvalidationLevel::None ||
        self.axes != InvalidationLevel::None ||
        self.crosshair != InvalidationLevel::None
    }
    
    /// Check if full re-render is needed
    pub fn needs_full_render(&self) -> bool {
        self.candles == InvalidationLevel::Full ||
        self.indicators == InvalidationLevel::Full ||
        self.drawings == InvalidationLevel::Full ||
        self.axes == InvalidationLevel::Full
    }
    
    /// Check if only cursor update is needed
    pub fn is_cursor_only(&self) -> bool {
        self.crosshair == InvalidationLevel::Cursor &&
        self.candles == InvalidationLevel::None &&
        self.indicators == InvalidationLevel::None &&
        self.drawings == InvalidationLevel::None &&
        self.axes == InvalidationLevel::None
    }
    
    /// Reset all invalidation flags
    pub fn reset(&mut self) {
        self.candles = InvalidationLevel::None;
        self.indicators = InvalidationLevel::None;
        self.drawings = InvalidationLevel::None;
        self.axes = InvalidationLevel::None;
        self.crosshair = InvalidationLevel::None;
    }
}

impl PartialOrd for InvalidationLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InvalidationLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let self_val = match self {
            InvalidationLevel::None => 0,
            InvalidationLevel::Cursor => 1,
            InvalidationLevel::Light => 2,
            InvalidationLevel::Full => 3,
        };
        let other_val = match other {
            InvalidationLevel::None => 0,
            InvalidationLevel::Cursor => 1,
            InvalidationLevel::Light => 2,
            InvalidationLevel::Full => 3,
        };
        self_val.cmp(&other_val)
    }
}
```

**Usage in render loop:**

```rust
// In Chart::render()
pub fn render(&mut self) {
    if !self.invalidation.needs_render() {
        return; // Skip render if nothing changed
    }
    
    if self.invalidation.is_cursor_only() {
        // Fast path: only redraw crosshair
        self.render_crosshair_only();
    } else if self.invalidation.needs_full_render() {
        // Full render
        self.render_all();
    } else {
        // Incremental update
        self.render_incremental();
    }
    
    self.invalidation.reset();
}
```

**Estimated Time:** 1 day

---

#### Task 1.3: Batch Rendering Optimizations ✅

**Source:** `packages/chart-core/src/renderer/canvas.ts`

**Target:** `crates/chartcore/src/renderers/canvas2d.rs`

**Feature:** `drawCandlesBatch` - render multiple candles in one operation

**Implementation:**

```rust
// In Canvas2DRenderer

pub fn draw_candles_batch(
    &self,
    candles: &[CandleRenderData],
    bullish_color: Color,
    bearish_color: Color,
) {
    self.ctx.begin_path();
    
    // Draw all wicks in one path
    for candle in candles {
        let color = if candle.close_y <= candle.open_y {
            &bullish_color
        } else {
            &bearish_color
        };
        
        self.ctx.set_stroke_style(&color.to_js_value());
        self.ctx.move_to(candle.x + candle.width / 2.0, candle.high_y);
        self.ctx.line_to(candle.x + candle.width / 2.0, candle.low_y);
    }
    
    self.ctx.stroke();
    
    // Draw all bodies
    for candle in candles {
        let color = if candle.close_y <= candle.open_y {
            &bullish_color
        } else {
            &bearish_color
        };
        
        let body_top = candle.open_y.min(candle.close_y);
        let body_height = (candle.open_y - candle.close_y).abs().max(1.0);
        
        self.ctx.set_fill_style(&color.to_js_value());
        self.ctx.fill_rect(candle.x, body_top, candle.width, body_height);
    }
}

pub struct CandleRenderData {
    pub x: f64,
    pub open_y: f64,
    pub high_y: f64,
    pub low_y: f64,
    pub close_y: f64,
    pub width: f64,
}
```

**Estimated Time:** 0.5 days

---

#### Task 1.4: Volume Pane Improvements ✅

**Source:** `packages/chart-core/src/core/chart.ts` (renderVolumePane)

**Target:** `crates/chartcore/src/core/volume_pane.rs`

**Features:**
- Separate viewport for volume
- Auto-scaling
- Color-coded bars
- Opacity for volume bars

**Implementation:** Add to existing chartcore volume rendering.

**Estimated Time:** 0.5 days

---

### Phase 2: Update All Imports (Week 2)

#### Task 2.1: Frontend Imports ✅

**Files to Update:**
- `apps/frontend/src/lib/app-rust.ts`
- `apps/frontend/src/lib/chart-bridge.ts`
- Any components using chart

**Change:**
```typescript
// BEFORE (if any references to chart-core)
import { Chart } from '@loom/chart-core';

// AFTER
// Use WASM only
const wasm = window.getWasm?.();
```

**Estimated Time:** 1 hour

---

#### Task 2.2: Remove chart-core Package ✅

**Steps:**
1. Delete `packages/chart-core` directory
2. Update `pnpm-workspace.yaml` (remove chart-core)
3. Update root `package.json` (remove chart-core dependency)
4. Run `pnpm install` to clean lockfile

**Estimated Time:** 30 minutes

---

### Phase 3: Testing & Verification (Week 2)

#### Task 3.1: Test All 70 Indicators ✅

Create test suite to verify all indicators still work:

```rust
// File: crates/chartcore/tests/indicators_test.rs

#[cfg(test)]
mod tests {
    use chartcore::plugins::*;
    
    #[test]
    fn test_all_indicators_load() {
        let indicators = [
            "adr", "adx", "alligator", "alma", "aroon", "atr",
            // ... all 70
        ];
        
        for indicator in indicators {
            let result = create_indicator(indicator);
            assert!(result.is_ok(), "Failed to load {}", indicator);
        }
    }
}
```

**Estimated Time:** 1 day

---

#### Task 3.2: Performance Benchmark ✅

**Test Scenarios:**
1. Render 1000 candles (no indicators)
2. Render 1000 candles + 5 indicators
3. Render 1000 candles + 10 indicators
4. Pan operation (100 iterations)
5. Zoom operation (100 iterations)

**Targets:**
- 1000 candles: 60fps (16.6ms per frame)
- With 5 indicators: 60fps
- With 10 indicators: 45fps+
- Pan/Zoom: < 20ms response

**Estimated Time:** 1 day

---

### Phase 4: Documentation Update (Week 2)

#### Task 4.1: Update Architecture Docs ✅

**Files to Update:**
- README.md (remove chart-core references)
- TECHNICAL_ASSESSMENT.txt (update engine info)
- RENDERER_ARCHITECTURE.txt (single engine)

**Estimated Time:** 2 hours

---

## Migration Timeline

### Week 1

**Monday (Day 1):**
- ✅ Complete CHART_ENGINE_COMPARISON.md
- ✅ Create CHART_ENGINE_MIGRATION.md
- ⏳ Start Task 1.1: Adaptive FPS

**Tuesday (Day 2):**
- ⏳ Complete Task 1.1: Adaptive FPS
- ⏳ Start Task 1.2: Invalidation System

**Wednesday (Day 3):**
- ⏳ Complete Task 1.2: Invalidation System
- ⏳ Start Task 1.3: Batch Rendering

**Thursday (Day 4):**
- ⏳ Complete Task 1.3: Batch Rendering
- ⏳ Complete Task 1.4: Volume Pane
- ⏳ Integration testing

**Friday (Day 5):**
- ⏳ Fix any integration issues
- ⏳ Code review
- ⏳ Documentation

---

### Week 2

**Monday (Day 6):**
- ⏳ Task 2.1: Update frontend imports
- ⏳ Task 2.2: Remove chart-core package
- ⏳ Verify build works

**Tuesday (Day 7):**
- ⏳ Task 3.1: Test all 70 indicators
- ⏳ Fix any indicator issues

**Wednesday (Day 8):**
- ⏳ Task 3.2: Performance benchmarks
- ⏳ Optimize if needed

**Thursday (Day 9):**
- ⏳ Task 4.1: Update documentation
- ⏳ Final integration testing

**Friday (Day 10):**
- ⏳ Code review
- ⏳ Commit and tag release
- ✅ Phase 1 Complete!

---

## Rollback Plan

If migration fails, we can rollback:

1. Restore `packages/chart-core` from git
2. Revert import changes
3. Keep both engines temporarily
4. Investigate issues

**Risk:** Low (chart-core wasn't being used anyway)

---

## Success Criteria

Migration is successful when:

- ✅ All 70 indicators working
- ✅ Panel system functional
- ✅ Adaptive FPS implemented
- ✅ Invalidation system working
- ✅ Performance targets met (60fps with 5 indicators)
- ✅ chart-core package deleted
- ✅ All tests passing
- ✅ Documentation updated
- ✅ No TypeScript chart engine references remain

---

## Post-Migration Tasks

After migration completes:

1. **Render Command Pattern** (Phase 1, Week 2)
   - See ROADMAP_PHASE1.md Task 2.2
   - Prepare for future WebWorker migration

2. **Indicator UI Completion** (Phase 2)
   - See ROADMAP_PHASE2.md
   - Connect all 70 indicators to UI

3. **Drawing Tools** (Phase 4)
   - See ROADMAP_PHASE4.md
   - Implement trend line, h-line, etc.

---

## Notes

- The TypeScript engine had good optimizations but zero features
- The Rust engine has all features but can benefit from TS optimizations
- This migration gives us best of both worlds
- Total effort: 10 days vs 90+ days to port the other direction
- This is the right decision ✅

---

## Status Tracking

| Task | Status | Date Completed |
|------|--------|----------------|
| Decision Made | ✅ | 2025-12-31 |
| Comparison Doc | ✅ | 2025-12-31 |
| Migration Plan | ✅ | 2025-12-31 |
| Adaptive FPS | ⏳ | - |
| Invalidation | ⏳ | - |
| Batch Rendering | ⏳ | - |
| Volume Pane | ⏳ | - |
| Update Imports | ⏳ | - |
| Delete chart-core | ⏳ | - |
| Test Indicators | ⏳ | - |
| Performance Bench | ⏳ | - |
| Update Docs | ⏳ | - |
| **Complete** | ⏳ | - |

---

**Next Step:** Begin Task 1.1 - Implement Adaptive FPS in Rust

See ROADMAP_PHASE1.md for full Phase 1 details.
