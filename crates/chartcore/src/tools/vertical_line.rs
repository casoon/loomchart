use super::{ChartTool, ToolNode, ToolType};
use crate::core::Viewport;
use crate::rendering::{Canvas2DRenderer, Renderer, TextAlign, TextBaseline};
use crate::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerticalLine {
    id: String,
    nodes: Vec<ToolNode>, // Only needs 1 node for time position
    color: Color,
    width: f64,
}

impl VerticalLine {
    pub fn new(id: String) -> Self {
        Self {
            id,
            nodes: Vec::new(),
            color: Color::rgba(255, 152, 0, 1.0), // Orange
            width: 1.5,
        }
    }

    pub fn with_time(id: String, time: i64, price: f64) -> Self {
        Self {
            id,
            nodes: vec![ToolNode::new(time, price)],
            color: Color::rgba(255, 152, 0, 1.0),
            width: 1.5,
        }
    }

    pub fn add_node(&mut self, node: ToolNode) {
        if self.nodes.is_empty() {
            self.nodes.push(node);
        }
    }

    pub fn time(&self) -> Option<i64> {
        self.nodes.first().map(|n| n.time)
    }
}

impl ChartTool for VerticalLine {
    fn id(&self) -> &str {
        &self.id
    }

    fn tool_type(&self) -> ToolType {
        ToolType::VerticalLine
    }

    fn nodes(&self) -> &[ToolNode] {
        &self.nodes
    }

    fn render(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport) {
        if let Some(time) = self.time() {
            let x = viewport.time_to_x(time);
            let height = viewport.dimensions.height as f64;

            // Draw vertical line from top to bottom
            renderer.draw_vertical_line(x, 0.0, height, &self.color, self.width);

            // Draw time label at bottom
            let label_y = viewport.dimensions.height as f64 - 25.0;
            let date = chrono::DateTime::from_timestamp(time, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| time.to_string());

            renderer.draw_text(
                &date,
                x + 5.0,
                label_y,
                self.color,
                11.0,
                TextAlign::Left,
                TextBaseline::Bottom,
            );
        }
    }

    fn hit_test(&self, x: f64, _y: f64, viewport: &Viewport) -> bool {
        const HIT_THRESHOLD: f64 = 6.0;

        if let Some(time) = self.time() {
            let line_x = viewport.time_to_x(time);
            (x - line_x).abs() <= HIT_THRESHOLD
        } else {
            false
        }
    }

    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| format!("VerticalLine serialization error: {}", e))
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
