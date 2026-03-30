//! Support and Resistance Detection Plugin.
//!
//! Automatically detects and draws support/resistance levels.

use loom_core::{Candle, OHLCV, Price, Timestamp};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::{
    primitives::*,
    plugin::{ChartPlugin, PluginConfig},
    context::{DrawingContext, LayerId},
};

/// Detected level
#[derive(Debug, Clone)]
struct Level {
    price: Price,
    touches: u32,
    first_touch: Timestamp,
    last_touch: Timestamp,
    is_resistance: bool,
    strength: f64,
}

/// Support/Resistance Plugin
pub struct SupportResistancePlugin {
    config: PluginConfig,
    lookback: usize,
    tolerance: f64,
    min_touches: u32,
    candle_buffer: Vec<Candle>,
    levels: Vec<Level>,
    drawing_ids: Vec<DrawingId>,
}

impl SupportResistancePlugin {
    pub fn new() -> Self {
        Self {
            config: PluginConfig::default(),
            lookback: 100,
            tolerance: 0.002, // 0.2% tolerance
            min_touches: 2,
            candle_buffer: Vec::new(),
            levels: Vec::new(),
            drawing_ids: Vec::new(),
        }
    }

    pub fn with_lookback(mut self, lookback: usize) -> Self {
        self.lookback = lookback;
        self
    }

    pub fn with_tolerance(mut self, tolerance: f64) -> Self {
        self.tolerance = tolerance;
        self
    }

    fn find_levels(&mut self) {
        self.levels.clear();

        if self.candle_buffer.len() < 10 {
            return;
        }

        // Find swing highs and lows
        let mut swings: Vec<(Timestamp, Price, bool)> = Vec::new();

        for i in 2..self.candle_buffer.len() - 2 {
            let c = &self.candle_buffer[i];

            // Swing high
            if c.high > self.candle_buffer[i - 1].high
                && c.high > self.candle_buffer[i - 2].high
                && c.high > self.candle_buffer[i + 1].high
                && c.high > self.candle_buffer[i + 2].high
            {
                swings.push((c.time, c.high, true));
            }

            // Swing low
            if c.low < self.candle_buffer[i - 1].low
                && c.low < self.candle_buffer[i - 2].low
                && c.low < self.candle_buffer[i + 1].low
                && c.low < self.candle_buffer[i + 2].low
            {
                swings.push((c.time, c.low, false));
            }
        }

        // Cluster swings into levels
        let avg_price = self.candle_buffer.iter().map(|c| c.close).sum::<f64>()
            / self.candle_buffer.len() as f64;

        for swing in &swings {
            let (time, price, is_high) = *swing;

            // Check if this swing clusters with an existing level
            let mut found = false;
            for level in &mut self.levels {
                let diff = (price - level.price).abs() / avg_price;
                if diff < self.tolerance {
                    level.touches += 1;
                    level.last_touch = time;
                    level.strength += 1.0 / (1.0 + diff * 100.0);
                    found = true;
                    break;
                }
            }

            if !found {
                self.levels.push(Level {
                    price,
                    touches: 1,
                    first_touch: time,
                    last_touch: time,
                    is_resistance: is_high,
                    strength: 1.0,
                });
            }
        }

        // Filter by minimum touches
        self.levels.retain(|l| l.touches >= self.min_touches);

        // Sort by strength
        self.levels.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap());

        // Keep top levels
        self.levels.truncate(10);
    }

    fn draw_levels(&mut self, ctx: &mut DrawingContext) {
        for id in self.drawing_ids.drain(..) {
            ctx.remove(id);
        }

        let last_time = self.candle_buffer.last().map(|c| c.time).unwrap_or(0);

        for level in &self.levels {
            let color = if level.is_resistance {
                Color::RESISTANCE
            } else {
                Color::SUPPORT
            };

            let alpha = (level.strength / 5.0).min(1.0) as f32;

            // Draw level line
            let id = ctx.draw(
                HorizontalLine::from_to(level.price, level.first_touch, last_time + 86400000 * 5)
                    .color(color.with_alpha(alpha))
                    .width(1.0 + level.touches as f32 * 0.5)
                    .style(if level.touches >= 3 { LineStyle::Solid } else { LineStyle::Dashed })
                    .label(format!(
                        "{} ({}x)",
                        if level.is_resistance { "R" } else { "S" },
                        level.touches
                    ))
            );
            self.drawing_ids.push(id);

            // Draw zone around level
            let zone_height = level.price * self.tolerance;
            let id = ctx.draw_on_layer(
                PriceZone::from_to(
                    level.price + zone_height,
                    level.price - zone_height,
                    level.first_touch,
                    last_time,
                )
                .fill(color.with_alpha(0.1)),
                LayerId::BACKGROUND,
            );
            self.drawing_ids.push(id);
        }
    }
}

impl Default for SupportResistancePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartPlugin for SupportResistancePlugin {
    fn name(&self) -> &str {
        "Support/Resistance"
    }

    fn description(&self) -> &str {
        "Automatically detects support and resistance levels"
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
        self.find_levels();
        self.draw_levels(ctx);
    }

    fn on_candle(&mut self, ctx: &mut DrawingContext, candle: &Candle) {
        self.candle_buffer.push(candle.clone());

        if self.candle_buffer.len() > self.lookback * 2 {
            self.candle_buffer.remove(0);
        }

        // Re-analyze periodically
        if self.candle_buffer.len() % 10 == 0 {
            self.find_levels();
            self.draw_levels(ctx);
        }
    }
}
