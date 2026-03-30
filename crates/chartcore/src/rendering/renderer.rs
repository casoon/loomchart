//! Renderer trait - Abstract interface for drawing operations
//!
//! This trait defines all drawing operations needed by the chart.
//! Different backends (Canvas 2D, WebGPU) implement this trait.

use crate::primitives::Color;

/// Text alignment
#[derive(Debug, Clone, Copy)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

/// Line style for indicators and drawings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Solid,
    Dashed { dash_length: u32, gap_length: u32 },
    Dotted,
}

/// Text baseline
#[derive(Debug, Clone, Copy)]
pub enum TextBaseline {
    Top,
    Middle,
    Bottom,
}

/// Data for a single candle in batch rendering
#[derive(Debug, Clone, Copy)]
pub struct CandleData {
    pub x: f64,
    pub open_y: f64,
    pub high_y: f64,
    pub low_y: f64,
    pub close_y: f64,
    pub width: f64,
}

impl CandleData {
    /// Check if this is a bullish candle
    pub fn is_bullish(&self) -> bool {
        self.close_y <= self.open_y // Note: Y is inverted (0 at top)
    }

    /// Get the top of the candle body
    pub fn body_top(&self) -> f64 {
        self.open_y.min(self.close_y)
    }

    /// Get the height of the candle body
    pub fn body_height(&self) -> f64 {
        (self.open_y - self.close_y).abs().max(1.0) // Min 1px for doji
    }
}

/// Drawing style
#[derive(Debug, Clone)]
pub struct DrawStyle {
    pub stroke_color: Option<Color>,
    pub fill_color: Option<Color>,
    pub line_width: f32,
}

impl Default for DrawStyle {
    fn default() -> Self {
        Self {
            stroke_color: Some(Color::rgb(255, 255, 255)),
            fill_color: None,
            line_width: 1.0,
        }
    }
}

/// Render command - Represents a single drawing operation
#[derive(Debug, Clone)]
pub enum RenderCommand {
    Clear {
        color: Color,
    },
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: Color,
        width: f32,
    },
    Rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        style: DrawStyle,
    },
    Text {
        text: String,
        x: f64,
        y: f64,
        color: Color,
        size: f32,
        align: TextAlign,
        baseline: TextBaseline,
    },
    Candle {
        x: f64,
        open_y: f64,
        high_y: f64,
        low_y: f64,
        close_y: f64,
        width: f64,
        color: Color,
    },
    /// Draw multiple candlesticks in one batch (optimized)
    CandlesBatch {
        candles: Vec<CandleData>,
        bullish_color: Color,
        bearish_color: Color,
    },
    /// Draw an indicator line (optimized for many points)
    IndicatorLine {
        points: Vec<(f64, f64)>,
        color: Color,
        width: f32,
        style: LineStyle,
    },
}

/// Renderer trait - All backends must implement this
pub trait Renderer {
    /// Begin a new frame
    fn begin_frame(&mut self);

    /// End the current frame and flush
    fn end_frame(&mut self);

    /// Clear the canvas with a color
    fn clear(&mut self, color: Color);

    /// Draw a line
    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: Color, width: f32);

    /// Fill a rectangle
    fn fill_rect(&mut self, x: f64, y: f64, width: f64, height: f64, color: Color);

    /// Stroke a rectangle
    fn stroke_rect(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: Color,
        line_width: f32,
    );

    /// Draw text
    fn draw_text(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        color: Color,
        size: f32,
        align: TextAlign,
        baseline: TextBaseline,
    );

    /// Draw a candle (optimized batch operation)
    fn draw_candle(
        &mut self,
        x: f64,
        open_y: f64,
        high_y: f64,
        low_y: f64,
        close_y: f64,
        width: f64,
        bullish_color: Color,
        bearish_color: Color,
        unchanged_color: Color,
    ) {
        // 3-color system: up, down, unchanged
        let color = if (close_y - open_y).abs() < 0.001 {
            unchanged_color // Doji: open === close
        } else if close_y < open_y {
            bullish_color // Bullish: close > open (lower Y = higher price)
        } else {
            bearish_color // Bearish: close < open
        };

        // Draw wick (thin line from high to low)
        let wick_x = x + width / 2.0;
        self.draw_line(wick_x, high_y, wick_x, low_y, color, 1.0);

        // Draw body (rectangle from open to close)
        let body_height = (close_y - open_y).abs();
        let body_y = open_y.min(close_y);

        if body_height > 0.5 {
            self.fill_rect(x, body_y, width, body_height, color);
        } else {
            // Doji - draw as thin line
            self.draw_line(x, open_y, x + width, open_y, color, 1.0);
        }
    }

    /// Draw multiple candles in a batch (optimized)
    fn draw_candles_batch(
        &mut self,
        candles: &[CandleData],
        bullish_color: Color,
        bearish_color: Color,
        unchanged_color: Color,
    ) {
        for candle in candles {
            self.draw_candle(
                candle.x,
                candle.open_y,
                candle.high_y,
                candle.low_y,
                candle.close_y,
                candle.width,
                bullish_color,
                bearish_color,
                unchanged_color,
            );
        }
    }

    /// Draw an indicator line through multiple points
    fn draw_indicator_line(
        &mut self,
        points: &[(f64, f64)],
        color: Color,
        width: f32,
        _style: LineStyle,
    ) {
        if points.len() < 2 {
            return;
        }

        // TODO: Implement line styles (dashed, dotted)
        // For now, just draw solid lines

        for i in 0..points.len() - 1 {
            let (x1, y1) = points[i];
            let (x2, y2) = points[i + 1];
            self.draw_line(x1, y1, x2, y2, color, width);
        }
    }

    /// Set clipping region
    fn set_clip(&mut self, x: f64, y: f64, width: f64, height: f64);

    /// Clear clipping region
    fn clear_clip(&mut self);

    /// Get canvas dimensions
    fn dimensions(&self) -> (u32, u32);
}

/// Batch renderer - Collects commands for efficient rendering
#[allow(dead_code)]
pub struct BatchRenderer {
    commands: Vec<RenderCommand>,
    width: u32,
    height: u32,
}

impl BatchRenderer {
    #[allow(dead_code)]
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            commands: Vec::new(),
            width,
            height,
        }
    }
    #[allow(dead_code)]

    pub fn add_command(&mut self, cmd: RenderCommand) {
        self.commands.push(cmd);
    }

    #[allow(dead_code)]
    pub fn commands(&self) -> &[RenderCommand] {
        &self.commands
    }

    #[allow(dead_code)]
    pub fn clear_commands(&mut self) {
        self.commands.clear();
    }

    #[allow(dead_code)]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}

impl Renderer for BatchRenderer {
    fn begin_frame(&mut self) {
        self.commands.clear();
    }

    fn end_frame(&mut self) {
        // Commands are ready to be sent to actual renderer
    }

    fn clear(&mut self, color: Color) {
        self.commands.push(RenderCommand::Clear { color });
    }

    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: Color, width: f32) {
        self.commands.push(RenderCommand::Line {
            x1,
            y1,
            x2,
            y2,
            color,
            width,
        });
    }

    fn fill_rect(&mut self, x: f64, y: f64, width: f64, height: f64, color: Color) {
        self.commands.push(RenderCommand::Rect {
            x,
            y,
            width,
            height,
            style: DrawStyle {
                fill_color: Some(color),
                stroke_color: None,
                line_width: 0.0,
            },
        });
    }

    fn stroke_rect(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: Color,
        line_width: f32,
    ) {
        self.commands.push(RenderCommand::Rect {
            x,
            y,
            width,
            height,
            style: DrawStyle {
                fill_color: None,
                stroke_color: Some(color),
                line_width,
            },
        });
    }

    fn draw_text(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        color: Color,
        size: f32,
        align: TextAlign,
        baseline: TextBaseline,
    ) {
        self.commands.push(RenderCommand::Text {
            text: text.to_string(),
            x,
            y,
            color,
            size,
            align,
            baseline,
        });
    }

    fn set_clip(&mut self, _x: f64, _y: f64, _width: f64, _height: f64) {
        // TODO: Add clip command
    }

    fn clear_clip(&mut self) {
        // TODO: Add clear clip command
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}
