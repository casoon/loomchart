//! Custom Chart Plugin Example.
//!
//! This example shows how to create a custom chart plugin
//! that draws on the chart based on price action.

use loom_core::{Candle, OHLCV};
use loom_chart::{
    ChartPlugin, PluginConfig, DrawingContext,
    primitives::*,
};
use loom_indicators::prelude::*;

/// Custom plugin that draws EMA lines and highlights crossovers
pub struct EmaCrossoverPlugin {
    config: PluginConfig,
    fast_period: usize,
    slow_period: usize,
    fast_ema: Ema,
    slow_ema: Ema,
    prev_fast: Option<f64>,
    prev_slow: Option<f64>,
    candle_buffer: Vec<Candle>,
    drawing_ids: Vec<DrawingId>,
}

impl EmaCrossoverPlugin {
    pub fn new(fast_period: usize, slow_period: usize) -> Self {
        Self {
            config: PluginConfig::default(),
            fast_period,
            slow_period,
            fast_ema: Ema::new(fast_period),
            slow_ema: Ema::new(slow_period),
            prev_fast: None,
            prev_slow: None,
            candle_buffer: Vec::new(),
            drawing_ids: Vec::new(),
        }
    }

    fn draw_crossover(&mut self, ctx: &mut DrawingContext, candle: &Candle, is_bullish: bool) {
        let color = if is_bullish { Color::BULL } else { Color::BEAR };
        let icon = if is_bullish { IconType::ArrowUp } else { IconType::ArrowDown };
        let price = if is_bullish { candle.low - 1.0 } else { candle.high + 1.0 };

        // Draw arrow icon at crossover point
        let id = ctx.draw(
            Icon::new(Point::new(candle.time, price), icon)
                .color(color)
                .size(16.0)
                .tooltip(if is_bullish { "Golden Cross" } else { "Death Cross" })
        );
        self.drawing_ids.push(id);

        // Draw label
        let label_price = if is_bullish { price - 2.0 } else { price + 2.0 };
        let id = ctx.draw(
            Label::new(
                if is_bullish { "BUY" } else { "SELL" },
                candle.time,
                label_price,
            )
            .color(color)
            .font_size(10.0)
        );
        self.drawing_ids.push(id);
    }
}

impl ChartPlugin for EmaCrossoverPlugin {
    fn name(&self) -> &str {
        "EMA Crossover"
    }

    fn description(&self) -> &str {
        "Draws EMA lines and highlights crossover points"
    }

    fn config(&self) -> &PluginConfig {
        &self.config
    }

    fn set_config(&mut self, config: PluginConfig) {
        self.config = config;
    }

    fn on_historical_data(&mut self, ctx: &mut DrawingContext, candles: &[Candle]) {
        // Clear previous drawings
        for id in self.drawing_ids.drain(..) {
            ctx.remove(id);
        }

        // Reset indicators
        self.fast_ema = Ema::new(self.fast_period);
        self.slow_ema = Ema::new(self.slow_period);
        self.prev_fast = None;
        self.prev_slow = None;

        // Process historical data
        let mut fast_points: Vec<Point> = Vec::new();
        let mut slow_points: Vec<Point> = Vec::new();

        for candle in candles {
            let fast = self.fast_ema.next(candle.close);
            let slow = self.slow_ema.next(candle.close);

            if let Some(f) = fast {
                fast_points.push(Point::new(candle.time, f));
            }
            if let Some(s) = slow {
                slow_points.push(Point::new(candle.time, s));
            }

            // Check for crossovers
            if let (Some(f), Some(s), Some(pf), Some(ps)) =
                (fast, slow, self.prev_fast, self.prev_slow)
            {
                if pf <= ps && f > s {
                    self.draw_crossover(ctx, candle, true);
                } else if pf >= ps && f < s {
                    self.draw_crossover(ctx, candle, false);
                }
            }

            self.prev_fast = fast;
            self.prev_slow = slow;
        }

        // Draw EMA lines using path (connect points)
        // In a real implementation, you'd use a polyline or path primitive
        // For now, we'll draw segments between consecutive points

        let fast_color = Color::rgb(0, 150, 255); // Blue
        let slow_color = Color::rgb(255, 150, 0); // Orange

        for window in fast_points.windows(2) {
            let id = ctx.draw(
                Line::new(window[0], window[1])
                    .color(fast_color)
                    .width(1.5)
            );
            self.drawing_ids.push(id);
        }

        for window in slow_points.windows(2) {
            let id = ctx.draw(
                Line::new(window[0], window[1])
                    .color(slow_color)
                    .width(1.5)
            );
            self.drawing_ids.push(id);
        }

        self.candle_buffer = candles.to_vec();
    }

    fn on_candle(&mut self, ctx: &mut DrawingContext, candle: &Candle) {
        let fast = self.fast_ema.next(candle.close);
        let slow = self.slow_ema.next(candle.close);

        // Check for crossovers
        if let (Some(f), Some(s), Some(pf), Some(ps)) =
            (fast, slow, self.prev_fast, self.prev_slow)
        {
            if pf <= ps && f > s {
                self.draw_crossover(ctx, candle, true);
            } else if pf >= ps && f < s {
                self.draw_crossover(ctx, candle, false);
            }

            // Draw latest EMA segment
            if let Some(last_candle) = self.candle_buffer.last() {
                if let (Some(last_f), Some(last_s)) = (self.prev_fast, self.prev_slow) {
                    let id = ctx.draw(
                        Line::new(
                            Point::new(last_candle.time, last_f),
                            Point::new(candle.time, f),
                        )
                        .color(Color::rgb(0, 150, 255))
                        .width(1.5)
                    );
                    self.drawing_ids.push(id);

                    let id = ctx.draw(
                        Line::new(
                            Point::new(last_candle.time, last_s),
                            Point::new(candle.time, s),
                        )
                        .color(Color::rgb(255, 150, 0))
                        .width(1.5)
                    );
                    self.drawing_ids.push(id);
                }
            }
        }

        self.prev_fast = fast;
        self.prev_slow = slow;
        self.candle_buffer.push(candle.clone());
    }
}

fn main() {
    println!("=== Custom Chart Plugin Example ===\n");

    // Create plugin
    let plugin = EmaCrossoverPlugin::new(9, 21);
    println!("Plugin: {}", plugin.name());
    println!("Description: {}", plugin.description());
    println!("\nThis plugin:");
    println!("- Draws fast EMA (9) in blue");
    println!("- Draws slow EMA (21) in orange");
    println!("- Shows BUY arrows on golden crosses");
    println!("- Shows SELL arrows on death crosses");

    println!("\nTo use in your chart:");
    println!("  let mut registry = PluginRegistry::new();");
    println!("  registry.register(EmaCrossoverPlugin::new(9, 21));");
    println!("  registry.process_candles(&mut ctx, &candles);");
}
