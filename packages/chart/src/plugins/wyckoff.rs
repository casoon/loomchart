//! Wyckoff Analysis Visualization Plugin.
//!
//! Visualizes Wyckoff patterns, springs, upthrusts, and phases.

use loom_core::{Candle, OHLCV, Price, Timestamp};
use loom_signals::wyckoff::{WyckoffAnalyzer, WyckoffPhase, Spring, Upthrust};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::{
    primitives::*,
    plugin::{ChartPlugin, PluginConfig},
    context::{DrawingContext, LayerId},
};

/// Wyckoff visualization configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WyckoffVisualConfig {
    /// Show spring patterns
    pub show_springs: bool,
    /// Show upthrust patterns
    pub show_upthrusts: bool,
    /// Show phase background
    pub show_phases: bool,
    /// Show volume analysis
    pub show_volume_analysis: bool,
    /// Minimum strength to display
    pub min_strength: f64,
    /// Colors
    pub spring_color: Color,
    pub upthrust_color: Color,
    pub accumulation_color: Color,
    pub distribution_color: Color,
}

impl Default for WyckoffVisualConfig {
    fn default() -> Self {
        Self {
            show_springs: true,
            show_upthrusts: true,
            show_phases: true,
            show_volume_analysis: false,
            min_strength: 50.0,
            spring_color: Color::GREEN,
            upthrust_color: Color::RED,
            accumulation_color: Color::rgba(38, 166, 154, 0.1),
            distribution_color: Color::rgba(239, 83, 80, 0.1),
        }
    }
}

/// Wyckoff Visualization Plugin
pub struct WyckoffPlugin {
    config: PluginConfig,
    visual_config: WyckoffVisualConfig,
    analyzer: WyckoffAnalyzer,
    candle_buffer: Vec<Candle>,
    drawing_ids: Vec<DrawingId>,
    detected_springs: Vec<(Timestamp, Spring)>,
    detected_upthrusts: Vec<(Timestamp, Upthrust)>,
}

impl WyckoffPlugin {
    pub fn new() -> Self {
        Self {
            config: PluginConfig::default(),
            visual_config: WyckoffVisualConfig::default(),
            analyzer: WyckoffAnalyzer::new(),
            candle_buffer: Vec::new(),
            drawing_ids: Vec::new(),
            detected_springs: Vec::new(),
            detected_upthrusts: Vec::new(),
        }
    }

    pub fn with_config(mut self, config: WyckoffVisualConfig) -> Self {
        self.visual_config = config;
        self
    }

    fn analyze_and_draw(&mut self, ctx: &mut DrawingContext) {
        if self.candle_buffer.len() < 50 {
            return;
        }

        let cfg = &self.visual_config;

        // Detect springs
        if cfg.show_springs {
            if let Some(spring) = self.analyzer.detect_spring(&self.candle_buffer) {
                if spring.strength >= cfg.min_strength {
                    let candle = &self.candle_buffer[spring.bar_index];
                    self.detected_springs.push((candle.time, spring));
                    self.draw_spring(ctx, candle, &spring);
                }
            }
        }

        // Detect upthrusts
        if cfg.show_upthrusts {
            if let Some(upthrust) = self.analyzer.detect_upthrust(&self.candle_buffer) {
                if upthrust.strength >= cfg.min_strength {
                    let candle = &self.candle_buffer[upthrust.bar_index];
                    self.detected_upthrusts.push((candle.time, upthrust));
                    self.draw_upthrust(ctx, candle, &upthrust);
                }
            }
        }

        // Draw phase background
        if cfg.show_phases {
            self.draw_phase(ctx);
        }
    }

    fn draw_spring(&mut self, ctx: &mut DrawingContext, candle: &Candle, spring: &Spring) {
        let cfg = &self.visual_config;

        // Draw arrow up at spring low
        let id = ctx.draw(
            Icon::new(
                Point::new(candle.time, spring.low),
                IconType::ArrowUp,
            )
            .color(cfg.spring_color)
            .size(20.0)
            .tooltip(format!("Spring ({}%)", spring.strength as u32))
        );
        self.drawing_ids.push(id);

        // Draw support level that was tested
        let id = ctx.draw(
            HorizontalLine::from_to(
                spring.support,
                candle.time - 86400000 * 5, // 5 days back
                candle.time,
            )
            .color(cfg.spring_color.with_alpha(0.5))
            .style(LineStyle::Dashed)
            .label("Support")
        );
        self.drawing_ids.push(id);

        // Draw zone showing the spring area
        let id = ctx.draw(
            PriceZone::from_to(
                spring.support,
                spring.low,
                candle.time - 86400000,
                candle.time + 86400000,
            )
            .fill(cfg.spring_color.with_alpha(0.2))
            .label("Spring Zone")
        );
        self.drawing_ids.push(id);

        // Label
        let id = ctx.draw(
            Label::new(
                format!("Spring {:.0}%", spring.strength),
                candle.time,
                spring.low,
            )
            .color(cfg.spring_color)
            .anchor(Anchor::TopCenter)
            .offset(0.0, 10.0)
        );
        self.drawing_ids.push(id);
    }

    fn draw_upthrust(&mut self, ctx: &mut DrawingContext, candle: &Candle, upthrust: &Upthrust) {
        let cfg = &self.visual_config;

        // Draw arrow down at upthrust high
        let id = ctx.draw(
            Icon::new(
                Point::new(candle.time, upthrust.high),
                IconType::ArrowDown,
            )
            .color(cfg.upthrust_color)
            .size(20.0)
            .tooltip(format!("Upthrust ({}%)", upthrust.strength as u32))
        );
        self.drawing_ids.push(id);

        // Draw resistance level
        let id = ctx.draw(
            HorizontalLine::from_to(
                upthrust.resistance,
                candle.time - 86400000 * 5,
                candle.time,
            )
            .color(cfg.upthrust_color.with_alpha(0.5))
            .style(LineStyle::Dashed)
            .label("Resistance")
        );
        self.drawing_ids.push(id);

        // Draw zone
        let id = ctx.draw(
            PriceZone::from_to(
                upthrust.high,
                upthrust.resistance,
                candle.time - 86400000,
                candle.time + 86400000,
            )
            .fill(cfg.upthrust_color.with_alpha(0.2))
            .label("Upthrust Zone")
        );
        self.drawing_ids.push(id);

        // Label
        let id = ctx.draw(
            Label::new(
                format!("Upthrust {:.0}%", upthrust.strength),
                candle.time,
                upthrust.high,
            )
            .color(cfg.upthrust_color)
            .anchor(Anchor::BottomCenter)
            .offset(0.0, -10.0)
        );
        self.drawing_ids.push(id);
    }

    fn draw_phase(&mut self, ctx: &mut DrawingContext) {
        let phase = self.analyzer.analyze_phase(&self.candle_buffer);
        let cfg = &self.visual_config;

        let (color, label) = match phase {
            WyckoffPhase::Accumulation => (cfg.accumulation_color, "Accumulation"),
            WyckoffPhase::Distribution => (cfg.distribution_color, "Distribution"),
            WyckoffPhase::Markup => (Color::GREEN.with_alpha(0.05), "Markup"),
            WyckoffPhase::Markdown => (Color::RED.with_alpha(0.05), "Markdown"),
            WyckoffPhase::Unknown => return,
        };

        // Get time range of current data
        if let (Some(first), Some(last)) = (self.candle_buffer.first(), self.candle_buffer.last()) {
            let high = self.candle_buffer.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
            let low = self.candle_buffer.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);

            // Background zone
            let id = ctx.draw_on_layer(
                PriceZone::from_to(high, low, first.time, last.time)
                    .fill(color)
                    .label(label),
                LayerId::BACKGROUND,
            );
            self.drawing_ids.push(id);

            // Phase label
            let id = ctx.draw(
                Label::new(
                    format!("Wyckoff: {}", label),
                    last.time,
                    high,
                )
                .anchor(Anchor::TopRight)
                .background(Color::BLACK.with_alpha(0.7))
            );
            self.drawing_ids.push(id);
        }
    }
}

impl Default for WyckoffPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartPlugin for WyckoffPlugin {
    fn name(&self) -> &str {
        "Wyckoff Analysis"
    }

    fn description(&self) -> &str {
        "Visualizes Wyckoff patterns including Springs, Upthrusts, and market phases"
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
        self.analyze_and_draw(ctx);
    }

    fn on_candle(&mut self, ctx: &mut DrawingContext, candle: &Candle) {
        self.candle_buffer.push(candle.clone());

        // Keep buffer manageable
        if self.candle_buffer.len() > 500 {
            self.candle_buffer.remove(0);
        }

        // Re-analyze on each candle
        self.analyze_and_draw(ctx);
    }
}
