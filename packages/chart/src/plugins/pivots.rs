//! Pivot Points Plugin.
//!
//! Visualizes various pivot point calculations.

use loom_core::{Candle, OHLCV, Price, Timestamp};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::{
    primitives::*,
    plugin::{ChartPlugin, PluginConfig},
    context::{DrawingContext, LayerId},
};

/// Pivot type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PivotType {
    /// Standard pivot points
    #[default]
    Standard,
    /// Fibonacci pivot points
    Fibonacci,
    /// Woodie pivot points
    Woodie,
    /// Camarilla pivot points
    Camarilla,
    /// DeMark pivot points
    DeMark,
}

/// Pivot configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PivotConfig {
    /// Pivot calculation type
    pub pivot_type: PivotType,
    /// Show pivot point
    pub show_pivot: bool,
    /// Show support levels
    pub show_support: bool,
    /// Show resistance levels
    pub show_resistance: bool,
    /// Number of levels to show (1-4)
    pub levels: u8,
    /// Line style
    pub line_style: LineStyle,
    /// Show labels
    pub show_labels: bool,
    /// Colors
    pub pivot_color: Color,
    pub support_color: Color,
    pub resistance_color: Color,
}

impl Default for PivotConfig {
    fn default() -> Self {
        Self {
            pivot_type: PivotType::Standard,
            show_pivot: true,
            show_support: true,
            show_resistance: true,
            levels: 3,
            line_style: LineStyle::Solid,
            show_labels: true,
            pivot_color: Color::YELLOW,
            support_color: Color::GREEN,
            resistance_color: Color::RED,
        }
    }
}

/// Calculated pivot levels
#[derive(Debug, Clone)]
pub struct PivotLevels {
    pub pivot: Price,
    pub r1: Price,
    pub r2: Price,
    pub r3: Price,
    pub r4: Price,
    pub s1: Price,
    pub s2: Price,
    pub s3: Price,
    pub s4: Price,
}

impl PivotLevels {
    /// Calculate standard pivot points
    pub fn standard(high: Price, low: Price, close: Price) -> Self {
        let pivot = (high + low + close) / 3.0;
        let r1 = 2.0 * pivot - low;
        let s1 = 2.0 * pivot - high;
        let r2 = pivot + (high - low);
        let s2 = pivot - (high - low);
        let r3 = high + 2.0 * (pivot - low);
        let s3 = low - 2.0 * (high - pivot);
        let r4 = r3 + (high - low);
        let s4 = s3 - (high - low);

        Self { pivot, r1, r2, r3, r4, s1, s2, s3, s4 }
    }

    /// Calculate Fibonacci pivot points
    pub fn fibonacci(high: Price, low: Price, close: Price) -> Self {
        let pivot = (high + low + close) / 3.0;
        let range = high - low;

        let r1 = pivot + 0.382 * range;
        let r2 = pivot + 0.618 * range;
        let r3 = pivot + 1.0 * range;
        let r4 = pivot + 1.618 * range;
        let s1 = pivot - 0.382 * range;
        let s2 = pivot - 0.618 * range;
        let s3 = pivot - 1.0 * range;
        let s4 = pivot - 1.618 * range;

        Self { pivot, r1, r2, r3, r4, s1, s2, s3, s4 }
    }

    /// Calculate Woodie pivot points
    pub fn woodie(high: Price, low: Price, close: Price) -> Self {
        let pivot = (high + low + 2.0 * close) / 4.0;
        let r1 = 2.0 * pivot - low;
        let r2 = pivot + (high - low);
        let r3 = high + 2.0 * (pivot - low);
        let r4 = r3 + (high - low);
        let s1 = 2.0 * pivot - high;
        let s2 = pivot - (high - low);
        let s3 = low - 2.0 * (high - pivot);
        let s4 = s3 - (high - low);

        Self { pivot, r1, r2, r3, r4, s1, s2, s3, s4 }
    }

    /// Calculate Camarilla pivot points
    pub fn camarilla(high: Price, low: Price, close: Price) -> Self {
        let range = high - low;
        let pivot = (high + low + close) / 3.0;

        let r1 = close + range * 1.1 / 12.0;
        let r2 = close + range * 1.1 / 6.0;
        let r3 = close + range * 1.1 / 4.0;
        let r4 = close + range * 1.1 / 2.0;
        let s1 = close - range * 1.1 / 12.0;
        let s2 = close - range * 1.1 / 6.0;
        let s3 = close - range * 1.1 / 4.0;
        let s4 = close - range * 1.1 / 2.0;

        Self { pivot, r1, r2, r3, r4, s1, s2, s3, s4 }
    }

    /// Calculate DeMark pivot points
    pub fn demark(high: Price, low: Price, open: Price, close: Price) -> Self {
        let x = if close < open {
            high + 2.0 * low + close
        } else if close > open {
            2.0 * high + low + close
        } else {
            high + low + 2.0 * close
        };

        let pivot = x / 4.0;
        let r1 = x / 2.0 - low;
        let s1 = x / 2.0 - high;

        // DeMark only has one level, use pivot for others
        Self {
            pivot,
            r1,
            r2: r1,
            r3: r1,
            r4: r1,
            s1,
            s2: s1,
            s3: s1,
            s4: s1,
        }
    }

    /// Get resistance levels as array
    pub fn resistances(&self) -> [Price; 4] {
        [self.r1, self.r2, self.r3, self.r4]
    }

    /// Get support levels as array
    pub fn supports(&self) -> [Price; 4] {
        [self.s1, self.s2, self.s3, self.s4]
    }
}

/// Pivot Points Plugin
pub struct PivotPlugin {
    config: PluginConfig,
    pivot_config: PivotConfig,
    current_levels: Option<PivotLevels>,
    period_high: Price,
    period_low: Price,
    period_open: Price,
    period_close: Price,
    period_start: Timestamp,
    drawing_ids: Vec<DrawingId>,
}

impl PivotPlugin {
    pub fn new() -> Self {
        Self {
            config: PluginConfig::default(),
            pivot_config: PivotConfig::default(),
            current_levels: None,
            period_high: 0.0,
            period_low: f64::MAX,
            period_open: 0.0,
            period_close: 0.0,
            period_start: 0,
            drawing_ids: Vec::new(),
        }
    }

    pub fn with_type(mut self, pivot_type: PivotType) -> Self {
        self.pivot_config.pivot_type = pivot_type;
        self
    }

    pub fn with_levels(mut self, levels: u8) -> Self {
        self.pivot_config.levels = levels.clamp(1, 4);
        self
    }

    fn calculate_pivots(&mut self) {
        if self.period_high == 0.0 || self.period_low == f64::MAX {
            return;
        }

        self.current_levels = Some(match self.pivot_config.pivot_type {
            PivotType::Standard => PivotLevels::standard(
                self.period_high,
                self.period_low,
                self.period_close,
            ),
            PivotType::Fibonacci => PivotLevels::fibonacci(
                self.period_high,
                self.period_low,
                self.period_close,
            ),
            PivotType::Woodie => PivotLevels::woodie(
                self.period_high,
                self.period_low,
                self.period_close,
            ),
            PivotType::Camarilla => PivotLevels::camarilla(
                self.period_high,
                self.period_low,
                self.period_close,
            ),
            PivotType::DeMark => PivotLevels::demark(
                self.period_high,
                self.period_low,
                self.period_open,
                self.period_close,
            ),
        });
    }

    fn draw_levels(&mut self, ctx: &mut DrawingContext) {
        // Clear previous drawings
        for id in self.drawing_ids.drain(..) {
            ctx.remove(id);
        }

        let levels = match &self.current_levels {
            Some(l) => l,
            None => return,
        };

        let cfg = &self.pivot_config;

        // Draw pivot point
        if cfg.show_pivot {
            let id = ctx.draw(
                HorizontalLine::new(levels.pivot)
                    .color(cfg.pivot_color)
                    .style(cfg.line_style)
                    .label("P")
            );
            self.drawing_ids.push(id);
        }

        // Draw resistance levels
        if cfg.show_resistance {
            let resistances = levels.resistances();
            for (i, &price) in resistances.iter().take(cfg.levels as usize).enumerate() {
                let label = format!("R{}", i + 1);
                let alpha = 1.0 - (i as f32 * 0.2);
                let id = ctx.draw(
                    HorizontalLine::new(price)
                        .color(cfg.resistance_color.with_alpha(alpha))
                        .style(cfg.line_style)
                        .label(label)
                );
                self.drawing_ids.push(id);
            }
        }

        // Draw support levels
        if cfg.show_support {
            let supports = levels.supports();
            for (i, &price) in supports.iter().take(cfg.levels as usize).enumerate() {
                let label = format!("S{}", i + 1);
                let alpha = 1.0 - (i as f32 * 0.2);
                let id = ctx.draw(
                    HorizontalLine::new(price)
                        .color(cfg.support_color.with_alpha(alpha))
                        .style(cfg.line_style)
                        .label(label)
                );
                self.drawing_ids.push(id);
            }
        }
    }

    fn is_new_period(&self, candle: &Candle) -> bool {
        // Simple daily detection - could be enhanced
        let day_ms = 86400 * 1000;
        (candle.time / day_ms) != (self.period_start / day_ms)
    }
}

impl Default for PivotPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartPlugin for PivotPlugin {
    fn name(&self) -> &str {
        "Pivot Points"
    }

    fn description(&self) -> &str {
        "Displays pivot points with support and resistance levels"
    }

    fn config(&self) -> &PluginConfig {
        &self.config
    }

    fn set_config(&mut self, config: PluginConfig) {
        self.config = config;
    }

    fn on_historical_data(&mut self, ctx: &mut DrawingContext, candles: &[Candle]) {
        if candles.is_empty() {
            return;
        }

        // Use yesterday's data for today's pivots
        // Find the last complete day
        let last = candles.last().unwrap();
        let day_ms = 86400 * 1000;
        let today_start = (last.time / day_ms) * day_ms;

        // Collect yesterday's candles
        let yesterday_start = today_start - day_ms;
        let yesterday_candles: Vec<_> = candles
            .iter()
            .filter(|c| c.time >= yesterday_start && c.time < today_start)
            .collect();

        if !yesterday_candles.is_empty() {
            self.period_high = yesterday_candles.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
            self.period_low = yesterday_candles.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);
            self.period_open = yesterday_candles.first().unwrap().open;
            self.period_close = yesterday_candles.last().unwrap().close;
            self.period_start = today_start;

            self.calculate_pivots();
            self.draw_levels(ctx);
        }
    }

    fn on_candle(&mut self, ctx: &mut DrawingContext, candle: &Candle) {
        if self.is_new_period(candle) {
            // Calculate pivots from previous period
            self.calculate_pivots();
            self.draw_levels(ctx);

            // Reset for new period
            self.period_high = candle.high;
            self.period_low = candle.low;
            self.period_open = candle.open;
            self.period_start = candle.time;
        } else {
            // Update current period stats
            self.period_high = self.period_high.max(candle.high);
            self.period_low = self.period_low.min(candle.low);
        }

        self.period_close = candle.close;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_pivots() {
        let levels = PivotLevels::standard(110.0, 90.0, 100.0);
        assert!((levels.pivot - 100.0).abs() < 0.01);
        assert!(levels.r1 > levels.pivot);
        assert!(levels.s1 < levels.pivot);
    }

    #[test]
    fn test_fibonacci_pivots() {
        let levels = PivotLevels::fibonacci(110.0, 90.0, 100.0);
        assert!((levels.r1 - levels.pivot).abs() < (levels.r2 - levels.pivot).abs());
    }
}
