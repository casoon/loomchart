/// Canvas2D renderer for WASM
///
/// Executes RenderCommands using browser Canvas2D API via web_sys.
/// Optimized for batch operations and minimal context switches.

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

#[cfg_attr(all(target_arch = "wasm32", feature = "wasm"), allow(unused_imports))]
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use wasm_bindgen::JsCast;

use super::commands::{CandleData, LineStyle, RenderCommand, RenderCommandBuffer, TextAlign};
use crate::primitives::Color;

/// Canvas2D renderer implementation
#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub struct Canvas2DRenderer {
    ctx: CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
    pixel_ratio: f64,
    current_clip: Option<(f64, f64, f64, f64)>,
}

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
impl Canvas2DRenderer {
    /// Create a new Canvas2D renderer
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, String> {
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| "Failed to get 2d context")?
            .ok_or("2d context is None")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "Failed to cast to CanvasRenderingContext2d")?;

        let pixel_ratio = web_sys::window()
            .and_then(|w| Some(w.device_pixel_ratio()))
            .unwrap_or(1.0);

        Ok(Self {
            ctx,
            canvas,
            pixel_ratio,
            current_clip: None,
        })
    }

    /// Execute a buffer of render commands
    pub fn execute_commands(&mut self, buffer: &RenderCommandBuffer) {
        for cmd in &buffer.commands {
            self.execute_command(cmd);
        }
    }

    /// Execute a single render command
    fn execute_command(&mut self, cmd: &RenderCommand) {
        match cmd {
            RenderCommand::Clear { color } => {
                self.clear(color);
            }

            RenderCommand::DrawLine {
                x1,
                y1,
                x2,
                y2,
                color,
                width,
            } => {
                self.draw_line(*x1, *y1, *x2, *y2, color, *width);
            }

            RenderCommand::DrawRect {
                x,
                y,
                width,
                height,
                fill,
                stroke,
                stroke_width,
            } => {
                self.draw_rect(*x, *y, *width, *height, fill, stroke, *stroke_width);
            }

            RenderCommand::DrawText {
                text,
                x,
                y,
                font,
                color,
                align,
            } => {
                self.draw_text(text, *x, *y, font, color, *align);
            }

            RenderCommand::DrawCandle {
                x,
                open,
                high,
                low,
                close,
                body_width,
                wick_width,
                bullish_color,
                bearish_color,
            } => {
                self.draw_candle(
                    *x,
                    *open,
                    *high,
                    *low,
                    *close,
                    *body_width,
                    *wick_width,
                    bullish_color,
                    bearish_color,
                );
            }

            RenderCommand::DrawCandlesBatch {
                candles,
                bullish_color,
                bearish_color,
            } => {
                self.draw_candles_batch(candles, bullish_color, bearish_color);
            }

            RenderCommand::DrawIndicatorLine {
                points,
                color,
                width,
                style,
            } => {
                self.draw_indicator_line(points, color, *width, style);
            }

            RenderCommand::SetClip {
                x,
                y,
                width,
                height,
            } => {
                self.set_clip(*x, *y, *width, *height);
            }

            RenderCommand::ClearClip => {
                self.clear_clip();
            }
        }
    }

    /// Clear the canvas with a color
    fn clear(&mut self, color: &Color) {
        self.ctx.set_fill_style_str(&color.to_css());
        self.ctx.fill_rect(
            0.0,
            0.0,
            self.canvas.width() as f64,
            self.canvas.height() as f64,
        );
    }

    /// Draw a line
    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: &Color, width: f64) {
        let pr = self.pixel_ratio;
        let line_width = (width * pr).floor().max(1.0);

        self.ctx.set_stroke_style_str(&color.to_css());
        self.ctx.set_line_width(line_width);

        // Pixel-perfect correction for odd line widths (TradingView technique)
        let correction = if (line_width as i32) % 2 == 1 {
            0.5
        } else {
            0.0
        };

        self.ctx.begin_path();
        self.ctx.move_to(x1 * pr + correction, y1 * pr + correction);
        self.ctx.line_to(x2 * pr + correction, y2 * pr + correction);
        self.ctx.stroke();
    }

    /// Draw a horizontal line with pixel-perfect alignment (optimized for crosshair)
    pub fn draw_horizontal_line(
        &mut self,
        y: f64,
        left: f64,
        right: f64,
        color: &Color,
        width: f64,
    ) {
        let pr = self.pixel_ratio;
        let line_width = (width * pr).floor().max(1.0);
        let correction = if (line_width as i32) % 2 == 1 {
            0.5
        } else {
            0.0
        };

        self.ctx.set_stroke_style_str(&color.to_css());
        self.ctx.set_line_width(line_width);
        self.ctx.begin_path();
        self.ctx.move_to(left, y * pr + correction);
        self.ctx.line_to(right, y * pr + correction);
        self.ctx.stroke();
    }

    /// Draw a vertical line with pixel-perfect alignment (optimized for crosshair)
    pub fn draw_vertical_line(&mut self, x: f64, top: f64, bottom: f64, color: &Color, width: f64) {
        let pr = self.pixel_ratio;
        let line_width = (width * pr).floor().max(1.0);
        let correction = if (line_width as i32) % 2 == 1 {
            0.5
        } else {
            0.0
        };

        self.ctx.set_stroke_style_str(&color.to_css());
        self.ctx.set_line_width(line_width);
        self.ctx.begin_path();
        self.ctx.move_to(x * pr + correction, top);
        self.ctx.line_to(x * pr + correction, bottom);
        self.ctx.stroke();
    }

    /// Draw a rectangle
    fn draw_rect(
        &mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        fill: &Option<Color>,
        stroke: &Option<Color>,
        stroke_width: f64,
    ) {
        let px = x * self.pixel_ratio;
        let py = y * self.pixel_ratio;
        let pw = width * self.pixel_ratio;
        let ph = height * self.pixel_ratio;

        if let Some(fill_color) = fill {
            self.ctx.set_fill_style_str(&fill_color.to_css());
            self.ctx.fill_rect(px, py, pw, ph);
        }

        if let Some(stroke_color) = stroke {
            self.ctx.set_stroke_style_str(&stroke_color.to_css());
            self.ctx.set_line_width(stroke_width * self.pixel_ratio);
            self.ctx.stroke_rect(px, py, pw, ph);
        }
    }

    /// Draw text
    fn draw_text(
        &mut self,
        text: &str,
        x: f64,
        y: f64,
        font: &str,
        color: &Color,
        align: TextAlign,
    ) {
        self.ctx.set_font(font);
        self.ctx.set_fill_style_str(&color.to_css());
        self.ctx.set_text_align(match align {
            TextAlign::Left => "left",
            TextAlign::Center => "center",
            TextAlign::Right => "right",
        });
        self.ctx
            .fill_text(text, x * self.pixel_ratio, y * self.pixel_ratio)
            .ok();
    }

    /// Draw a single candle
    fn draw_candle(
        &mut self,
        x: f64,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        body_width: f64,
        wick_width: f64,
        bullish_color: &Color,
        bearish_color: &Color,
    ) {
        let is_bullish = close >= open;
        let color = if is_bullish {
            bullish_color
        } else {
            bearish_color
        };

        // Draw wick
        self.draw_line(x, high, x, low, color, wick_width);

        // Draw body
        let body_top = open.max(close);
        let body_bottom = open.min(close);
        let body_height = (body_top - body_bottom).max(1.0); // Min 1px for doji

        self.draw_rect(
            x - body_width / 2.0,
            body_bottom,
            body_width,
            body_height,
            &Some(color.clone()),
            &None,
            0.0,
        );
    }

    /// Draw multiple candles in one batch (OPTIMIZED)
    ///
    /// This is much faster than drawing candles individually because:
    /// 1. Minimizes context state changes
    /// 2. Uses single path for all wicks
    /// 3. Batches all bodies of same color together
    fn draw_candles_batch(
        &mut self,
        candles: &[CandleData],
        bullish_color: &Color,
        bearish_color: &Color,
    ) {
        if candles.is_empty() {
            return;
        }

        // Separate bullish and bearish candles for batching
        let (bullish, bearish): (Vec<&CandleData>, Vec<&CandleData>) =
            candles.iter().partition(|c| c.is_bullish());

        // Draw all bullish candles
        if !bullish.is_empty() {
            self.draw_candles_batch_internal(&bullish, bullish_color);
        }

        // Draw all bearish candles
        if !bearish.is_empty() {
            self.draw_candles_batch_internal(&bearish, bearish_color);
        }
    }

    /// Internal batch drawing for candles of same color
    fn draw_candles_batch_internal(&mut self, candles: &[&CandleData], color: &Color) {
        let pr = self.pixel_ratio;

        // Step 1: Draw all wicks in one path
        self.ctx.set_stroke_style_str(&color.to_css());
        self.ctx.set_line_width(1.0 * pr);
        self.ctx.begin_path();

        for candle in candles {
            self.ctx.move_to(candle.x * pr, candle.high_y * pr);
            self.ctx.line_to(candle.x * pr, candle.low_y * pr);
        }

        self.ctx.stroke();

        // Step 2: Draw all bodies
        self.ctx.set_fill_style_str(&color.to_css());

        for candle in candles {
            let body_top = candle.body_top();
            let body_height = candle.body_height();

            self.ctx.fill_rect(
                (candle.x - candle.width / 2.0) * pr,
                body_top * pr,
                candle.width * pr,
                body_height * pr,
            );
        }
    }

    /// Draw an indicator line with many points
    fn draw_indicator_line(
        &mut self,
        points: &[(f64, f64)],
        color: &Color,
        width: f64,
        style: &LineStyle,
    ) {
        if points.len() < 2 {
            return;
        }

        self.ctx.set_stroke_style_str(&color.to_css());
        self.ctx.set_line_width(width * self.pixel_ratio);

        // Set line style
        match style {
            LineStyle::Solid => {
                self.ctx.set_line_dash(&js_sys::Array::new()).ok();
            }
            LineStyle::Dashed {
                dash_length,
                gap_length,
            } => {
                let dash_array = js_sys::Array::new();
                dash_array.push(&((*dash_length as f64) * self.pixel_ratio).into());
                dash_array.push(&((*gap_length as f64) * self.pixel_ratio).into());
                self.ctx.set_line_dash(&dash_array).ok();
            }
            LineStyle::Dotted => {
                let dash_array = js_sys::Array::new();
                dash_array.push(&(2.0 * self.pixel_ratio).into());
                dash_array.push(&(2.0 * self.pixel_ratio).into());
                self.ctx.set_line_dash(&dash_array).ok();
            }
        }

        // Draw line through all points
        self.ctx.begin_path();
        let first = &points[0];
        self.ctx
            .move_to(first.0 * self.pixel_ratio, first.1 * self.pixel_ratio);

        for point in &points[1..] {
            self.ctx
                .line_to(point.0 * self.pixel_ratio, point.1 * self.pixel_ratio);
        }

        self.ctx.stroke();

        // Reset line dash
        self.ctx.set_line_dash(&js_sys::Array::new()).ok();
    }

    /// Set clipping region
    fn set_clip(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.ctx.save();
        self.ctx.begin_path();
        self.ctx.rect(
            x * self.pixel_ratio,
            y * self.pixel_ratio,
            width * self.pixel_ratio,
            height * self.pixel_ratio,
        );
        self.ctx.clip();
        self.current_clip = Some((x, y, width, height));
    }

    /// Clear clipping region
    fn clear_clip(&mut self) {
        if self.current_clip.is_some() {
            self.ctx.restore();
            self.current_clip = None;
        }
    }

    /// Resize the canvas
    pub fn resize(&mut self, width: u32, height: u32) {
        self.canvas
            .set_width((width as f64 * self.pixel_ratio) as u32);
        self.canvas
            .set_height((height as f64 * self.pixel_ratio) as u32);
    }
}

impl Color {
    /// Convert to JavaScript value for canvas API
    #[cfg(target_arch = "wasm32")]
    pub fn to_js_value(&self) -> wasm_bindgen::JsValue {
        wasm_bindgen::JsValue::from_str(&self.to_css())
    }
}

// Non-WASM stub for testing/compilation
#[cfg(not(target_arch = "wasm32"))]
pub struct Canvas2DRenderer;

#[cfg(not(target_arch = "wasm32"))]
impl Canvas2DRenderer {
    pub fn execute_commands(&mut self, _buffer: &RenderCommandBuffer) {
        // No-op on non-WASM
    }
}
