/// Volume pane rendering support
///
/// Provides a separate pane below the main chart for volume visualization.
/// Features:
/// - Separate viewport with auto-scaling
/// - Color-coded bars (bullish/bearish)
/// - Configurable opacity
/// - Smooth resize and layout
use super::types::Candle;
use crate::primitives::Color;
use crate::renderers::commands::{RenderCommand, RenderCommandBuffer};

/// Configuration for volume pane rendering
#[derive(Debug, Clone)]
pub struct VolumePaneConfig {
    /// Height ratio relative to chart (0.0 to 1.0)
    /// Default: 0.15 (15% of total height)
    pub height_ratio: f64,

    /// Bar width relative to candle width (0.0 to 1.0)
    /// Default: 0.8 (80% of candle width)
    pub bar_width_ratio: f64,

    /// Bullish bar color
    pub bullish_color: Color,

    /// Bearish bar color
    pub bearish_color: Color,

    /// Bar opacity (0.0 to 1.0)
    /// Default: 0.5 for semi-transparent
    pub opacity: f64,

    /// Show volume moving average line
    pub show_ma: bool,

    /// Period for volume MA (if show_ma is true)
    pub ma_period: usize,

    /// Color for volume MA line
    pub ma_color: Color,
}

impl Default for VolumePaneConfig {
    fn default() -> Self {
        Self {
            height_ratio: 0.15,
            bar_width_ratio: 0.8,
            bullish_color: Color::rgba(76, 175, 80, 0.5), // Green with alpha
            bearish_color: Color::rgba(244, 67, 54, 0.5), // Red with alpha
            opacity: 0.5,
            show_ma: false,
            ma_period: 20,
            ma_color: Color::rgba(33, 150, 243, 0.7), // Blue with alpha
        }
    }
}

/// Volume pane viewport and state
pub struct VolumePane {
    config: VolumePaneConfig,

    /// X position of pane
    pub x: f64,

    /// Y position of pane (relative to container)
    pub y: f64,

    /// Width of pane
    pub width: f64,

    /// Height of pane
    pub height: f64,

    /// Current volume scale (min, max)
    volume_range: (f64, f64),
}

impl VolumePane {
    /// Create a new volume pane
    pub fn new(config: VolumePaneConfig) -> Self {
        Self {
            config,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            volume_range: (0.0, 0.0),
        }
    }

    /// Update layout based on container dimensions
    pub fn update_layout(
        &mut self,
        container_width: f64,
        container_height: f64,
        chart_height: f64,
    ) {
        self.x = 0.0;
        self.y = chart_height;
        self.width = container_width;
        self.height = container_height - chart_height;
    }

    /// Calculate volume range for visible candles
    pub fn calculate_volume_range(&mut self, candles: &[Candle]) {
        if candles.is_empty() {
            self.volume_range = (0.0, 0.0);
            return;
        }

        let max_volume = candles.iter().map(|c| c.v).fold(0.0f64, |a, b| a.max(b));

        // Add 10% padding at top
        self.volume_range = (0.0, max_volume * 1.1);
    }

    /// Render volume bars to command buffer
    pub fn render(
        &self,
        candles: &[Candle],
        bar_width: f64,
        index_to_x: impl Fn(usize) -> f64,
        buffer: &mut RenderCommandBuffer,
    ) {
        if candles.is_empty() || self.height <= 0.0 {
            return;
        }

        let (min_vol, max_vol) = self.volume_range;
        if max_vol <= 0.0 {
            return;
        }

        // Draw separator line at top of volume pane
        buffer.push(RenderCommand::DrawLine {
            x1: self.x,
            y1: self.y,
            x2: self.x + self.width,
            y2: self.y,
            color: Color::rgba(128, 128, 128, 0.3),
            width: 1.0,
        });

        // Set clipping region
        buffer.push(RenderCommand::SetClip {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        });

        // Draw volume bars
        let bar_width_actual = bar_width * self.config.bar_width_ratio;

        for (i, candle) in candles.iter().enumerate() {
            let x = index_to_x(i);

            // Calculate bar height (scaled to pane height)
            let volume_ratio = (candle.v - min_vol) / (max_vol - min_vol);
            let bar_height = (volume_ratio * self.height * 0.95).max(1.0); // 95% to leave margin

            // Bar starts from bottom of pane
            let bar_y = self.y + self.height - bar_height;

            // Choose color based on candle direction
            let is_bullish = candle.c >= candle.o;
            let mut color = if is_bullish {
                self.config.bullish_color.clone()
            } else {
                self.config.bearish_color.clone()
            };

            // Apply opacity
            color = color.with_alpha(self.config.opacity as f32);

            // Draw bar
            buffer.push(RenderCommand::DrawRect {
                x: x - bar_width_actual / 2.0,
                y: bar_y,
                width: bar_width_actual,
                height: bar_height,
                fill: Some(color),
                stroke: None,
                stroke_width: 0.0,
            });
        }

        // Draw volume MA if enabled
        if self.config.show_ma && candles.len() >= self.config.ma_period {
            self.render_volume_ma(candles, index_to_x, buffer);
        }

        // Clear clipping
        buffer.push(RenderCommand::ClearClip);
    }

    /// Render volume moving average line
    fn render_volume_ma(
        &self,
        candles: &[Candle],
        index_to_x: impl Fn(usize) -> f64,
        buffer: &mut RenderCommandBuffer,
    ) {
        let period = self.config.ma_period;
        let (min_vol, max_vol) = self.volume_range;

        if max_vol <= 0.0 || candles.len() < period {
            return;
        }

        let mut points = Vec::new();

        for i in period - 1..candles.len() {
            // Calculate simple moving average
            let sum: f64 = candles[i - period + 1..=i].iter().map(|c| c.v).sum();
            let avg = sum / period as f64;

            // Convert to screen coordinates
            let x = index_to_x(i);
            let volume_ratio = (avg - min_vol) / (max_vol - min_vol);
            let y = self.y + self.height - (volume_ratio * self.height * 0.95);

            points.push((x, y));
        }

        if points.len() >= 2 {
            buffer.push(RenderCommand::DrawIndicatorLine {
                points,
                color: self.config.ma_color.clone(),
                width: 1.5,
                style: crate::renderers::commands::LineStyle::Solid,
            });
        }
    }

    /// Convert volume value to Y coordinate in pane
    pub fn volume_to_y(&self, v: f64) -> f64 {
        let (min_vol, max_vol) = self.volume_range;
        if max_vol <= min_vol {
            return self.y + self.height;
        }

        let volume_ratio = (v - min_vol) / (max_vol - min_vol);
        self.y + self.height - (volume_ratio * self.height * 0.95)
    }

    /// Convert Y coordinate to volume value
    pub fn y_to_volume(&self, y: f64) -> f64 {
        let (min_vol, max_vol) = self.volume_range;

        let y_ratio = (self.y + self.height - y) / (self.height * 0.95);
        min_vol + y_ratio * (max_vol - min_vol)
    }

    /// Get current configuration
    pub fn config(&self) -> &VolumePaneConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: VolumePaneConfig) {
        self.config = config;
    }

    /// Check if point is inside volume pane
    pub fn contains_point(&self, x: f64, y: f64) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_pane_creation() {
        let config = VolumePaneConfig::default();
        let pane = VolumePane::new(config);

        assert_eq!(pane.x, 0.0);
        assert_eq!(pane.y, 0.0);
    }

    #[test]
    fn test_volume_pane_layout() {
        let config = VolumePaneConfig::default();
        let mut pane = VolumePane::new(config);

        pane.update_layout(800.0, 600.0, 500.0);

        assert_eq!(pane.x, 0.0);
        assert_eq!(pane.y, 500.0);
        assert_eq!(pane.width, 800.0);
        assert_eq!(pane.height, 100.0);
    }

    #[test]
    fn test_volume_range_calculation() {
        let config = VolumePaneConfig::default();
        let mut pane = VolumePane::new(config);

        let candles = vec![
            Candle {
                time: 0,
                o: 100.0,
                h: 110.0,
                l: 95.0,
                c: 105.0,
                v: 1000.0,
            },
            Candle {
                time: 60,
                o: 105.0,
                h: 115.0,
                l: 100.0,
                c: 110.0,
                v: 1500.0,
            },
            Candle {
                time: 120,
                o: 110.0,
                h: 120.0,
                l: 105.0,
                c: 115.0,
                v: 2000.0,
            },
        ];

        pane.calculate_volume_range(&candles);

        assert_eq!(pane.volume_range.0, 0.0);
        assert_eq!(pane.volume_range.1, 2000.0 * 1.1); // Max volume + 10% padding
    }

    #[test]
    fn test_contains_point() {
        let config = VolumePaneConfig::default();
        let mut pane = VolumePane::new(config);
        pane.update_layout(800.0, 600.0, 500.0);

        assert!(pane.contains_point(400.0, 550.0)); // Center of pane
        assert!(!pane.contains_point(400.0, 400.0)); // Above pane
        assert!(!pane.contains_point(900.0, 550.0)); // Right of pane
    }

    #[test]
    fn test_volume_to_y_conversion() {
        let config = VolumePaneConfig::default();
        let mut pane = VolumePane::new(config);
        pane.update_layout(800.0, 600.0, 500.0);
        pane.volume_range = (0.0, 2000.0);

        // Max volume should be near top of pane
        let y_max = pane.volume_to_y(2000.0);
        assert!(y_max < pane.y + 10.0); // Near top

        // Zero volume should be at bottom
        let y_zero = pane.volume_to_y(0.0);
        assert!((y_zero - (pane.y + pane.height)).abs() < 1.0); // At bottom
    }
}
