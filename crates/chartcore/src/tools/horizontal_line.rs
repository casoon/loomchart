use super::{ChartTool, ToolNode, ToolType};
use crate::core::Viewport;
use crate::rendering::{Canvas2DRenderer, Renderer, TextAlign, TextBaseline};
use crate::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HorizontalLine {
    id: String,
    nodes: Vec<ToolNode>, // Only needs 1 node for price level
    color: Color,
    width: f64,
    style: LineStyle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
}

impl HorizontalLine {
    pub fn new(id: String) -> Self {
        Self {
            id,
            nodes: Vec::new(),
            color: Color::rgba(156, 39, 176, 1.0), // Purple
            width: 1.5,
            style: LineStyle::Dashed,
        }
    }

    pub fn with_price(id: String, time: i64, price: f64) -> Self {
        Self {
            id,
            nodes: vec![ToolNode::new(time, price)],
            color: Color::rgba(156, 39, 176, 1.0),
            width: 1.5,
            style: LineStyle::Dashed,
        }
    }

    pub fn add_node(&mut self, node: ToolNode) {
        if self.nodes.is_empty() {
            self.nodes.push(node);
        }
    }

    pub fn set_style(&mut self, style: LineStyle) {
        self.style = style;
    }

    pub fn price(&self) -> Option<f64> {
        self.nodes.first().map(|n| n.price)
    }
}

impl ChartTool for HorizontalLine {
    fn id(&self) -> &str {
        &self.id
    }

    fn tool_type(&self) -> ToolType {
        ToolType::HorizontalLine
    }

    fn nodes(&self) -> &[ToolNode] {
        &self.nodes
    }

    fn render(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport) {
        if let Some(price) = self.price() {
            let y = viewport.price_to_y(price);

            match self.style {
                LineStyle::Solid => {
                    let width = viewport.dimensions.width as f64;
                    renderer.draw_line(0.0, y, width, y, self.color, self.width as f32);
                }
                LineStyle::Dashed => {
                    self.draw_dashed_horizontal(renderer, viewport, y);
                }
                LineStyle::Dotted => {
                    self.draw_dotted_horizontal(renderer, viewport, y);
                }
            }

            // Draw price label on the line
            let label_x = viewport.dimensions.width as f64 - 80.0;
            renderer.draw_text(
                &format!("{:.2}", price),
                label_x,
                y - 12.0,
                self.color,
                12.0,
                TextAlign::Right,
                TextBaseline::Bottom,
            );
        }
    }

    fn hit_test(&self, _x: f64, y: f64, viewport: &Viewport) -> bool {
        const HIT_THRESHOLD: f64 = 6.0;

        if let Some(price) = self.price() {
            let line_y = viewport.price_to_y(price);
            (y - line_y).abs() <= HIT_THRESHOLD
        } else {
            false
        }
    }

    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self)
            .map_err(|e| format!("HorizontalLine serialization error: {}", e))
    }

    fn is_complete(&self) -> bool {
        !self.nodes.is_empty()
    }

    fn color(&self) -> Color {
        self.color
    }

    fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}

impl HorizontalLine {
    fn draw_dashed_horizontal(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport, y: f64) {
        let width = viewport.dimensions.width as f64;
        let dash_length = 8.0;
        let gap_length = 4.0;
        let pattern_length = dash_length + gap_length;

        let mut x = 0.0;
        while x < width {
            let x_end = (x + dash_length).min(width);
            renderer.draw_line(x, y, x_end, y, self.color, self.width as f32);
            x += pattern_length;
        }
    }

    fn draw_dotted_horizontal(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport, y: f64) {
        let width = viewport.dimensions.width as f64;
        let dot_spacing = 6.0;

        let mut x = 0.0;
        while x < width {
            renderer.draw_circle(x, y, 1.5, self.color);
            x += dot_spacing;
        }
    }
}
