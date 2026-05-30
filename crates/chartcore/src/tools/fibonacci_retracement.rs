use super::{ChartTool, ToolNode, ToolType};
use crate::core::Viewport;
#[cfg(feature = "wasm")]
use crate::rendering::{Canvas2DRenderer, Renderer, TextAlign, TextBaseline};
use crate::Color;
use serde::{Deserialize, Serialize};

/// Standard Fibonacci levels (0–100) plus common extensions
const FIB_LEVELS: &[(f64, &str)] = &[
    (0.0, "0"),
    (0.236, "23.6"),
    (0.382, "38.2"),
    (0.500, "50.0"),
    (0.618, "61.8"),
    (0.786, "78.6"),
    (1.000, "100"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FibonacciRetracement {
    id: String,
    nodes: Vec<ToolNode>, // 2 nodes: high point, low point
    color: Color,
    width: f64,
}

impl FibonacciRetracement {
    pub fn with_points(id: String, high: ToolNode, low: ToolNode) -> Self {
        Self {
            id,
            nodes: vec![high, low],
            color: Color::rgba(255, 178, 66, 1.0), // Amber
            width: 1.0,
        }
    }
}

impl ChartTool for FibonacciRetracement {
    fn id(&self) -> &str {
        &self.id
    }

    fn tool_type(&self) -> ToolType {
        ToolType::FibonacciRetracement
    }

    fn nodes(&self) -> &[ToolNode] {
        &self.nodes
    }

    fn nodes_mut(&mut self) -> &mut Vec<ToolNode> {
        &mut self.nodes
    }

    #[cfg(feature = "wasm")]
    fn render(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport) {
        if self.nodes.len() < 2 {
            return;
        }

        let p1 = &self.nodes[0];
        let p2 = &self.nodes[1];
        let price_high = p1.price.max(p2.price);
        let price_low = p1.price.min(p2.price);
        let price_range = price_high - price_low;

        let chart_width = viewport.dimensions.width as f64;
        let label_x = chart_width - 4.0;

        for (ratio, label) in FIB_LEVELS {
            let price = price_high - ratio * price_range;
            let y = viewport.price_to_y(price);

            // Line color: highlight 61.8 and 38.2 as key levels
            let line_color = if (*ratio - 0.618).abs() < 0.001 || (*ratio - 0.382).abs() < 0.001 {
                Color::rgba(255, 178, 66, 0.9)
            } else {
                Color::rgba(255, 178, 66, 0.5)
            };

            renderer.draw_line(0.0, y, chart_width, y, line_color, self.width as f32);

            // Price label on right edge
            let price_str = format!("{:.2}  {}", price, label);
            renderer.draw_text(
                &price_str,
                label_x,
                y - 2.0,
                self.color,
                10.0,
                TextAlign::Right,
                TextBaseline::Bottom,
            );
        }

        // Anchor handles
        for node in &self.nodes {
            let nx = viewport.time_to_x(node.time);
            let ny = viewport.price_to_y(node.price);
            renderer.draw_circle(nx, ny, 4.0, self.color);
        }
    }

    fn hit_test(&self, x: f64, y: f64, viewport: &Viewport) -> bool {
        if self.nodes.len() < 2 {
            return false;
        }
        const THRESHOLD: f64 = 8.0;
        let p1 = &self.nodes[0];
        let p2 = &self.nodes[1];
        let price_high = p1.price.max(p2.price);
        let price_low = p1.price.min(p2.price);
        let price_range = price_high - price_low;
        let chart_width = viewport.dimensions.width as f64;

        if x < 0.0 || x > chart_width {
            return false;
        }
        for (ratio, _) in FIB_LEVELS {
            let price = price_high - ratio * price_range;
            let line_y = viewport.price_to_y(price);
            if (y - line_y).abs() <= THRESHOLD {
                return true;
            }
        }
        false
    }

    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    fn is_complete(&self) -> bool {
        self.nodes.len() >= 2
    }

    fn color(&self) -> Color {
        self.color
    }

    fn set_color(&mut self, color: Color) {
        self.color = color;
    }
}
