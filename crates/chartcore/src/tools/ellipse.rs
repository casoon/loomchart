use super::{ChartTool, ToolNode, ToolType};
use crate::core::Viewport;
#[cfg(feature = "wasm")]
use crate::rendering::{Canvas2DRenderer, Renderer};
use crate::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ellipse {
    id: String,
    nodes: Vec<ToolNode>, // 2 nodes: corner1, corner2 of bounding box
    color: Color,
    fill_color: Option<Color>,
    width: f64,
}

impl Ellipse {
    pub fn with_corners(id: String, corner1: ToolNode, corner2: ToolNode) -> Self {
        Self {
            id,
            nodes: vec![corner1, corner2],
            color: Color::rgba(33, 150, 243, 1.0),
            fill_color: Some(Color::rgba(33, 150, 243, 0.08)),
            width: 1.5,
        }
    }
}

impl ChartTool for Ellipse {
    fn id(&self) -> &str {
        &self.id
    }

    fn tool_type(&self) -> ToolType {
        ToolType::Ellipse
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
        let x1 = viewport.time_to_x(self.nodes[0].time);
        let y1 = viewport.price_to_y(self.nodes[0].price);
        let x2 = viewport.time_to_x(self.nodes[1].time);
        let y2 = viewport.price_to_y(self.nodes[1].price);

        let cx = (x1 + x2) / 2.0;
        let cy = (y1 + y2) / 2.0;
        let rx = (x2 - x1).abs() / 2.0;
        let ry = (y2 - y1).abs() / 2.0;

        if rx < 1.0 || ry < 1.0 {
            return;
        }

        if let Some(fill) = self.fill_color {
            renderer.fill_ellipse(cx, cy, rx, ry, fill);
        }
        renderer.stroke_ellipse(cx, cy, rx, ry, self.color, self.width as f32);

        // Corner handles
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
        let x1 = viewport.time_to_x(self.nodes[0].time);
        let y1 = viewport.price_to_y(self.nodes[0].price);
        let x2 = viewport.time_to_x(self.nodes[1].time);
        let y2 = viewport.price_to_y(self.nodes[1].price);

        let cx = (x1 + x2) / 2.0;
        let cy = (y1 + y2) / 2.0;
        let rx = (x2 - x1).abs() / 2.0 + 8.0;
        let ry = (y2 - y1).abs() / 2.0 + 8.0;

        if rx < 1.0 || ry < 1.0 {
            return false;
        }

        // Ellipse hit test: (dx/rx)^2 + (dy/ry)^2 <= 1
        let dx = x - cx;
        let dy = y - cy;
        let outer = (dx / rx).powi(2) + (dy / ry).powi(2);

        // Also check inner boundary (threshold ~8px inward)
        let rx_inner = (rx - 16.0).max(0.0);
        let ry_inner = (ry - 16.0).max(0.0);
        let inner = if rx_inner > 0.0 && ry_inner > 0.0 {
            (dx / rx_inner).powi(2) + (dy / ry_inner).powi(2)
        } else {
            0.0
        };

        outer <= 1.0 && inner >= 1.0
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
