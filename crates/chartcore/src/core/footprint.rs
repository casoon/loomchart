// Footprint (Delta) Candle types and renderer.
//
// A footprint candle breaks each OHLCV candle into price-level buckets,
// showing bid vs. ask volume (aggressor side) at each level.
//
// Delta = total ask volume − total bid volume for the candle.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// A single price-level bucket inside a footprint candle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootprintLevel {
    pub price: f64,
    #[serde(alias = "sell_volume")]
    pub bid_volume: f64,
    #[serde(alias = "buy_volume")]
    pub ask_volume: f64,
}

/// A footprint candle augments a normal OHLCV candle with tick-level volume data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootprintCandle {
    /// Unix timestamp in seconds (candle open time)
    #[serde(alias = "timestamp")]
    pub time: i64,
    #[serde(alias = "open")]
    pub o: f64,
    #[serde(alias = "high")]
    pub h: f64,
    #[serde(alias = "low")]
    pub l: f64,
    #[serde(alias = "close")]
    pub c: f64,
    #[serde(default, alias = "volume")]
    pub v: f64,
    /// Price levels sorted ascending
    pub levels: Vec<FootprintLevel>,
}

impl FootprintCandle {
    /// True when more buying than selling pressure.
    pub fn is_positive_delta(&self) -> bool {
        self.delta() >= 0.0
    }

    /// Total ask volume minus bid volume.
    pub fn delta(&self) -> f64 {
        self.levels
            .iter()
            .map(|level| level.ask_volume - level.bid_volume)
            .sum()
    }

    /// Maximum volume across all levels (for normalising bar widths).
    pub fn max_level_volume(&self) -> f64 {
        self.levels
            .iter()
            .map(|l| l.ask_volume.max(l.bid_volume))
            .fold(0.0_f64, f64::max)
    }

    /// Maximum total volume level, used for POC highlighting.
    pub fn poc_price(&self) -> Option<f64> {
        self.levels
            .iter()
            .max_by(|a, b| {
                let av = a.ask_volume + a.bid_volume;
                let bv = b.ask_volume + b.bid_volume;
                av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|level| level.price)
    }

    pub fn level_delta(level: &FootprintLevel) -> f64 {
        level.ask_volume - level.bid_volume
    }

    pub fn max_total_level_volume(&self) -> f64 {
        self.levels
            .iter()
            .map(|l| l.ask_volume + l.bid_volume)
            .fold(0.0_f64, f64::max)
    }
}

// ---------------------------------------------------------------------------
// Renderer
// ---------------------------------------------------------------------------

use crate::primitives::Color;
use crate::rendering::{DrawStyle, RenderCommand, TextAlign, TextBaseline};

/// Configuration for footprint rendering.
#[derive(Debug, Clone)]
pub struct FootprintConfig {
    /// Minimum height in pixels for a price-level row below which we skip rendering.
    pub min_row_height_px: f64,
    /// Width fraction of one candle slot for volume bars (0–1).
    pub bar_width_fraction: f64,
    /// Show delta column below each candle.
    pub show_delta: bool,
    pub ask_color: Color,
    pub bid_color: Color,
    pub delta_positive_color: Color,
    pub delta_negative_color: Color,
    /// Highlight rows where buy/sell ratio exceeds this value.
    pub imbalance_threshold: f64,
    pub imbalance_color: Color,
}

impl Default for FootprintConfig {
    fn default() -> Self {
        Self {
            min_row_height_px: 8.0,
            bar_width_fraction: 0.9,
            show_delta: true,
            ask_color: Color::rgba(0, 200, 120, 0.7),
            bid_color: Color::rgba(220, 60, 60, 0.7),
            delta_positive_color: Color::rgba(0, 220, 140, 1.0),
            delta_negative_color: Color::rgba(240, 80, 80, 1.0),
            imbalance_threshold: 3.0,
            imbalance_color: Color::rgba(255, 200, 0, 0.25),
        }
    }
}

/// Converts a slice of `FootprintCandle` into `RenderCommand`s.
pub struct FootprintRenderer<'a> {
    config: &'a FootprintConfig,
}

impl<'a> FootprintRenderer<'a> {
    pub fn new(config: &'a FootprintConfig) -> Self {
        Self { config }
    }

    /// Generate render commands for visible footprint candles.
    ///
    /// - `first_visible` / `last_visible`: index range into `candles`
    /// - `x_for_index`: returns x pixel coordinate for a candle index
    /// - `candle_width`: width in pixels of one candle slot
    /// - `y_for_price`: converts a price to a y pixel coordinate
    /// - `price_per_pixel`: how many price units fit in one pixel (zoom level)
    pub fn render(
        &self,
        candles: &[FootprintCandle],
        first_visible: usize,
        last_visible: usize,
        x_for_index: impl Fn(usize) -> f64,
        candle_width: f64,
        y_for_price: impl Fn(f64) -> f64,
        price_per_pixel: f64,
    ) -> Vec<RenderCommand> {
        let mut cmds = Vec::new();

        // Not enough space to show footprint data — bail out.
        if candle_width < 8.0 || price_per_pixel <= 0.0 {
            return cmds;
        }

        let end = last_visible.min(candles.len().saturating_sub(1));
        if first_visible > end {
            return cmds;
        }

        let visible = &candles[first_visible..=end];

        for (rel_idx, candle) in visible.iter().enumerate() {
            let abs_idx = first_visible + rel_idx;
            let cx = x_for_index(abs_idx);
            let slot_w = candle_width * self.config.bar_width_fraction;
            let half_slot = slot_w / 2.0;

            let max_vol = candle.max_level_volume().max(1.0);

            // Compute row height from adjacent levels, or fall back to minimum.
            let row_height_px = if candle.levels.len() >= 2 {
                let dp = (candle.levels[1].price - candle.levels[0].price).abs();
                (dp / price_per_pixel).max(self.config.min_row_height_px)
            } else {
                self.config.min_row_height_px
            };

            if row_height_px < 2.0 {
                continue;
            }

            for level in &candle.levels {
                let y_center = y_for_price(level.price);
                let half_h = row_height_px / 2.0;
                let y_top = y_center - half_h;

                // Imbalance highlight background
                if self.config.imbalance_threshold > 0.0 {
                    let ratio = if level.bid_volume > 0.0 {
                        level.ask_volume / level.bid_volume
                    } else {
                        f64::INFINITY
                    };
                    let imbalanced = ratio >= self.config.imbalance_threshold
                        || (level.bid_volume > 0.0
                            && (1.0 / ratio) >= self.config.imbalance_threshold);
                    if imbalanced {
                        cmds.push(RenderCommand::Rect {
                            x: cx - half_slot,
                            y: y_top,
                            width: slot_w,
                            height: row_height_px,
                            style: DrawStyle {
                                fill_color: Some(self.config.imbalance_color),
                                stroke_color: None,
                                line_width: 0.0,
                            },
                        });
                    }
                }

                // Bid volume bar (left of centre)
                if level.bid_volume > 0.0 {
                    let bar_w = (level.bid_volume / max_vol) * half_slot;
                    cmds.push(RenderCommand::Rect {
                        x: cx - bar_w,
                        y: y_top + 1.0,
                        width: bar_w,
                        height: (row_height_px - 2.0).max(1.0),
                        style: DrawStyle {
                            fill_color: Some(self.config.bid_color),
                            stroke_color: None,
                            line_width: 0.0,
                        },
                    });
                }

                // Ask volume bar (right of centre)
                if level.ask_volume > 0.0 {
                    let bar_w = (level.ask_volume / max_vol) * half_slot;
                    cmds.push(RenderCommand::Rect {
                        x: cx,
                        y: y_top + 1.0,
                        width: bar_w,
                        height: (row_height_px - 2.0).max(1.0),
                        style: DrawStyle {
                            fill_color: Some(self.config.ask_color),
                            stroke_color: None,
                            line_width: 0.0,
                        },
                    });
                }
            }

            // Delta label below the candle's low
            if self.config.show_delta {
                let delta_color = if candle.is_positive_delta() {
                    self.config.delta_positive_color
                } else {
                    self.config.delta_negative_color
                };
                let delta = candle.delta();
                let sign = if delta >= 0.0 { "+" } else { "" };
                cmds.push(RenderCommand::Text {
                    text: format!("{}{:.0}", sign, delta),
                    x: cx,
                    y: y_for_price(candle.l) + 12.0,
                    color: delta_color,
                    size: 9.0,
                    align: TextAlign::Center,
                    baseline: TextBaseline::Top,
                });
            }
        }

        cmds
    }
}
