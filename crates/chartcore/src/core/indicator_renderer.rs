/// Indicator rendering - converts IndicatorOutput to RenderCommands
///
/// This module bridges the gap between indicator calculations (IndicatorOutput)
/// and the rendering system (RenderCommand). Each IndicatorOutput variant
/// gets converted to appropriate render commands.
use crate::core::viewport::Viewport;
use crate::indicators::output::{IndicatorOutput, LineStyle, MarkerShape, ScatterPoint};
use crate::primitives::Color;
use crate::renderers::commands::{RenderCommand, RenderCommandBuffer};

/// Indicator renderer - converts indicator output to render commands
pub struct IndicatorRenderer<'a> {
    viewport: &'a Viewport,
}

impl<'a> IndicatorRenderer<'a> {
    /// Create new indicator renderer with viewport
    pub fn new(viewport: &'a Viewport) -> Self {
        Self { viewport }
    }

    /// Main render method - converts IndicatorOutput to commands
    pub fn render(&self, output: &IndicatorOutput, buffer: &mut RenderCommandBuffer) {
        match output {
            IndicatorOutput::SingleLine {
                values,
                color,
                width,
                style,
            } => {
                self.render_line(values, color, *width, style, buffer);
            }

            IndicatorOutput::MultiLine { lines } => {
                for line in lines {
                    self.render_line(&line.values, &line.color, line.width, &line.style, buffer);
                }
            }

            IndicatorOutput::Histogram {
                values,
                positive_color,
                negative_color,
                zero_line,
            } => {
                self.render_histogram(values, positive_color, negative_color, *zero_line, buffer);
            }

            IndicatorOutput::CloudArea {
                upper,
                lower,
                bullish_color,
                bearish_color,
                alpha,
            } => {
                self.render_cloud(upper, lower, bullish_color, bearish_color, *alpha, buffer);
            }

            IndicatorOutput::Scatter {
                points,
                color,
                size,
                shape,
            } => {
                self.render_scatter(points, color, *size, shape, buffer);
            }

            IndicatorOutput::Bands {
                middle,
                upper,
                lower,
                middle_color,
                band_color,
                fill_alpha,
            } => {
                self.render_bands(
                    middle,
                    upper,
                    lower,
                    middle_color,
                    band_color,
                    *fill_alpha,
                    buffer,
                );
            }
        }
    }

    /// Render a single line
    fn render_line(
        &self,
        values: &[Option<f64>],
        color: &Color,
        width: f64,
        style: &LineStyle,
        buffer: &mut RenderCommandBuffer,
    ) {
        let mut points = Vec::new();

        for (i, value_opt) in values.iter().enumerate() {
            if let Some(value) = value_opt {
                let x = self.index_to_x(i);
                let y = self.viewport.price_to_y(*value);
                points.push((x, y));
            } else {
                // Break line on missing data
                if !points.is_empty() {
                    self.push_polyline(&points, color, width, style, buffer);
                    points.clear();
                }
            }
        }

        // Draw remaining points
        if !points.is_empty() {
            self.push_polyline(&points, color, width, style, buffer);
        }
    }

    /// Push a polyline (connected series of line segments)
    fn push_polyline(
        &self,
        points: &[(f64, f64)],
        color: &Color,
        width: f64,
        _style: &LineStyle,
        buffer: &mut RenderCommandBuffer,
    ) {
        if points.len() < 2 {
            return;
        }

        // For now, draw as individual line segments
        // TODO: Optimize with a single DrawPolyline command
        for i in 0..points.len() - 1 {
            let (x1, y1) = points[i];
            let (x2, y2) = points[i + 1];

            buffer.push(RenderCommand::DrawLine {
                x1,
                y1,
                x2,
                y2,
                color: color.clone(),
                width,
            });

            // Apply line style (dashed/dotted) via dash pattern
            // TODO: Add dash pattern support to RenderCommand
        }
    }

    /// Render histogram bars
    fn render_histogram(
        &self,
        values: &[f64],
        pos_color: &Color,
        neg_color: &Color,
        zero_line: bool,
        buffer: &mut RenderCommandBuffer,
    ) {
        let zero_y = self.viewport.price_to_y(0.0);
        let bar_width = self.calculate_bar_width(values.len());

        for (i, &value) in values.iter().enumerate() {
            let x = self.index_to_x(i);
            let value_y = self.viewport.price_to_y(value);
            let color = if value >= 0.0 { pos_color } else { neg_color };

            let width = bar_width * 0.6;
            let height = (value_y - zero_y).abs();
            let y = value_y.min(zero_y);

            buffer.push(RenderCommand::DrawRect {
                x: x - width / 2.0,
                y,
                width,
                height,
                fill: Some(color.clone()),
                stroke: None,
                stroke_width: 0.0,
            });
        }

        // Draw zero line
        if zero_line {
            let chart_width = self.viewport.dimensions.width as f64;
            buffer.push(RenderCommand::DrawLine {
                x1: 0.0,
                y1: zero_y,
                x2: chart_width,
                y2: zero_y,
                color: Color::rgba(255, 255, 255, 0.3),
                width: 1.0,
            });
        }
    }

    /// Render cloud area between two lines
    fn render_cloud(
        &self,
        upper: &[f64],
        lower: &[f64],
        bullish_color: &Color,
        bearish_color: &Color,
        alpha: f64,
        buffer: &mut RenderCommandBuffer,
    ) {
        let len = upper.len().min(lower.len());

        for i in 1..len {
            let is_bullish = upper[i] > lower[i];
            let mut color = if is_bullish {
                bullish_color.clone()
            } else {
                bearish_color.clone()
            };

            // Apply alpha transparency
            color = self.apply_alpha(color, alpha);

            // Draw cloud segment as trapezoid (approximated as rect for now)
            let x1 = self.index_to_x(i - 1);
            let x2 = self.index_to_x(i);

            let upper1 = self.viewport.price_to_y(upper[i - 1]);
            let upper2 = self.viewport.price_to_y(upper[i]);
            let lower1 = self.viewport.price_to_y(lower[i - 1]);
            let lower2 = self.viewport.price_to_y(lower[i]);

            // For simplicity, draw as filled rectangle
            // TODO: Implement polygon rendering for smoother clouds
            let y = upper1.min(upper2).min(lower1).min(lower2);
            let max_y = upper1.max(upper2).max(lower1).max(lower2);
            let height = max_y - y;

            buffer.push(RenderCommand::DrawRect {
                x: x1,
                y,
                width: x2 - x1,
                height,
                fill: Some(color),
                stroke: None,
                stroke_width: 0.0,
            });
        }
    }

    /// Render scatter plot points
    fn render_scatter(
        &self,
        points: &[ScatterPoint],
        color: &Color,
        size: f64,
        shape: &MarkerShape,
        buffer: &mut RenderCommandBuffer,
    ) {
        for point in points {
            let x = self.index_to_x(point.index);
            let y = self.viewport.price_to_y(point.value);

            match shape {
                MarkerShape::Circle => {
                    self.draw_circle(x, y, size, color, buffer);
                }
                MarkerShape::Square => {
                    let half = size / 2.0;
                    buffer.push(RenderCommand::DrawRect {
                        x: x - half,
                        y: y - half,
                        width: size,
                        height: size,
                        fill: Some(color.clone()),
                        stroke: None,
                        stroke_width: 0.0,
                    });
                }
                MarkerShape::Triangle => {
                    self.draw_triangle(x, y, size, color, buffer);
                }
                MarkerShape::Cross => {
                    self.draw_cross(x, y, size, color, buffer);
                }
                MarkerShape::Diamond => {
                    self.draw_diamond(x, y, size, color, buffer);
                }
            }
        }
    }

    /// Render bands (upper, middle, lower lines with fill)
    fn render_bands(
        &self,
        middle: &[Option<f64>],
        upper: &[Option<f64>],
        lower: &[Option<f64>],
        middle_color: &Color,
        band_color: &Color,
        fill_alpha: f64,
        buffer: &mut RenderCommandBuffer,
    ) {
        // 1. Draw filled area between upper and lower bands
        let len = middle.len().min(upper.len()).min(lower.len());
        let mut fill_color = band_color.clone();
        fill_color = self.apply_alpha(fill_color, fill_alpha);

        for i in 1..len {
            if let (Some(u1), Some(u2), Some(l1), Some(l2)) =
                (upper[i - 1], upper[i], lower[i - 1], lower[i])
            {
                let x1 = self.index_to_x(i - 1);
                let x2 = self.index_to_x(i);

                let upper1_y = self.viewport.price_to_y(u1);
                let upper2_y = self.viewport.price_to_y(u2);
                let lower1_y = self.viewport.price_to_y(l1);
                let lower2_y = self.viewport.price_to_y(l2);

                // Draw as rectangle (approximate)
                let y = upper1_y.min(upper2_y);
                let max_y = lower1_y.max(lower2_y);
                let height = max_y - y;

                buffer.push(RenderCommand::DrawRect {
                    x: x1,
                    y,
                    width: x2 - x1,
                    height,
                    fill: Some(fill_color.clone()),
                    stroke: None,
                    stroke_width: 0.0,
                });
            }
        }

        // 2. Draw upper band line
        self.render_line(upper, band_color, 1.0, &LineStyle::Solid, buffer);

        // 3. Draw middle line
        self.render_line(middle, middle_color, 1.5, &LineStyle::Solid, buffer);

        // 4. Draw lower band line
        self.render_line(lower, band_color, 1.0, &LineStyle::Solid, buffer);
    }

    // === Helper methods ===

    /// Convert candle index to X coordinate
    fn index_to_x(&self, index: usize) -> f64 {
        // TODO: Use proper time-to-x conversion based on viewport
        // For now, use simple linear mapping
        let chart_width = self.viewport.dimensions.width as f64;
        let visible_bars = 100.0; // TODO: Calculate from viewport time range
        (index as f64 / visible_bars) * chart_width
    }

    /// Calculate bar width based on visible candles
    fn calculate_bar_width(&self, num_candles: usize) -> f64 {
        if num_candles == 0 {
            return 10.0;
        }
        let chart_width = self.viewport.dimensions.width as f64;
        (chart_width / num_candles as f64) * 0.8
    }

    /// Apply alpha transparency to color
    fn apply_alpha(&self, mut color: Color, alpha: f64) -> Color {
        // Clamp alpha between 0.0 and 1.0
        color.a = (alpha.max(0.0).min(1.0)) as f32;
        color
    }

    /// Draw circle marker
    fn draw_circle(
        &self,
        x: f64,
        y: f64,
        size: f64,
        color: &Color,
        buffer: &mut RenderCommandBuffer,
    ) {
        // Approximate circle with small rectangle for now
        // TODO: Add DrawCircle command to RenderCommand
        let radius = size / 2.0;
        buffer.push(RenderCommand::DrawRect {
            x: x - radius,
            y: y - radius,
            width: size,
            height: size,
            fill: Some(color.clone()),
            stroke: None,
            stroke_width: 0.0,
        });
    }

    /// Draw triangle marker
    fn draw_triangle(
        &self,
        x: f64,
        y: f64,
        size: f64,
        color: &Color,
        buffer: &mut RenderCommandBuffer,
    ) {
        // Draw triangle as three lines
        let half = size / 2.0;
        let height = (size * 0.866) as f64; // sqrt(3)/2

        // Top vertex
        let x1 = x;
        let y1 = y - height / 1.5;

        // Bottom left
        let x2 = x - half;
        let y2 = y + height / 3.0;

        // Bottom right
        let x3 = x + half;
        let y3 = y + height / 3.0;

        buffer.push(RenderCommand::DrawLine {
            x1,
            y1,
            x2,
            y2,
            color: color.clone(),
            width: 2.0,
        });

        buffer.push(RenderCommand::DrawLine {
            x1: x2,
            y1: y2,
            x2: x3,
            y2: y3,
            color: color.clone(),
            width: 2.0,
        });

        buffer.push(RenderCommand::DrawLine {
            x1: x3,
            y1: y3,
            x2: x1,
            y2: y1,
            color: color.clone(),
            width: 2.0,
        });
    }

    /// Draw cross marker (+ shape)
    fn draw_cross(
        &self,
        x: f64,
        y: f64,
        size: f64,
        color: &Color,
        buffer: &mut RenderCommandBuffer,
    ) {
        let half = size / 2.0;

        // Vertical line
        buffer.push(RenderCommand::DrawLine {
            x1: x,
            y1: y - half,
            x2: x,
            y2: y + half,
            color: color.clone(),
            width: 2.0,
        });

        // Horizontal line
        buffer.push(RenderCommand::DrawLine {
            x1: x - half,
            y1: y,
            x2: x + half,
            y2: y,
            color: color.clone(),
            width: 2.0,
        });
    }

    /// Draw diamond marker
    fn draw_diamond(
        &self,
        x: f64,
        y: f64,
        size: f64,
        color: &Color,
        buffer: &mut RenderCommandBuffer,
    ) {
        let half = size / 2.0;

        // Top
        let x1 = x;
        let y1 = y - half;

        // Right
        let x2 = x + half;
        let y2 = y;

        // Bottom
        let x3 = x;
        let y3 = y + half;

        // Left
        let x4 = x - half;
        let y4 = y;

        // Draw four lines
        buffer.push(RenderCommand::DrawLine {
            x1,
            y1,
            x2,
            y2,
            color: color.clone(),
            width: 2.0,
        });

        buffer.push(RenderCommand::DrawLine {
            x1: x2,
            y1: y2,
            x2: x3,
            y2: y3,
            color: color.clone(),
            width: 2.0,
        });

        buffer.push(RenderCommand::DrawLine {
            x1: x3,
            y1: y3,
            x2: x4,
            y2: y4,
            color: color.clone(),
            width: 2.0,
        });

        buffer.push(RenderCommand::DrawLine {
            x1: x4,
            y1: y4,
            x2: x1,
            y2: y1,
            color: color.clone(),
            width: 2.0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::viewport::{Dimensions, PriceRange, TimeRange};

    fn create_test_viewport() -> Viewport {
        let mut viewport = Viewport::new(800, 600);
        viewport.price = PriceRange {
            min: 100.0,
            max: 200.0,
        };
        viewport.time = TimeRange {
            start: 0,
            end: 1000,
        };
        viewport
    }

    #[test]
    fn test_render_single_line() {
        let viewport = create_test_viewport();
        let renderer = IndicatorRenderer::new(&viewport);
        let mut buffer = RenderCommandBuffer::new(0);

        let values = vec![Some(150.0), Some(155.0), Some(160.0)];
        let output = IndicatorOutput::SingleLine {
            values,
            color: Color::rgb(255, 0, 0),
            width: 2.0,
            style: LineStyle::Solid,
        };

        renderer.render(&output, &mut buffer);

        // Should have at least 2 line segments (3 points = 2 segments)
        assert!(buffer.len() >= 2);
    }

    #[test]
    fn test_render_histogram() {
        let viewport = create_test_viewport();
        let renderer = IndicatorRenderer::new(&viewport);
        let mut buffer = RenderCommandBuffer::new(0);

        let values = vec![10.0, -5.0, 15.0, -10.0];
        let output = IndicatorOutput::Histogram {
            values,
            positive_color: Color::rgb(0, 255, 0),
            negative_color: Color::rgb(255, 0, 0),
            zero_line: true,
        };

        renderer.render(&output, &mut buffer);

        // Should have 4 bars + 1 zero line = 5 commands
        assert_eq!(buffer.len(), 5);
    }

    #[test]
    fn test_render_with_gaps() {
        let viewport = create_test_viewport();
        let renderer = IndicatorRenderer::new(&viewport);
        let mut buffer = RenderCommandBuffer::new(0);

        let values = vec![Some(150.0), Some(155.0), None, Some(160.0), Some(165.0)];
        let output = IndicatorOutput::SingleLine {
            values,
            color: Color::rgb(255, 0, 0),
            width: 2.0,
            style: LineStyle::Solid,
        };

        renderer.render(&output, &mut buffer);

        // Should break into 2 segments: [150,155] and [160,165]
        // Segment 1: 1 line, Segment 2: 1 line = 2 lines total
        assert!(buffer.len() >= 2);
    }

    #[test]
    fn test_apply_alpha() {
        let viewport = create_test_viewport();
        let renderer = IndicatorRenderer::new(&viewport);

        let color = Color::rgb(255, 0, 0);
        let transparent = renderer.apply_alpha(color, 0.5);

        assert_eq!(transparent.a, 0.5);
    }

    #[test]
    fn test_scatter_render() {
        let viewport = create_test_viewport();
        let renderer = IndicatorRenderer::new(&viewport);
        let mut buffer = RenderCommandBuffer::new(0);

        let points = vec![ScatterPoint::new(10, 150.0), ScatterPoint::new(20, 160.0)];

        let output = IndicatorOutput::Scatter {
            points,
            color: Color::rgb(0, 0, 255),
            size: 8.0,
            shape: MarkerShape::Circle,
        };

        renderer.render(&output, &mut buffer);

        // Should have 2 markers
        assert_eq!(buffer.len(), 2);
    }
}
