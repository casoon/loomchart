use super::{ChartTool, ToolNode, ToolType};
use crate::core::Viewport;
#[cfg(feature = "wasm")]
use crate::rendering::{Canvas2DRenderer, Renderer, TextAlign, TextBaseline};
use crate::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextLabel {
    id: String,
    nodes: Vec<ToolNode>, // 1 node: anchor point
    text: String,
    color: Color,
    font_size: f32,
    background: Option<Color>,
}

impl TextLabel {
    pub fn new(id: String, anchor: ToolNode, text: impl Into<String>) -> Self {
        Self {
            id,
            nodes: vec![anchor],
            text: text.into(),
            color: Color::rgba(231, 233, 234, 1.0),
            font_size: 12.0,
            background: Some(Color::rgba(30, 40, 60, 0.8)),
        }
    }
}

impl ChartTool for TextLabel {
    fn id(&self) -> &str {
        &self.id
    }

    fn tool_type(&self) -> ToolType {
        ToolType::TextLabel
    }

    fn nodes(&self) -> &[ToolNode] {
        &self.nodes
    }

    fn nodes_mut(&mut self) -> &mut Vec<ToolNode> {
        &mut self.nodes
    }

    #[cfg(feature = "wasm")]
    fn render(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport) {
        if self.nodes.is_empty() {
            return;
        }
        let x = viewport.time_to_x(self.nodes[0].time);
        let y = viewport.price_to_y(self.nodes[0].price);

        // Simple background pill
        if let Some(bg) = self.background {
            let text_w = self.text.len() as f64 * (self.font_size as f64 * 0.6);
            let pad = 4.0;
            renderer.fill_rect(x, y - self.font_size as f64 - pad, text_w + pad * 2.0, self.font_size as f64 + pad * 2.0, bg);
        }

        renderer.draw_text(
            &self.text,
            x + 4.0,
            y - 4.0,
            self.color,
            self.font_size,
            TextAlign::Left,
            TextBaseline::Bottom,
        );

        // Anchor dot
        renderer.draw_circle(x, y, 3.0, self.color);
    }

    fn hit_test(&self, x: f64, y: f64, viewport: &Viewport) -> bool {
        if self.nodes.is_empty() {
            return false;
        }
        const THRESHOLD: f64 = 20.0;
        let nx = viewport.time_to_x(self.nodes[0].time);
        let ny = viewport.price_to_y(self.nodes[0].price);
        (x - nx).powi(2) + (y - ny).powi(2) <= THRESHOLD.powi(2)
    }

    fn to_json(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
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
