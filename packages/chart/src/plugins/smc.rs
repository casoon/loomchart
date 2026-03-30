//! Smart Money Concepts (SMC) Visualization Plugin.
//!
//! Visualizes Order Blocks, Fair Value Gaps, Liquidity zones, and BOS/CHoCH.

use loom_core::{Candle, OHLCV, Price, Timestamp};
use loom_signals::smc::{
    SmcAnalyzer, OrderBlock, FairValueGap, LiquidityZone,
    BreakOfStructure, MarketStructure, Swing,
};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec};

use crate::{
    primitives::*,
    plugin::{ChartPlugin, PluginConfig},
    context::{DrawingContext, LayerId},
};

/// SMC visualization configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SmcVisualConfig {
    /// Show order blocks
    pub show_order_blocks: bool,
    /// Show fair value gaps
    pub show_fvg: bool,
    /// Show liquidity zones
    pub show_liquidity: bool,
    /// Show break of structure
    pub show_bos: bool,
    /// Show swing points
    pub show_swings: bool,
    /// Show premium/discount zones
    pub show_premium_discount: bool,
    /// Only show unfilled FVGs
    pub fvg_unfilled_only: bool,
    /// Only show unmitigated OBs
    pub ob_unmitigated_only: bool,
    /// Minimum OB strength
    pub min_ob_strength: f64,
    /// Colors
    pub bullish_ob_color: Color,
    pub bearish_ob_color: Color,
    pub bullish_fvg_color: Color,
    pub bearish_fvg_color: Color,
    pub buy_liquidity_color: Color,
    pub sell_liquidity_color: Color,
    pub bos_bullish_color: Color,
    pub bos_bearish_color: Color,
    pub choch_color: Color,
    pub premium_color: Color,
    pub discount_color: Color,
}

impl Default for SmcVisualConfig {
    fn default() -> Self {
        Self {
            show_order_blocks: true,
            show_fvg: true,
            show_liquidity: true,
            show_bos: true,
            show_swings: true,
            show_premium_discount: false,
            fvg_unfilled_only: true,
            ob_unmitigated_only: true,
            min_ob_strength: 40.0,
            bullish_ob_color: Color::rgba(38, 166, 154, 0.3),
            bearish_ob_color: Color::rgba(239, 83, 80, 0.3),
            bullish_fvg_color: Color::rgba(0, 188, 212, 0.2),
            bearish_fvg_color: Color::rgba(255, 152, 0, 0.2),
            buy_liquidity_color: Color::GREEN,
            sell_liquidity_color: Color::RED,
            bos_bullish_color: Color::GREEN,
            bos_bearish_color: Color::RED,
            choch_color: Color::YELLOW,
            premium_color: Color::rgba(239, 83, 80, 0.1),
            discount_color: Color::rgba(38, 166, 154, 0.1),
        }
    }
}

/// SMC Visualization Plugin
pub struct SmcPlugin {
    config: PluginConfig,
    visual_config: SmcVisualConfig,
    analyzer: SmcAnalyzer,
    candle_buffer: Vec<Candle>,
    drawing_ids: Vec<DrawingId>,
}

impl SmcPlugin {
    pub fn new() -> Self {
        Self {
            config: PluginConfig::default(),
            visual_config: SmcVisualConfig::default(),
            analyzer: SmcAnalyzer::new(),
            candle_buffer: Vec::new(),
            drawing_ids: Vec::new(),
        }
    }

    pub fn with_config(mut self, config: SmcVisualConfig) -> Self {
        self.visual_config = config;
        self
    }

    fn clear_drawings(&mut self, ctx: &mut DrawingContext) {
        for id in self.drawing_ids.drain(..) {
            ctx.remove(id);
        }
    }

    fn analyze_and_draw(&mut self, ctx: &mut DrawingContext) {
        if self.candle_buffer.len() < 20 {
            return;
        }

        self.clear_drawings(ctx);
        let cfg = &self.visual_config;

        // Draw Order Blocks
        if cfg.show_order_blocks {
            self.draw_order_blocks(ctx);
        }

        // Draw Fair Value Gaps
        if cfg.show_fvg {
            self.draw_fvgs(ctx);
        }

        // Draw Liquidity Zones
        if cfg.show_liquidity {
            self.draw_liquidity(ctx);
        }

        // Draw Break of Structure
        if cfg.show_bos {
            self.draw_bos(ctx);
        }

        // Draw Swing Points
        if cfg.show_swings {
            self.draw_swings(ctx);
        }

        // Draw Premium/Discount
        if cfg.show_premium_discount {
            self.draw_premium_discount(ctx);
        }
    }

    fn draw_order_blocks(&mut self, ctx: &mut DrawingContext) {
        let cfg = &self.visual_config;
        let obs = self.analyzer.find_order_blocks(&self.candle_buffer);

        for ob in obs {
            if cfg.ob_unmitigated_only && ob.mitigated {
                continue;
            }
            if ob.strength < cfg.min_ob_strength {
                continue;
            }

            let color = if ob.is_bullish {
                cfg.bullish_ob_color
            } else {
                cfg.bearish_ob_color
            };

            let candle = &self.candle_buffer[ob.index];
            let end_time = self.candle_buffer.last().map(|c| c.time).unwrap_or(ob.time);

            // Draw OB zone
            let id = ctx.draw_on_layer(
                PriceZone::from_to(ob.top, ob.bottom, ob.time, end_time)
                    .fill(color)
                    .border(color.with_alpha(0.8))
                    .label(if ob.is_bullish { "Bullish OB" } else { "Bearish OB" }),
                LayerId::ZONES,
            );
            self.drawing_ids.push(id);

            // Draw midline (often respected)
            let mid = (ob.top + ob.bottom) / 2.0;
            let id = ctx.draw(
                HorizontalLine::from_to(mid, ob.time, end_time)
                    .color(color.with_alpha(0.5))
                    .style(LineStyle::Dotted)
            );
            self.drawing_ids.push(id);
        }
    }

    fn draw_fvgs(&mut self, ctx: &mut DrawingContext) {
        let cfg = &self.visual_config;
        let fvgs = self.analyzer.find_fvgs(&self.candle_buffer);

        for fvg in fvgs {
            if cfg.fvg_unfilled_only && fvg.filled {
                continue;
            }

            let color = if fvg.is_bullish {
                cfg.bullish_fvg_color
            } else {
                cfg.bearish_fvg_color
            };

            let candle = &self.candle_buffer[fvg.index];
            let end_time = self.candle_buffer.last().map(|c| c.time).unwrap_or(fvg.time);

            // Draw FVG zone
            let id = ctx.draw_on_layer(
                PriceZone::from_to(fvg.top, fvg.bottom, fvg.time, end_time)
                    .fill(color)
                    .label(format!(
                        "FVG {}%",
                        if fvg.filled { "Filled" } else { &format!("{:.0}", fvg.fill_percent) }
                    )),
                LayerId::ZONES,
            );
            self.drawing_ids.push(id);

            // Draw equilibrium (50% level)
            let eq = fvg.equilibrium();
            let id = ctx.draw(
                HorizontalLine::from_to(eq, fvg.time, end_time)
                    .color(color.with_alpha(0.7))
                    .style(LineStyle::Dashed)
            );
            self.drawing_ids.push(id);
        }
    }

    fn draw_liquidity(&mut self, ctx: &mut DrawingContext) {
        let cfg = &self.visual_config;
        let zones = self.analyzer.find_liquidity_zones(&self.candle_buffer);

        for zone in zones {
            if zone.swept {
                continue; // Don't show swept liquidity
            }

            let color = if zone.is_buy_side {
                cfg.buy_liquidity_color
            } else {
                cfg.sell_liquidity_color
            };

            let end_time = self.candle_buffer.last().map(|c| c.time).unwrap_or(0);
            let start_time = end_time - 86400000 * 30; // 30 days back

            // Draw liquidity level
            let id = ctx.draw(
                HorizontalLine::from_to(zone.level, start_time, end_time)
                    .color(color)
                    .style(LineStyle::Dotted)
                    .width(2.0)
                    .label(format!(
                        "{}x {}",
                        zone.touches,
                        if zone.is_buy_side { "BSL" } else { "SSL" }
                    ))
            );
            self.drawing_ids.push(id);

            // Draw $ markers for liquidity
            let id = ctx.draw(
                Label::new("$$$", end_time, zone.level)
                    .color(color)
                    .font_size(10.0)
                    .anchor(Anchor::MiddleRight)
            );
            self.drawing_ids.push(id);
        }
    }

    fn draw_bos(&mut self, ctx: &mut DrawingContext) {
        let cfg = &self.visual_config;
        let bos_list = self.analyzer.find_bos(&self.candle_buffer);

        for bos in bos_list {
            let color = if bos.is_choch {
                cfg.choch_color
            } else if bos.is_bullish {
                cfg.bos_bullish_color
            } else {
                cfg.bos_bearish_color
            };

            let label = if bos.is_choch {
                "CHoCH"
            } else if bos.is_bullish {
                "BOS ↑"
            } else {
                "BOS ↓"
            };

            // Draw level
            let end_time = bos.time + 86400000 * 5;
            let id = ctx.draw(
                HorizontalLine::from_to(bos.level, bos.time - 86400000, end_time)
                    .color(color)
                    .style(LineStyle::Dashed)
                    .width(2.0)
                    .label(label)
            );
            self.drawing_ids.push(id);

            // Draw break marker
            let id = ctx.draw(
                Icon::new(
                    Point::new(bos.time, bos.level),
                    if bos.is_bullish { IconType::TriangleUp } else { IconType::TriangleDown },
                )
                .color(color)
                .size(16.0)
                .tooltip(String::from(label))
            );
            self.drawing_ids.push(id);
        }
    }

    fn draw_swings(&mut self, ctx: &mut DrawingContext) {
        let swings = self.analyzer.find_swings(&self.candle_buffer);

        for swing in &swings {
            let (color, icon) = if swing.is_high {
                (Color::GRAY, IconType::TriangleDown)
            } else {
                (Color::GRAY, IconType::TriangleUp)
            };

            let id = ctx.draw(
                Icon::new(Point::new(swing.time, swing.price), icon)
                    .color(color.with_alpha(0.5))
                    .size(8.0)
            );
            self.drawing_ids.push(id);
        }

        // Connect swings with lines (market structure)
        if swings.len() >= 2 {
            for window in swings.windows(2) {
                let from = &window[0];
                let to = &window[1];

                let color = if to.price > from.price {
                    Color::GREEN.with_alpha(0.3)
                } else {
                    Color::RED.with_alpha(0.3)
                };

                let id = ctx.draw(
                    Line::new(
                        Point::new(from.time, from.price),
                        Point::new(to.time, to.price),
                    )
                    .color(color)
                    .style(LineStyle::Dotted)
                );
                self.drawing_ids.push(id);
            }
        }
    }

    fn draw_premium_discount(&mut self, ctx: &mut DrawingContext) {
        let cfg = &self.visual_config;

        if let Some((discount_top, eq, premium_bottom)) =
            self.analyzer.premium_discount_zones(&self.candle_buffer, 50)
        {
            let start = self.candle_buffer.first().map(|c| c.time).unwrap_or(0);
            let end = self.candle_buffer.last().map(|c| c.time).unwrap_or(0);

            let high = self.candle_buffer.iter().map(|c| c.high).fold(f64::NEG_INFINITY, f64::max);
            let low = self.candle_buffer.iter().map(|c| c.low).fold(f64::INFINITY, f64::min);

            // Premium zone (above equilibrium)
            let id = ctx.draw_on_layer(
                PriceZone::from_to(high, eq, start, end)
                    .fill(cfg.premium_color)
                    .label("Premium"),
                LayerId::BACKGROUND,
            );
            self.drawing_ids.push(id);

            // Discount zone (below equilibrium)
            let id = ctx.draw_on_layer(
                PriceZone::from_to(eq, low, start, end)
                    .fill(cfg.discount_color)
                    .label("Discount"),
                LayerId::BACKGROUND,
            );
            self.drawing_ids.push(id);

            // Equilibrium line
            let id = ctx.draw(
                HorizontalLine::from_to(eq, start, end)
                    .color(Color::YELLOW)
                    .style(LineStyle::Dashed)
                    .label("Equilibrium")
            );
            self.drawing_ids.push(id);
        }
    }
}

impl Default for SmcPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartPlugin for SmcPlugin {
    fn name(&self) -> &str {
        "Smart Money Concepts"
    }

    fn description(&self) -> &str {
        "Visualizes SMC patterns: Order Blocks, FVGs, Liquidity, BOS/CHoCH"
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

        self.analyze_and_draw(ctx);
    }
}
