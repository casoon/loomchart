use super::{ChartTool, ToolNode, ToolType};
use crate::core::Viewport;
use crate::rendering::{Canvas2DRenderer, Renderer};
use crate::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendLine {
    id: String,
    nodes: Vec<ToolNode>,
    color: Color,
    width: f64,
    extend: bool, // Extend line to edges
}

impl TrendLine {
    pub fn new(id: String) -> Self {
        Self {
            id,
            nodes: Vec::new(),
            color: Color::rgba(33, 150, 243, 1.0), // Blue
            width: 2.0,
            extend: false,
        }
    }

    pub fn with_nodes(id: String, start: ToolNode, end: ToolNode) -> Self {
        Self {
            id,
            nodes: vec![start, end],
            color: Color::rgba(33, 150, 243, 1.0),
            width: 2.0,
            extend: false,
        }
    }

    pub fn add_node(&mut self, node: ToolNode) {
        if self.nodes.len() < 2 {
            self.nodes.push(node);
        }
    }

    pub fn set_extend(&mut self, extend: bool) {
        self.extend = extend;
    }

    /// Calculate distance from point to line segment
    fn distance_to_line(&self, x: f64, y: f64, viewport: &Viewport) -> f64 {
        if self.nodes.len() < 2 {
            return f64::INFINITY;
        }

        let x1 = viewport.time_to_x(self.nodes[0].time);
        let y1 = viewport.price_to_y(self.nodes[0].price);
        let x2 = viewport.time_to_x(self.nodes[1].time);
        let y2 = viewport.price_to_y(self.nodes[1].price);

        // Vector from point1 to point2
        let dx = x2 - x1;
        let dy = y2 - y1;

        if dx.abs() < 0.001 && dy.abs() < 0.001 {
            // Points are the same
            return ((x - x1).powi(2) + (y - y1).powi(2)).sqrt();
        }

        // Calculate projection parameter
        let t = ((x - x1) * dx + (y - y1) * dy) / (dx * dx + dy * dy);

        let (closest_x, closest_y) = if !self.extend {
            // Clamp to segment
            let t_clamped = t.clamp(0.0, 1.0);
            (x1 + t_clamped * dx, y1 + t_clamped * dy)
        } else {
            // Extend infinitely
            (x1 + t * dx, y1 + t * dy)
        };

        ((x - closest_x).powi(2) + (y - closest_y).powi(2)).sqrt()
    }
}

impl ChartTool for TrendLine {
    fn id(&self) -> &str {
        &self.id
    }

    fn tool_type(&self) -> ToolType {
        ToolType::TrendLine
    }

    fn nodes(&self) -> &[ToolNode] {
        &self.nodes
    }

    fn render(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport) {
        if self.nodes.len() < 2 {
            return;
        }

        let x1 = viewport.time_to_x(self.nodes[0].time);
        let y1 = viewport.price_to_y(self.nodes[0].price);
        let x2 = viewport.time_to_x(self.nodes[1].time);
        let y2 = viewport.price_to_y(self.nodes[1].price);

        if self.extend {
            // Calculate line equation: y = mx + b
            let dx = x2 - x1;
            let dy = y2 - y1;

            if dx.abs() > 0.001 {
                let slope = dy / dx;
                let intercept = y1 - slope * x1;

                // Extend to viewport edges
                let x_start = 0.0;
                let x_end = viewport.dimensions.width as f64;
                let y_start = slope * x_start + intercept;
                let y_end = slope * x_end + intercept;

                renderer.draw_line(
                    x_start,
                    y_start,
                    x_end,
                    y_end,
                    self.color,
                    self.width as f32,
                );
            } else {
                // Vertical line
                let height = viewport.dimensions.height as f64;
                renderer.draw_vertical_line(x1, 0.0, height, &self.color, self.width);
            }
        } else {
            // Draw line segment
            renderer.draw_line(x1, y1, x2, y2, self.color, self.width as f32);
        }

        // Draw nodes (small circles)
        let node_radius = 4.0;
        for node in &self.nodes {
            let x = viewport.time_to_x(node.time);
            let y = viewport.price_to_y(node.price);
            renderer.draw_circle(x, y, node_radius, self.color);
        }
    }

    fn hit_test(&self, x: f64, y: f64, viewport: &Viewport) -> bool {
        const HIT_THRESHOLD: f64 = 8.0; // pixels

        let distance = self.distance_to_line(x, y, viewport);
        distance <= HIT_THRESHOLD
    }

    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("TrendLine serialization error: {}", e))
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
