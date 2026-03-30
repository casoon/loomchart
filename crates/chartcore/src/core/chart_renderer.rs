/// Chart rendering using command pattern
///
/// This module provides the main rendering logic that generates RenderCommands
/// instead of directly drawing to canvas. This approach has several benefits:
/// - Testable (commands are data, not side effects)
/// - Serializable (can be sent to WebWorker)
/// - Replayable (save/restore render frames)
/// - Optimizable (can batch/merge commands)
use super::indicator_renderer::IndicatorRenderer;
use super::invalidation::{InvalidationLevel, InvalidationMask};
use super::types::Candle;
use super::viewport::{PriceRange, TimeRange, Viewport};
use super::volume_pane::{VolumePane, VolumePaneConfig};
use crate::indicators::output::Indicator;
use crate::primitives::Color;
use crate::renderers::commands::{CandleData, RenderCommand, RenderCommandBuffer, TextAlign};

/// Chart renderer that generates render commands
pub struct ChartRenderer {
    /// Current frame counter
    frame_id: u64,

    /// Viewport for main chart area
    viewport: Viewport,

    /// Volume pane (optional)
    volume_pane: Option<VolumePane>,

    /// Active indicators
    indicators: Vec<Box<dyn Indicator>>,

    /// Invalidation tracking
    invalidation: InvalidationMask,

    /// Theme colors
    theme: ChartTheme,
}

/// Chart theme colors
#[derive(Debug, Clone)]
pub struct ChartTheme {
    pub background: Color,
    pub grid: Color,
    pub text: Color,
    pub bullish: Color,
    pub bearish: Color,
    pub crosshair: Color,
    pub axis_separator: Color,
}

impl Default for ChartTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl ChartTheme {
    /// Dark theme (default)
    pub fn dark() -> Self {
        Self {
            background: Color::rgb(17, 17, 17),
            grid: Color::rgba(128, 128, 128, 0.2),
            text: Color::rgba(255, 255, 255, 0.9),
            bullish: Color::rgb(76, 175, 80), // Green
            bearish: Color::rgb(244, 67, 54), // Red
            crosshair: Color::rgba(136, 136, 136, 0.6),
            axis_separator: Color::rgba(128, 128, 128, 0.3),
        }
    }

    /// Light theme
    pub fn light() -> Self {
        Self {
            background: Color::rgb(255, 255, 255),
            grid: Color::rgba(128, 128, 128, 0.1),
            text: Color::rgba(0, 0, 0, 0.87),
            bullish: Color::rgb(76, 175, 80),
            bearish: Color::rgb(244, 67, 54),
            crosshair: Color::rgba(96, 96, 96, 0.6),
            axis_separator: Color::rgba(128, 128, 128, 0.2),
        }
    }
}

impl ChartRenderer {
    /// Create a new chart renderer
    pub fn new(width: f64, height: f64) -> Self {
        let viewport = Viewport::new(width as u32, height as u32);

        Self {
            frame_id: 0,
            viewport,
            volume_pane: None,
            indicators: Vec::new(),
            invalidation: InvalidationMask::new(),
            theme: ChartTheme::default(),
        }
    }

    /// Create renderer with volume pane
    pub fn with_volume_pane(width: f64, height: f64) -> Self {
        let mut renderer = Self::new(width, height);
        renderer.volume_pane = Some(VolumePane::new(VolumePaneConfig::default()));
        renderer
    }

    /// Set theme
    pub fn set_theme(&mut self, theme: ChartTheme) {
        self.theme = theme;
        self.invalidation.invalidate_all(InvalidationLevel::Full);
    }

    /// Resize the renderer
    pub fn resize(&mut self, width: f64, height: f64) {
        self.viewport
            .set_dimensions(width as u32, height as u32, 1.0);
        self.invalidation.invalidate_all(InvalidationLevel::Full);
    }

    /// Set time range
    pub fn set_time_range(&mut self, range: TimeRange) {
        self.viewport.time = range;
        self.invalidation.invalidate_all(InvalidationLevel::Full);
    }

    /// Set price range
    pub fn set_price_range(&mut self, range: PriceRange) {
        self.viewport.price = range;
        self.invalidation.invalidate_all(InvalidationLevel::Full);
    }

    /// Invalidate specific component
    pub fn invalidate(&mut self, level: InvalidationLevel) {
        self.invalidation.invalidate_all(level);
    }

    /// Main render method - generates render commands for the current frame
    pub fn render(&mut self, candles: &[Candle]) -> RenderCommandBuffer {
        let mut buffer = RenderCommandBuffer::new(self.frame_id);
        self.frame_id += 1;

        // Skip rendering if nothing changed
        if !self.invalidation.needs_render() {
            return buffer;
        }

        // Fast path for cursor-only updates
        if self.invalidation.is_cursor_only() {
            // Only redraw crosshair (not implemented yet)
            self.invalidation.reset();
            return buffer;
        }

        // Full render
        self.render_full(candles, &mut buffer);

        // Reset invalidation after render
        self.invalidation.reset();

        buffer
    }

    /// Full render implementation
    fn render_full(&self, candles: &[Candle], buffer: &mut RenderCommandBuffer) {
        // 1. Clear canvas
        buffer.push(RenderCommand::Clear {
            color: self.theme.background.clone(),
        });

        // 2. Render grid
        self.render_grid(buffer);

        // 3. Render candles
        self.render_candles(candles, buffer);

        // 4. Render volume pane if enabled
        if self.volume_pane.is_some() {
            self.render_volume_pane(candles, buffer);
        }

        // 5. Render axes
        self.render_axes(buffer);

        // 6. Render indicators
        self.render_indicators(candles, buffer);

        // TODO: 7. Render drawings
        // TODO: 8. Render crosshair
    }

    /// Render grid lines
    fn render_grid(&self, buffer: &mut RenderCommandBuffer) {
        let width = self.viewport.dimensions.width as f64;
        let height = self.viewport.dimensions.height as f64;

        // Horizontal grid lines (price levels)
        let price_range = &self.viewport.price;
        let num_horizontal_lines = 8;
        let price_step = (price_range.max - price_range.min) / num_horizontal_lines as f64;

        for i in 0..=num_horizontal_lines {
            let price = price_range.min + (i as f64 * price_step);
            let y = self.viewport.price_to_y(price);

            buffer.push(RenderCommand::DrawLine {
                x1: 0.0,
                y1: y,
                x2: width,
                y2: y,
                color: self.theme.grid.clone(),
                width: 1.0,
            });
        }

        // Vertical grid lines (time)
        let num_vertical_lines = 10;
        let x_step = width / num_vertical_lines as f64;

        for i in 0..=num_vertical_lines {
            let x = i as f64 * x_step;

            buffer.push(RenderCommand::DrawLine {
                x1: x,
                y1: 0.0,
                x2: x,
                y2: height,
                color: self.theme.grid.clone(),
                width: 1.0,
            });
        }
    }

    /// Render candlesticks
    fn render_candles(&self, candles: &[Candle], buffer: &mut RenderCommandBuffer) {
        if candles.is_empty() {
            return;
        }

        let time_range = &self.viewport.time;
        let width = self.viewport.dimensions.width as f64;

        // Filter visible candles
        let visible_candles: Vec<&Candle> = candles
            .iter()
            .filter(|c| c.time >= time_range.start && c.time <= time_range.end)
            .collect();

        if visible_candles.is_empty() {
            return;
        }

        // Calculate bar width
        let bar_width = (width / visible_candles.len() as f64) * 0.8;

        // Convert candles to render data
        let candle_data: Vec<CandleData> = visible_candles
            .iter()
            .map(|candle| {
                let x = self.viewport.time_to_x(candle.time);

                CandleData {
                    x,
                    open_y: self.viewport.price_to_y(candle.o),
                    high_y: self.viewport.price_to_y(candle.h),
                    low_y: self.viewport.price_to_y(candle.l),
                    close_y: self.viewport.price_to_y(candle.c),
                    width: bar_width,
                }
            })
            .collect();

        // Use batch rendering for better performance
        buffer.push(RenderCommand::DrawCandlesBatch {
            candles: candle_data,
            bullish_color: self.theme.bullish.clone(),
            bearish_color: self.theme.bearish.clone(),
        });
    }

    /// Render volume pane
    fn render_volume_pane(&self, candles: &[Candle], buffer: &mut RenderCommandBuffer) {
        if let Some(ref pane) = self.volume_pane {
            let time_range = &self.viewport.time;
            let width = self.viewport.dimensions.width as f64;

            // Filter visible candles
            let visible_candles: Vec<&Candle> = candles
                .iter()
                .filter(|c| c.time >= time_range.start && c.time <= time_range.end)
                .collect();

            if visible_candles.is_empty() {
                return;
            }

            // Calculate bar width (same as candles)
            let bar_width = (width / visible_candles.len() as f64) * 0.8;

            // Convert to owned candles for volume pane
            let owned_candles: Vec<Candle> = visible_candles.into_iter().cloned().collect();

            // Render volume pane
            let index_to_x = |i: usize| -> f64 {
                if i < owned_candles.len() {
                    self.viewport.time_to_x(owned_candles[i].time)
                } else {
                    0.0
                }
            };

            pane.render(&owned_candles, bar_width, index_to_x, buffer);
        }
    }

    /// Render price and time axes
    fn render_axes(&self, buffer: &mut RenderCommandBuffer) {
        let width = self.viewport.dimensions.width as f64;
        let price_range = &self.viewport.price;

        // Price axis (right side)
        let num_price_labels = 8;
        let price_step = (price_range.max - price_range.min) / num_price_labels as f64;

        for i in 0..=num_price_labels {
            let price = price_range.min + (i as f64 * price_step);
            let y = self.viewport.price_to_y(price);

            buffer.push(RenderCommand::DrawText {
                text: format!("{:.2}", price),
                x: width - 60.0,
                y,
                font: "12px sans-serif".to_string(),
                color: self.theme.text.clone(),
                align: TextAlign::Right,
            });
        }

        // Time axis (bottom)
        // TODO: Implement time label formatting
    }

    /// Render all active indicators
    fn render_indicators(&self, candles: &[Candle], buffer: &mut RenderCommandBuffer) {
        let indicator_renderer = IndicatorRenderer::new(&self.viewport);

        for indicator in &self.indicators {
            let output = indicator.calculate(candles);
            indicator_renderer.render(&output, buffer);
        }
    }

    /// Get current viewport
    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    /// Get mutable viewport
    pub fn viewport_mut(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    /// Get invalidation mask
    pub fn invalidation(&self) -> &InvalidationMask {
        &self.invalidation
    }

    /// Get mutable invalidation mask
    pub fn invalidation_mut(&mut self) -> &mut InvalidationMask {
        &mut self.invalidation
    }

    /// Add an indicator to the chart
    pub fn add_indicator(&mut self, indicator: Box<dyn Indicator>) {
        self.indicators.push(indicator);
        self.invalidation.invalidate_all(InvalidationLevel::Full);
    }

    /// Remove an indicator by index
    pub fn remove_indicator(&mut self, index: usize) -> Option<Box<dyn Indicator>> {
        if index < self.indicators.len() {
            let indicator = self.indicators.remove(index);
            self.invalidation.invalidate_all(InvalidationLevel::Full);
            Some(indicator)
        } else {
            None
        }
    }

    /// Clear all indicators
    pub fn clear_indicators(&mut self) {
        self.indicators.clear();
        self.invalidation.invalidate_all(InvalidationLevel::Full);
    }

    /// Get the number of active indicators
    pub fn indicator_count(&self) -> usize {
        self.indicators.len()
    }

    /// Get a reference to all indicators
    pub fn indicators(&self) -> &[Box<dyn Indicator>] {
        &self.indicators
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_renderer_creation() {
        let renderer = ChartRenderer::new(800.0, 600.0);
        assert_eq!(renderer.frame_id, 0);
    }

    #[test]
    fn test_render_empty_candles() {
        let mut renderer = ChartRenderer::new(800.0, 600.0);
        renderer.invalidate(InvalidationLevel::Full);

        let buffer = renderer.render(&[]);
        assert!(buffer.len() > 0); // Should at least have clear command
    }

    #[test]
    fn test_render_with_candles() {
        let mut renderer = ChartRenderer::new(800.0, 600.0);
        renderer.set_time_range(TimeRange {
            start: 0,
            end: 1000,
        });
        renderer.set_price_range(PriceRange {
            min: 100.0,
            max: 200.0,
        });

        let candles = vec![
            Candle {
                time: 100,
                o: 150.0,
                h: 160.0,
                l: 140.0,
                c: 155.0,
                v: 1000.0,
            },
            Candle {
                time: 200,
                o: 155.0,
                h: 165.0,
                l: 145.0,
                c: 160.0,
                v: 1200.0,
            },
        ];

        let buffer = renderer.render(&candles);

        // Should have clear, grid, candles commands
        assert!(buffer.len() > 10);
    }

    #[test]
    fn test_theme_change() {
        let mut renderer = ChartRenderer::new(800.0, 600.0);
        renderer.set_theme(ChartTheme::light());

        assert!(renderer.invalidation.needs_full_render());
    }

    #[test]
    fn test_invalidation_tracking() {
        let mut renderer = ChartRenderer::new(800.0, 600.0);

        // Initial render
        renderer.invalidate(InvalidationLevel::Full);
        let _buffer = renderer.render(&[]);

        // Should be clean after render
        assert!(!renderer.invalidation.needs_render());

        // Invalidate again
        renderer.invalidate(InvalidationLevel::Light);
        assert!(renderer.invalidation.needs_render());
    }

    #[test]
    fn test_add_indicator() {
        use crate::indicators::builtin::rsi::RSI;

        let mut renderer = ChartRenderer::new(800.0, 600.0);
        assert_eq!(renderer.indicator_count(), 0);

        // Add RSI indicator
        let rsi = Box::new(RSI::new(14));
        renderer.add_indicator(rsi);

        assert_eq!(renderer.indicator_count(), 1);
        assert!(renderer.invalidation.needs_full_render());
    }

    #[test]
    fn test_render_with_indicator() {
        use crate::indicators::builtin::sma::SMA;

        let mut renderer = ChartRenderer::new(800.0, 600.0);
        renderer.set_time_range(TimeRange {
            start: 0,
            end: 1000,
        });
        renderer.set_price_range(PriceRange {
            min: 100.0,
            max: 200.0,
        });

        // Add SMA indicator
        let sma = Box::new(SMA::new(20));
        renderer.add_indicator(sma);

        let candles = vec![
            Candle {
                time: 100,
                o: 150.0,
                h: 160.0,
                l: 140.0,
                c: 155.0,
                v: 1000.0,
            },
            Candle {
                time: 200,
                o: 155.0,
                h: 165.0,
                l: 145.0,
                c: 160.0,
                v: 1200.0,
            },
        ];

        let buffer = renderer.render(&candles);

        // Should have candle commands + indicator line commands
        assert!(buffer.len() > 10);
        assert_eq!(renderer.indicator_count(), 1);
    }

    #[test]
    fn test_remove_indicator() {
        use crate::indicators::builtin::ema::EMA;

        let mut renderer = ChartRenderer::new(800.0, 600.0);

        // Add two indicators
        renderer.add_indicator(Box::new(EMA::new(12)));
        renderer.add_indicator(Box::new(EMA::new(26)));
        assert_eq!(renderer.indicator_count(), 2);

        // Remove first indicator
        let removed = renderer.remove_indicator(0);
        assert!(removed.is_some());
        assert_eq!(renderer.indicator_count(), 1);

        // Try to remove invalid index
        let invalid = renderer.remove_indicator(10);
        assert!(invalid.is_none());
        assert_eq!(renderer.indicator_count(), 1);
    }

    #[test]
    fn test_clear_indicators() {
        use crate::indicators::builtin::rsi::RSI;
        use crate::indicators::builtin::sma::SMA;

        let mut renderer = ChartRenderer::new(800.0, 600.0);

        // Add multiple indicators
        renderer.add_indicator(Box::new(RSI::new(14)));
        renderer.add_indicator(Box::new(SMA::new(20)));
        renderer.add_indicator(Box::new(SMA::new(50)));
        assert_eq!(renderer.indicator_count(), 3);

        // Clear all
        renderer.clear_indicators();
        assert_eq!(renderer.indicator_count(), 0);
        assert!(renderer.invalidation.needs_full_render());
    }
}
