use super::{ChartTool, ToolNode, ToolType};
use crate::core::Viewport;
#[cfg(feature = "wasm")]
use crate::rendering::{Canvas2DRenderer, Renderer};
use crate::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rectangle {
    id: String,
    nodes: Vec<ToolNode>, // 2 nodes: first corner, opposite corner
    color: Color,
    fill_color: Option<Color>,
    width: f64,
}

impl Rectangle {
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

impl ChartTool for Rectangle {
    fn id(&self) -> &str {
        &self.id
    }

    fn tool_type(&self) -> ToolType {
        ToolType::Rectangle
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

        let x = x1.min(x2);
        let y = y1.min(y2);
        let w = (x2 - x1).abs();
        let h = (y2 - y1).abs();

        if let Some(fill) = self.fill_color {
            renderer.fill_rect(x, y, w, h, fill);
        }
        renderer.stroke_rect(x, y, w, h, self.color, self.width as f32);

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
        const THRESHOLD: f64 = 8.0;
        let x1 = viewport.time_to_x(self.nodes[0].time);
        let y1 = viewport.price_to_y(self.nodes[0].price);
        let x2 = viewport.time_to_x(self.nodes[1].time);
        let y2 = viewport.price_to_y(self.nodes[1].price);
        let left = x1.min(x2);
        let right = x1.max(x2);
        let top = y1.min(y2);
        let bottom = y1.max(y2);
        let inside = x >= left - THRESHOLD
            && x <= right + THRESHOLD
            && y >= top - THRESHOLD
            && y <= bottom + THRESHOLD;
        let on_border = x <= left + THRESHOLD
            || x >= right - THRESHOLD
            || y <= top + THRESHOLD
            || y >= bottom - THRESHOLD;
        inside && on_border
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
