//! Automatic Trendline Detection Plugin.
//!
//! Detects and draws trendlines based on swing points.

use loom_core::{Candle, OHLCV, Price, Timestamp};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::{
    primitives::*,
    plugin::{ChartPlugin, PluginConfig},
    context::{DrawingContext, LayerId},
};

/// Detected trendline
#[derive(Debug, Clone)]
struct DetectedTrendline {
    start: Point,
    end: Point,
    touches: u32,
    is_support: bool, // Support trendline (ascending) or resistance (descending)
    slope: f64,
    strength: f64,
}

impl DetectedTrendline {
    fn price_at_time(&self, time: Timestamp) -> Price {
        let time_diff = (time - self.start.time) as f64;
        self.start.price + self.slope * time_diff
    }
}

/// Trendline Detection Plugin
pub struct TrendlinePlugin {
    config: PluginConfig,
    swing_lookback: usize,
    min_touches: u32,
    tolerance: f64,
    candle_buffer: Vec<Candle>,
    trendlines: Vec<DetectedTrendline>,
    drawing_ids: Vec<DrawingId>,
}

impl TrendlinePlugin {
    pub fn new() -> Self {
        Self {
            config: PluginConfig::default(),
            swing_lookback: 3,
            min_touches: 2,
            tolerance: 0.005, // 0.5% tolerance
            candle_buffer: Vec::new(),
            trendlines: Vec::new(),
            drawing_ids: Vec::new(),
        }
    }

    fn find_swings(&self) -> (Vec<Point>, Vec<Point>) {
        let mut highs = Vec::new();
        let mut lows = Vec::new();
        let lb = self.swing_lookback;

        if self.candle_buffer.len() < lb * 2 + 1 {
            return (highs, lows);
        }

        for i in lb..self.candle_buffer.len() - lb {
            let c = &self.candle_buffer[i];

            // Check swing high
            let is_high = (0..lb).all(|j| c.high >= self.candle_buffer[i - j - 1].high)
                && (0..lb).all(|j| c.high >= self.candle_buffer[i + j + 1].high);

            // Check swing low
            let is_low = (0..lb).all(|j| c.low <= self.candle_buffer[i - j - 1].low)
                && (0..lb).all(|j| c.low <= self.candle_buffer[i + j + 1].low);

            if is_high {
                highs.push(Point::new(c.time, c.high));
            }
            if is_low {
                lows.push(Point::new(c.time, c.low));
            }
        }

        (highs, lows)
    }

    fn find_trendlines(&mut self) {
        self.trendlines.clear();

        let (highs, lows) = self.find_swings();

        // Find support trendlines (connecting lows)
        self.find_lines_from_points(&lows, true);

        // Find resistance trendlines (connecting highs)
        self.find_lines_from_points(&highs, false);

        // Sort by strength
        self.trendlines.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap());

        // Keep best trendlines
        self.trendlines.truncate(6);
    }

    fn find_lines_from_points(&mut self, points: &[Point], is_support: bool) {
        if points.len() < 2 {
            return;
        }

        let avg_price = self.candle_buffer.iter().map(|c| c.close).sum::<f64>()
            / self.candle_buffer.len() as f64;

        // Try all combinations of two points
        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                let p1 = &points[i];
                let p2 = &points[j];

                if p1.time == p2.time {
                    continue;
                }

                // Calculate slope
                let time_diff = (p2.time - p1.time) as f64;
                let slope = (p2.price - p1.price) / time_diff;

                // For support lines, we want ascending or flat
                // For resistance lines, we want descending or flat
                if is_support && slope < -self.tolerance {
                    continue;
                }
                if !is_support && slope > self.tolerance {
                    continue;
                }

                // Count touches
                let mut touches = 2u32;
                let mut total_deviation = 0.0;

                for k in 0..points.len() {
                    if k == i || k == j {
                        continue;
                    }

                    let p = &points[k];
                    let expected_price = p1.price + slope * (p.time - p1.time) as f64;
                    let deviation = (p.price - expected_price).abs() / avg_price;

                    if deviation < self.tolerance {
                        touches += 1;
                        total_deviation += deviation;
                    }
                }

                if touches >= self.min_touches {
                    let strength = touches as f64 / (1.0 + total_deviation * 10.0);

                    self.trendlines.push(DetectedTrendline {
                        start: *p1,
                        end: *p2,
                        touches,
                        is_support,
                        slope,
                        strength,
                    });
                }
            }
        }
    }

    fn draw_trendlines(&mut self, ctx: &mut DrawingContext) {
        for id in self.drawing_ids.drain(..) {
            ctx.remove(id);
        }

        let last_time = self.candle_buffer.last().map(|c| c.time).unwrap_or(0);
        let future_time = last_time + 86400000 * 10; // Project 10 days ahead

        for tl in &self.trendlines {
            let color = if tl.is_support {
                Color::SUPPORT
            } else {
                Color::RESISTANCE
            };

            let alpha = (tl.strength / 3.0).min(1.0) as f32;

            // Calculate future price on trendline
            let future_price = tl.price_at_time(future_time);

            // Draw trendline with extension
            let id = ctx.draw(
                TrendLine::new(tl.start, Point::new(future_time, future_price))
                    .extend_right_only()
                    .color(color.with_alpha(alpha))
                    .width(1.0 + tl.touches as f32 * 0.3)
            );
            self.drawing_ids.push(id);

            // Draw touch points
            for point in &[tl.start, tl.end] {
                let id = ctx.draw(
                    Icon::new(*point, IconType::CircleFilled)
                        .color(color)
                        .size(8.0)
                );
                self.drawing_ids.push(id);
            }

            // Label
            let mid_time = (tl.start.time + tl.end.time) / 2;
            let mid_price = tl.price_at_time(mid_time);
            let id = ctx.draw(
                Label::new(
                    format!("{}x", tl.touches),
                    mid_time,
                    mid_price,
                )
                .color(color)
                .font_size(10.0)
                .offset(0.0, if tl.is_support { 10.0 } else { -10.0 })
            );
            self.drawing_ids.push(id);
        }
    }
}

impl Default for TrendlinePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartPlugin for TrendlinePlugin {
    fn name(&self) -> &str {
        "Auto Trendlines"
    }

    fn description(&self) -> &str {
        "Automatically detects and draws trendlines"
    }

    fn config(&self) -> &PluginConfig {
        &self.config
    }

    fn set_config(&mut self, config: PluginConfig) {
        self.config = config;
    }

    fn on_historical_data(&mut self, ctx: &mut DrawingContext, candles: &[Candle]) {
        self.candle_buffer.clear();
        self.candle_buffer.extend_from_slice(candles);
        self.find_trendlines();
        self.draw_trendlines(ctx);
    }

    fn on_candle(&mut self, ctx: &mut DrawingContext, candle: &Candle) {
        self.candle_buffer.push(candle.clone());

        if self.candle_buffer.len() > 200 {
            self.candle_buffer.remove(0);
        }

        // Re-analyze every 5 candles
        if self.candle_buffer.len() % 5 == 0 {
            self.find_trendlines();
            self.draw_trendlines(ctx);
        }
    }
}
