//! Chart State Management - Central state container for the chart engine

use super::types::{Candle, Timeframe};
use super::viewport::{PriceRange, TimeRange, Viewport};
use crate::primitives::{CandleStyle, Color};

/// Chart configuration
#[derive(Debug, Clone)]
pub struct ChartOptions {
    pub background_color: Color,
    pub grid_color: Color,
    pub text_color: Color,
    pub crosshair_color: Color,
    pub bullish_color: Color,
    pub bearish_color: Color,
    pub unchanged_color: Color,
    pub candle_style: CandleStyle,
    pub show_grid: bool,
    pub show_crosshair: bool,
    pub show_volume: bool,
}

impl Default for ChartOptions {
    fn default() -> Self {
        Self {
            background_color: Color::rgba(10, 14, 18, 1.0),
            grid_color: Color::rgba(45, 54, 64, 0.3),
            text_color: Color::rgba(231, 233, 234, 1.0),
            crosshair_color: Color::rgba(139, 152, 165, 0.5),
            bullish_color: Color::rgba(34, 197, 94, 1.0),
            bearish_color: Color::rgba(239, 68, 68, 1.0),
            unchanged_color: Color::rgba(201, 203, 207, 1.0), // Gray for Doji
            candle_style: CandleStyle::Candlestick,
            show_grid: true,
            show_crosshair: true,
            show_volume: true,
        }
    }
}

/// Crosshair state
#[derive(Debug, Clone, Copy)]
pub struct CrosshairState {
    pub visible: bool,
    pub x: f64,
    pub y: f64,
    pub time: i64,
    pub price: f64,
}

impl Default for CrosshairState {
    fn default() -> Self {
        Self {
            visible: false,
            x: 0.0,
            y: 0.0,
            time: 0,
            price: 0.0,
        }
    }
}

/// Interaction state for tracking mouse/touch input
#[derive(Debug, Clone, PartialEq)]
pub enum InteractionState {
    Idle,
    Panning {
        start_x: f64,
        start_y: f64,
    },
    Selecting {
        start_x: f64,
        start_y: f64,
    },
    ScalingPrice {
        start_y: f64,                    // Inverted Y coordinate from start
        initial_price_range: PriceRange, // Snapshot of price range
    },
    ScalingTime {
        start_x: f64,                  // X coordinate from start
        initial_time_range: TimeRange, // Snapshot of time range
    },
}

impl Default for InteractionState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Main chart state container
pub struct ChartState {
    pub viewport: Viewport,
    pub candles: Vec<Candle>,
    pub options: ChartOptions,
    pub crosshair: CrosshairState,
    pub interaction: InteractionState,
    pub timeframe: Timeframe,
    pub tool_manager: crate::tools::ToolManager,
    dirty: bool,
}

impl ChartState {
    pub fn new(width: u32, height: u32, timeframe: Timeframe) -> Self {
        let mut viewport = Viewport::new(width, height);
        viewport.timeframe = timeframe; // Set timeframe in viewport

        Self {
            viewport,
            candles: Vec::new(),
            options: ChartOptions::default(),
            crosshair: CrosshairState::default(),
            interaction: InteractionState::default(),
            timeframe,
            tool_manager: crate::tools::ToolManager::new(),
            dirty: true,
        }
    }

    /// Set candle data and auto-fit viewport
    pub fn set_candles(&mut self, candles: Vec<Candle>) {
        self.candles = candles;
        if !self.candles.is_empty() {
            self.fit_to_data();
        }
        self.mark_dirty();
    }

    /// Add a single candle (for real-time updates)
    pub fn add_candle(&mut self, candle: Candle) {
        // Check if we should update the last candle or add a new one
        if let Some(last) = self.candles.last_mut() {
            if last.time == candle.time {
                *last = candle;
            } else {
                self.candles.push(candle);
            }
        } else {
            self.candles.push(candle);
        }
        self.mark_dirty();
    }

    /// Fit viewport to show all data
    pub fn fit_to_data(&mut self) {
        if self.candles.is_empty() {
            return;
        }

        let time_start = self.candles.first().unwrap().time;
        let time_end = self.candles.last().unwrap().time;

        let mut min_price = f64::MAX;
        let mut max_price = f64::MIN;

        for candle in &self.candles {
            min_price = min_price.min(candle.l);
            max_price = max_price.max(candle.h);
        }

        // Add 5% padding
        let price_padding = (max_price - min_price) * 0.05;

        self.viewport.fit_to_data(
            TimeRange {
                start: time_start,
                end: time_end,
            },
            PriceRange {
                min: min_price - price_padding,
                max: max_price + price_padding,
            },
        );

        self.mark_dirty();
    }

    /// Resize the chart
    pub fn resize(&mut self, width: u32, height: u32) {
        self.viewport.dimensions.width = width;
        self.viewport.dimensions.height = height;
        self.mark_dirty();
    }

    /// Pan the viewport
    pub fn pan(&mut self, delta_x: i32, delta_y: i32) {
        self.viewport.pan(delta_x, delta_y);
        self.mark_dirty();
    }

    /// Zoom the viewport
    pub fn zoom(&mut self, factor: f64, center_x: Option<u32>) {
        self.viewport.zoom(factor, center_x);
        self.mark_dirty();
    }

    /// Update crosshair position
    pub fn update_crosshair(&mut self, x: f64, y: f64) {
        self.crosshair.visible = true;
        self.crosshair.x = x;
        self.crosshair.y = y;
        self.crosshair.time = self.viewport.x_to_time(x);
        self.crosshair.price = self.viewport.y_to_price(y);
        self.mark_dirty();
    }

    /// Hide crosshair
    pub fn hide_crosshair(&mut self) {
        self.crosshair.visible = false;
        self.mark_dirty();
    }

    /// Get candles visible in current viewport
    pub fn visible_candles(&self) -> Vec<&Candle> {
        let time_range = &self.viewport.time;
        self.candles
            .iter()
            .filter(|c| c.time >= time_range.start && c.time <= time_range.end)
            .collect()
    }

    /// Find candle at a given time
    pub fn candle_at_time(&self, time: i64) -> Option<&Candle> {
        self.candles.iter().find(|c| c.time == time)
    }

    /// Find closest candle to a given x coordinate
    pub fn candle_at_x(&self, x: f64) -> Option<&Candle> {
        let time = self.viewport.x_to_time(x);
        let bar_duration = self.timeframe.duration_ms() / 1000;

        // Find closest candle within half bar width
        self.candles
            .iter()
            .filter(|c| (c.time - time).abs() < bar_duration / 2)
            .min_by_key(|c| (c.time - time).abs())
    }

    /// Find candle at position with hit-testing (includes Y coordinate check)
    pub fn candle_at_position(&self, x: f64, y: f64) -> Option<&Candle> {
        let bar_width = self.viewport.bar_width();

        self.visible_candles().into_iter().find(|candle| {
            let candle_x = self.viewport.time_to_x(candle.time);
            let open_y = self.viewport.price_to_y(candle.o);
            let high_y = self.viewport.price_to_y(candle.h);
            let low_y = self.viewport.price_to_y(candle.l);
            let close_y = self.viewport.price_to_y(candle.c);

            candle.in_range(x, y, candle_x, bar_width, open_y, high_y, low_y, close_y)
        })
    }

    /// Get OHLC data at crosshair position (for tooltip)
    pub fn get_ohlc_at_crosshair(&self) -> Option<(f64, f64, f64, f64, f64)> {
        if !self.crosshair.visible {
            return None;
        }

        self.candle_at_x(self.crosshair.x)
            .map(|c| (c.o, c.h, c.l, c.c, c.v))
    }

    /// Get formatted OHLC string at crosshair position
    pub fn get_ohlc_formatted(&self) -> Option<String> {
        if !self.crosshair.visible {
            return None;
        }

        self.candle_at_x(self.crosshair.x).map(|c| c.format_ohlc())
    }

    /// Mark state as dirty (needs redraw)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if state needs redraw
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear dirty flag after rendering
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Start panning interaction
    pub fn start_pan(&mut self, x: f64, y: f64) {
        self.interaction = InteractionState::Panning {
            start_x: x,
            start_y: y,
        };
    }

    /// Start selection interaction
    pub fn start_select(&mut self, x: f64, y: f64) {
        self.interaction = InteractionState::Selecting {
            start_x: x,
            start_y: y,
        };
    }

    /// End current interaction
    pub fn end_interaction(&mut self) {
        self.interaction = InteractionState::Idle;
    }

    /// Get OHLCV data for display
    pub fn get_ohlcv_at_crosshair(&self) -> Option<(f64, f64, f64, f64, f64)> {
        if !self.crosshair.visible {
            return None;
        }

        self.candle_at_x(self.crosshair.x)
            .map(|c| (c.o, c.h, c.l, c.c, c.v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chart_state_creation() {
        let state = ChartState::new(800, 600, Timeframe::M5);
        assert_eq!(state.viewport.dimensions.width, 800);
        assert_eq!(state.viewport.dimensions.height, 600);
        assert!(state.is_dirty());
    }

    #[test]
    fn test_set_candles_and_fit() {
        let mut state = ChartState::new(800, 600, Timeframe::M5);
        let candles = vec![
            Candle::new(1000, 100.0, 105.0, 95.0, 102.0, 1000.0),
            Candle::new(1300, 102.0, 108.0, 100.0, 106.0, 1200.0),
        ];

        state.set_candles(candles);
        assert_eq!(state.candles.len(), 2);
        assert!(state.viewport.time.start <= 1000);
        assert!(state.viewport.time.end >= 1300);
    }

    #[test]
    fn test_crosshair_update() {
        let mut state = ChartState::new(800, 600, Timeframe::M5);
        state.update_crosshair(400.0, 300.0);

        assert!(state.crosshair.visible);
        assert_eq!(state.crosshair.x, 400.0);
        assert_eq!(state.crosshair.y, 300.0);
    }

    #[test]
    fn test_interaction_states() {
        let mut state = ChartState::new(800, 600, Timeframe::M5);

        state.start_pan(100.0, 200.0);
        assert!(matches!(
            state.interaction,
            InteractionState::Panning { .. }
        ));

        state.end_interaction();
        assert_eq!(state.interaction, InteractionState::Idle);
    }
}
