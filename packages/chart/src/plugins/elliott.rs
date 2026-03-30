//! Elliott Wave Visualization Plugin.
//!
//! Visualizes Elliott Wave patterns with wave labels and Fibonacci projections.

use loom_core::{Candle, OHLCV, Price, Timestamp};
use loom_signals::elliott::{ElliottAnalyzer, WaveCount, WaveDegree, Wave};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::{
    primitives::*,
    plugin::{ChartPlugin, PluginConfig},
    context::{DrawingContext, LayerId},
};

/// Elliott Wave visualization configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ElliottVisualConfig {
    /// Wave degree to analyze
    pub degree: WaveDegree,
    /// Show wave labels
    pub show_labels: bool,
    /// Show wave connectors
    pub show_connectors: bool,
    /// Show Fibonacci projections
    pub show_projections: bool,
    /// Show targets
    pub show_targets: bool,
    /// Minimum confidence to display
    pub min_confidence: f64,
    /// Colors
    pub impulse_color: Color,
    pub corrective_color: Color,
    pub label_color: Color,
    pub projection_color: Color,
}

impl Default for ElliottVisualConfig {
    fn default() -> Self {
        Self {
            degree: WaveDegree::Minor,
            show_labels: true,
            show_connectors: true,
            show_projections: true,
            show_targets: true,
            min_confidence: 50.0,
            impulse_color: Color::BLUE,
            corrective_color: Color::ORANGE,
            label_color: Color::WHITE,
            projection_color: Color::YELLOW,
        }
    }
}

/// Elliott Wave Visualization Plugin
pub struct ElliottPlugin {
    config: PluginConfig,
    visual_config: ElliottVisualConfig,
    analyzer: ElliottAnalyzer,
    candle_buffer: Vec<Candle>,
    drawing_ids: Vec<DrawingId>,
    current_count: Option<WaveCount>,
}

impl ElliottPlugin {
    pub fn new() -> Self {
        Self {
            config: PluginConfig::default(),
            visual_config: ElliottVisualConfig::default(),
            analyzer: ElliottAnalyzer::new(),
            candle_buffer: Vec::new(),
            drawing_ids: Vec::new(),
            current_count: None,
        }
    }

    pub fn with_degree(mut self, degree: WaveDegree) -> Self {
        self.visual_config.degree = degree;
        self
    }

    fn clear_drawings(&mut self, ctx: &mut DrawingContext) {
        for id in self.drawing_ids.drain(..) {
            ctx.remove(id);
        }
    }

    fn analyze_and_draw(&mut self, ctx: &mut DrawingContext) {
        if self.candle_buffer.len() < 50 {
            return;
        }

        self.clear_drawings(ctx);

        // Analyze for wave patterns
        let wave_count = self.analyzer.analyze(&self.candle_buffer, self.visual_config.degree);

        if let Some(ref count) = wave_count {
            if count.confidence >= self.visual_config.min_confidence {
                self.current_count = wave_count.clone();
                self.draw_wave_count(ctx, count);
            }
        }
    }

    fn draw_wave_count(&mut self, ctx: &mut DrawingContext, count: &WaveCount) {
        let cfg = &self.visual_config;

        // Draw connectors between waves
        if cfg.show_connectors && count.waves.len() >= 2 {
            for window in count.waves.windows(2) {
                let from_wave = &window[0];
                let to_wave = &window[1];

                let color = if from_wave.wave_type == loom_signals::elliott::WaveType::Impulse {
                    cfg.impulse_color
                } else {
                    cfg.corrective_color
                };

                let id = ctx.draw(
                    Line::new(
                        Point::new(from_wave.end_time, from_wave.end_price),
                        Point::new(to_wave.end_time, to_wave.end_price),
                    )
                    .color(color)
                    .width(2.0)
                );
                self.drawing_ids.push(id);
            }
        }

        // Draw wave labels
        if cfg.show_labels {
            for wave in &count.waves {
                self.draw_wave_label(ctx, wave);
            }
        }

        // Draw projections
        if cfg.show_projections {
            self.draw_projections(ctx, count);
        }

        // Draw targets
        if cfg.show_targets {
            self.draw_targets(ctx, count);
        }

        // Draw confidence indicator
        let last_time = self.candle_buffer.last().map(|c| c.time).unwrap_or(0);
        let high = self.candle_buffer.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);

        let id = ctx.draw(
            Label::new(
                format!("Elliott: {:.0}% conf", count.confidence),
                last_time,
                high,
            )
            .anchor(Anchor::TopRight)
            .background(Color::BLACK.with_alpha(0.7))
            .color(if count.trend_up { Color::GREEN } else { Color::RED })
        );
        self.drawing_ids.push(id);
    }

    fn draw_wave_label(&mut self, ctx: &mut DrawingContext, wave: &Wave) {
        let cfg = &self.visual_config;

        // Position label at wave end
        let label_text = &wave.label;
        let is_impulse = wave.wave_type == loom_signals::elliott::WaveType::Impulse;

        // Circle background for wave number
        let id = ctx.draw(
            Icon::new(
                Point::new(wave.end_time, wave.end_price),
                IconType::CircleFilled,
            )
            .color(if is_impulse { cfg.impulse_color } else { cfg.corrective_color })
            .size(20.0)
        );
        self.drawing_ids.push(id);

        // Wave label text
        let anchor = if wave.is_up {
            Anchor::BottomCenter
        } else {
            Anchor::TopCenter
        };

        let id = ctx.draw(
            Label::new(label_text.clone(), wave.end_time, wave.end_price)
                .color(cfg.label_color)
                .font_size(14.0)
                .anchor(anchor)
        );
        self.drawing_ids.push(id);
    }

    fn draw_projections(&mut self, ctx: &mut DrawingContext, count: &WaveCount) {
        let cfg = &self.visual_config;

        // Draw Fibonacci retracement for corrective waves
        for wave in &count.waves {
            if wave.wave_type == loom_signals::elliott::WaveType::Corrective {
                let id = ctx.draw_on_layer(
                    FibonacciRetracement::new(
                        Point::new(wave.start_time, wave.start_price),
                        Point::new(wave.end_time, wave.end_price),
                    ),
                    LayerId::DRAWINGS,
                );
                self.drawing_ids.push(id);
            }
        }
    }

    fn draw_targets(&mut self, ctx: &mut DrawingContext, count: &WaveCount) {
        let cfg = &self.visual_config;

        let targets = self.analyzer.project_targets(count);

        for (label, price) in targets {
            let last_time = self.candle_buffer.last().map(|c| c.time).unwrap_or(0);

            let id = ctx.draw(
                HorizontalLine::from_to(price, last_time, last_time + 86400000 * 10)
                    .color(cfg.projection_color)
                    .style(LineStyle::Dashed)
                    .label(format!("{}: {:.2}", label, price))
            );
            self.drawing_ids.push(id);

            // Target marker
            let id = ctx.draw(
                Icon::new(
                    Point::new(last_time + 86400000 * 5, price),
                    IconType::Diamond,
                )
                .color(cfg.projection_color)
                .size(12.0)
                .tooltip(label)
            );
            self.drawing_ids.push(id);
        }
    }
}

impl Default for ElliottPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartPlugin for ElliottPlugin {
    fn name(&self) -> &str {
        "Elliott Waves"
    }

    fn description(&self) -> &str {
        "Visualizes Elliott Wave patterns with labels and Fibonacci projections"
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

        if self.candle_buffer.len() > 500 {
            self.candle_buffer.remove(0);
        }

        // Only re-analyze every 5 candles to reduce computation
        if self.candle_buffer.len() % 5 == 0 {
            self.analyze_and_draw(ctx);
        }
    }
}
