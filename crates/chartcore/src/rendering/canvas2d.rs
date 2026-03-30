//! Canvas 2D Renderer - WebAssembly bindings for HTML5 Canvas 2D API

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use super::renderer::{RenderCommand, Renderer, TextAlign, TextBaseline};
use crate::canvas::{BitmapSpace, CssPixels, DevicePixels, MediaSpace, PixelRatio};
use crate::primitives::Color;

#[cfg(feature = "wasm")]
pub struct Canvas2DRenderer {
    pub ctx: CanvasRenderingContext2d,
    canvas: HtmlCanvasElement,
    width: u32,
    height: u32,
    pixel_ratio: f64,
    bitmap_space: BitmapSpace,
    media_space: MediaSpace,
}

#[cfg(feature = "wasm")]
impl Canvas2DRenderer {
    pub fn pixel_ratio(&self) -> f64 {
        self.pixel_ratio
    }

    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        let ctx = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("Failed to get 2D context"))?
            .dyn_into::<CanvasRenderingContext2d>()?;

        let width = canvas.width();
        let height = canvas.height();

        // Get device pixel ratio from window
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        let pixel_ratio = window.device_pixel_ratio();

        let pr = PixelRatio::new(pixel_ratio);
        let bitmap_space = BitmapSpace::new(CssPixels(width as f64), CssPixels(height as f64), pr);
        let media_space = MediaSpace::new(CssPixels(width as f64), CssPixels(height as f64), pr);

        Ok(Self {
            ctx,
            canvas,
            width,
            height,
            pixel_ratio,
            bitmap_space,
            media_space,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsValue> {
        // Apply device pixel ratio with maximum canvas size limit
        let max_canvas_size = 16384;
        let max_pixels = 268435456; // 256MB limit

        let mut pr = self.pixel_ratio;
        let mut physical_width = (width as f64 * pr) as u32;
        let mut physical_height = (height as f64 * pr) as u32;

        // Check if canvas would be too large
        while (physical_width > max_canvas_size
            || physical_height > max_canvas_size
            || (physical_width as u64 * physical_height as u64) > max_pixels)
            && pr > 1.0
        {
            pr = (pr / 2.0).max(1.0);
            physical_width = (width as f64 * pr) as u32;
            physical_height = (height as f64 * pr) as u32;
        }

        self.pixel_ratio = pr;

        // Set physical size (internal canvas buffer)
        self.canvas.set_width(physical_width);
        self.canvas.set_height(physical_height);

        // DON'T set CSS style here - that triggers ResizeObserver loop!
        // The CSS size is already set by the container element

        // Reset transform and scale context for pixel ratio
        self.ctx.reset_transform()?;
        self.ctx.scale(pr, pr)?;

        self.width = width;
        self.height = height;

        // Update coordinate spaces
        let pr = PixelRatio::new(self.pixel_ratio);
        self.bitmap_space = BitmapSpace::new(CssPixels(width as f64), CssPixels(height as f64), pr);
        self.media_space = MediaSpace::new(CssPixels(width as f64), CssPixels(height as f64), pr);

        Ok(())
    }

    pub fn execute_command(&mut self, cmd: &RenderCommand) {
        match cmd {
            RenderCommand::Clear { color } => {
                self.clear(*color);
            }
            RenderCommand::Line {
                x1,
                y1,
                x2,
                y2,
                color,
                width,
            } => {
                self.draw_line(*x1, *y1, *x2, *y2, *color, *width);
            }
            RenderCommand::Rect {
                x,
                y,
                width,
                height,
                style,
            } => {
                if let Some(fill_color) = style.fill_color {
                    self.fill_rect(*x, *y, *width, *height, fill_color);
                }
                if let Some(stroke_color) = style.stroke_color {
                    self.stroke_rect(*x, *y, *width, *height, stroke_color, style.line_width);
                }
            }
            RenderCommand::Text {
                text,
                x,
                y,
                color,
                size,
                align,
                baseline,
            } => {
                self.draw_text(text, *x, *y, *color, *size, *align, *baseline);
            }
            RenderCommand::Candle {
                x,
                open_y,
                high_y,
                low_y,
                close_y,
                width,
                color: _,
            } => {
                let is_bullish = close_y < open_y;
                let bullish_color = Color {
                    r: 34,
                    g: 197,
                    b: 94,
                    a: 1.0,
                };
                let bearish_color = Color {
                    r: 239,
                    g: 68,
                    b: 68,
                    a: 1.0,
                };
                let candle_color = if is_bullish {
                    bullish_color
                } else {
                    bearish_color
                };
                // Use gray for unchanged (we don't have it in RenderCommand yet, so use bearish)
                self.draw_candle(
                    *x,
                    *open_y,
                    *high_y,
                    *low_y,
                    *close_y,
                    *width,
                    bullish_color,
                    bearish_color,
                    bearish_color, // unchanged_color - TODO: add to RenderCommand
                );
            }
            RenderCommand::CandlesBatch {
                candles,
                bullish_color,
                bearish_color,
            } => {
                // Use gray for unchanged (we don't have it in RenderCommand yet, so use bearish)
                self.draw_candles_batch(candles, *bullish_color, *bearish_color, *bearish_color);
            }
            RenderCommand::IndicatorLine {
                points,
                color,
                width,
                style,
            } => {
                self.draw_indicator_line(points, *color, *width, *style);
            }
        }
    }

    fn set_fill_color(&self, color: Color) {
        let color_str = format!("rgba({}, {}, {}, {})", color.r, color.g, color.b, color.a);
        self.ctx.set_fill_style_str(&color_str);
    }

    fn set_stroke_color(&self, color: Color) {
        let color_str = format!("rgba({}, {}, {}, {})", color.r, color.g, color.b, color.a);
        self.ctx.set_stroke_style_str(&color_str);
    }

    /// Draw a horizontal line with pixel-perfect alignment (TradingView technique)
    pub fn draw_horizontal_line(
        &mut self,
        y: f64,
        left: f64,
        right: f64,
        color: &Color,
        width: f64,
    ) {
        let line_width = (width * self.pixel_ratio).floor().max(1.0);
        let correction = if (line_width as i32) % 2 == 1 {
            0.5
        } else {
            0.0
        };

        self.set_stroke_color(*color);
        self.ctx.set_line_width(line_width);
        self.ctx.begin_path();
        self.ctx.move_to(left, y + correction);
        self.ctx.line_to(right, y + correction);
        self.ctx.stroke();
    }

    /// Draw a vertical line with pixel-perfect alignment (TradingView technique)
    pub fn draw_vertical_line(&mut self, x: f64, top: f64, bottom: f64, color: &Color, width: f64) {
        let line_width = (width * self.pixel_ratio).floor().max(1.0);
        let correction = if (line_width as i32) % 2 == 1 {
            0.5
        } else {
            0.0
        };

        self.set_stroke_color(*color);
        self.ctx.set_line_width(line_width);
        self.ctx.begin_path();
        self.ctx.move_to(x + correction, top);
        self.ctx.line_to(x + correction, bottom);
        self.ctx.stroke();
    }
}

#[cfg(feature = "wasm")]
impl Renderer for Canvas2DRenderer {
    fn begin_frame(&mut self) {
        // Nothing to do for Canvas 2D
    }

    fn end_frame(&mut self) {
        // Nothing to do for Canvas 2D
    }

    fn clear(&mut self, color: Color) {
        self.set_fill_color(color);
        self.ctx
            .fill_rect(0.0, 0.0, self.width as f64, self.height as f64);
    }

    fn draw_line(&mut self, x1: f64, y1: f64, x2: f64, y2: f64, color: Color, width: f32) {
        self.set_stroke_color(color);
        self.ctx.set_line_width(width as f64);
        self.ctx.begin_path();
        self.ctx.move_to(x1, y1);
        self.ctx.line_to(x2, y2);
        self.ctx.stroke();
    }

    fn fill_rect(&mut self, x: f64, y: f64, width: f64, height: f64, color: Color) {
        self.set_fill_color(color);
        self.ctx.fill_rect(x, y, width, height);
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
        self.set_stroke_color(color);
        self.ctx.set_line_width(line_width as f64);
        self.ctx.stroke_rect(x, y, width, height);
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
        self.set_fill_color(color);
        self.ctx.set_font(&format!("{}px monospace", size));

        let align_str = match align {
            TextAlign::Left => "left",
            TextAlign::Center => "center",
            TextAlign::Right => "right",
        };
        self.ctx.set_text_align(align_str);

        let baseline_str = match baseline {
            TextBaseline::Top => "top",
            TextBaseline::Middle => "middle",
            TextBaseline::Bottom => "bottom",
        };
        self.ctx.set_text_baseline(baseline_str);

        let _ = self.ctx.fill_text(text, x, y);
    }

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

        // Draw wick (high to low)
        self.draw_line(x, high_y, x, low_y, color, 1.0);

        // Draw body (open to close)
        let body_top = open_y.min(close_y);
        let body_height = (open_y - close_y).abs();
        let body_x = x - width / 2.0;

        if body_height < 1.0 {
            // Doji - draw horizontal line
            self.draw_line(body_x, open_y, body_x + width, open_y, color, 1.0);
        } else {
            self.fill_rect(body_x, body_top, width, body_height, color);
        }
    }

    fn set_clip(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.ctx.save();
        self.ctx.begin_path();
        self.ctx.rect(x, y, width, height);
        self.ctx.clip();
    }

    fn clear_clip(&mut self) {
        self.ctx.restore();
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

// Additional rendering methods for Canvas2DRenderer
impl Canvas2DRenderer {
    /// Draw OHLC bar (Open-High-Low-Close)
    pub fn draw_ohlc(
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

        // Calculate arm length (like chartjs-chart-financial)
        // armLengthRatio = 0.8, multiplied by 0.5
        let arm_length = width * 0.8 * 0.5;

        // Vertical line from high to low
        self.draw_line(x, high_y, x, low_y, color, 2.0);

        // Open tick (left)
        self.draw_line(x - arm_length, open_y, x, open_y, color, 2.0);

        // Close tick (right)
        self.draw_line(x, close_y, x + arm_length, close_y, color, 2.0);
    }

    /// Draw hollow candlestick (outline only)
    pub fn draw_hollow_candle(
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

        // Draw wick (high to low)
        self.draw_line(x, high_y, x, low_y, color, 1.0);

        // Draw body outline (open to close)
        let body_top = open_y.min(close_y);
        let body_height = (open_y - close_y).abs();
        let body_x = x - width / 2.0;

        if body_height < 1.0 {
            // Doji - draw horizontal line
            self.draw_line(body_x, open_y, body_x + width, open_y, color, 1.0);
        } else {
            self.stroke_rect(body_x, body_top, width, body_height, color, 1.0);
        }
    }

    /// Draw a circle
    pub fn draw_circle(&mut self, x: f64, y: f64, radius: f64, color: Color) {
        self.set_fill_color(color);
        self.ctx.begin_path();
        self.ctx
            .arc(x, y, radius, 0.0, 2.0 * std::f64::consts::PI)
            .unwrap_or_default();
        self.ctx.fill();
    }

    // ===== Coordinate-Aware Rendering Methods =====

    /// Get bitmap space (device pixel coordinates)
    pub fn bitmap_space(&self) -> &BitmapSpace {
        &self.bitmap_space
    }

    /// Get media space (CSS pixel coordinates)
    pub fn media_space(&self) -> &MediaSpace {
        &self.media_space
    }

    /// Draw a line using bitmap coordinates with pixel-perfect alignment
    pub fn draw_line_bitmap(
        &mut self,
        x1: DevicePixels,
        y1: DevicePixels,
        x2: DevicePixels,
        y2: DevicePixels,
        color: Color,
        width: f64,
    ) {
        // Apply pixel-perfect correction for odd line widths
        let x1_aligned = self.bitmap_space.align_to_pixel_grid(x1, width);
        let y1_aligned = self.bitmap_space.align_to_pixel_grid(y1, width);
        let x2_aligned = self.bitmap_space.align_to_pixel_grid(x2, width);
        let y2_aligned = self.bitmap_space.align_to_pixel_grid(y2, width);

        // Convert to CSS coordinates for rendering (context is already scaled)
        let css_x1 = self.bitmap_space.to_css_x(x1_aligned);
        let css_y1 = self.bitmap_space.to_css_y(y1_aligned);
        let css_x2 = self.bitmap_space.to_css_x(x2_aligned);
        let css_y2 = self.bitmap_space.to_css_y(y2_aligned);

        self.draw_line(css_x1.0, css_y1.0, css_x2.0, css_y2.0, color, width as f32);
    }

    /// Draw a horizontal line in bitmap space with perfect pixel alignment
    pub fn draw_horizontal_line_bitmap(
        &mut self,
        y: DevicePixels,
        left: DevicePixels,
        right: DevicePixels,
        color: Color,
        width: f64,
    ) {
        let y_aligned = self.bitmap_space.align_to_pixel_grid(y, width);
        let left_aligned = self.bitmap_space.floor(left);
        let right_aligned = self.bitmap_space.ceil(right);

        let css_y = self.bitmap_space.to_css_y(y_aligned);
        let css_left = self.bitmap_space.to_css_x(left_aligned);
        let css_right = self.bitmap_space.to_css_x(right_aligned);

        self.draw_horizontal_line(css_y.0, css_left.0, css_right.0, &color, width);
    }

    /// Draw a vertical line in bitmap space with perfect pixel alignment
    pub fn draw_vertical_line_bitmap(
        &mut self,
        x: DevicePixels,
        top: DevicePixels,
        bottom: DevicePixels,
        color: Color,
        width: f64,
    ) {
        let x_aligned = self.bitmap_space.align_to_pixel_grid(x, width);
        let top_aligned = self.bitmap_space.floor(top);
        let bottom_aligned = self.bitmap_space.ceil(bottom);

        let css_x = self.bitmap_space.to_css_x(x_aligned);
        let css_top = self.bitmap_space.to_css_y(top_aligned);
        let css_bottom = self.bitmap_space.to_css_y(bottom_aligned);

        self.draw_vertical_line(css_x.0, css_top.0, css_bottom.0, &color, width);
    }

    /// Draw text in media space (CSS coordinates) with DPI-aware font sizing
    pub fn draw_text_media(
        &mut self,
        text: &str,
        x: CssPixels,
        y: CssPixels,
        color: Color,
        css_font_size: f64,
        align: TextAlign,
        baseline: TextBaseline,
    ) {
        // Calculate device pixel font size for crisp text rendering
        let device_font_size = self.media_space.font_size_device(css_font_size);

        self.draw_text(
            text,
            x.0,
            y.0,
            color,
            device_font_size as f32,
            align,
            baseline,
        );
    }

    /// Fill rectangle in bitmap space with pixel-perfect edges
    pub fn fill_rect_bitmap(
        &mut self,
        x: DevicePixels,
        y: DevicePixels,
        width: DevicePixels,
        height: DevicePixels,
        color: Color,
    ) {
        let x_aligned = self.bitmap_space.floor(x);
        let y_aligned = self.bitmap_space.floor(y);
        let width_aligned = self.bitmap_space.ceil(width);
        let height_aligned = self.bitmap_space.ceil(height);

        let css_x = self.bitmap_space.to_css_x(x_aligned);
        let css_y = self.bitmap_space.to_css_y(y_aligned);
        let css_width = self.bitmap_space.to_css_x(width_aligned);
        let css_height = self.bitmap_space.to_css_y(height_aligned);

        self.fill_rect(css_x.0, css_y.0, css_width.0, css_height.0, color);
    }

    /// Convert mouse event coordinates (CSS pixels) to device pixels
    pub fn css_to_device(
        &self,
        css_x: CssPixels,
        css_y: CssPixels,
    ) -> (DevicePixels, DevicePixels) {
        (
            self.bitmap_space.to_device_x(css_x),
            self.bitmap_space.to_device_y(css_y),
        )
    }

    /// Convert device pixel coordinates to CSS pixels
    pub fn device_to_css(
        &self,
        device_x: DevicePixels,
        device_y: DevicePixels,
    ) -> (CssPixels, CssPixels) {
        (
            self.bitmap_space.to_css_x(device_x),
            self.bitmap_space.to_css_y(device_y),
        )
    }
}
