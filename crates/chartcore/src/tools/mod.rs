use crate::core::Viewport;
use crate::rendering::Canvas2DRenderer;
use crate::Color;
use serde::{Deserialize, Serialize};

pub mod horizontal_line;
pub mod trendline;
pub mod vertical_line;

// Re-export tool types for convenience
pub use horizontal_line::HorizontalLine;
pub use trendline::TrendLine;
pub use vertical_line::VerticalLine;

/// Tool node - represents a point in price/time space
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolNode {
    pub time: i64,  // Unix timestamp in seconds
    pub price: f64, // Price level
}

impl ToolNode {
    pub fn new(time: i64, price: f64) -> Self {
        Self { time, price }
    }
}

/// Tool type identifier
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolType {
    TrendLine,
    HorizontalLine,
    VerticalLine,
}

/// Base trait for all drawing tools
pub trait ChartTool: Send + Sync {
    /// Get tool ID
    fn id(&self) -> &str;

    /// Get tool type
    fn tool_type(&self) -> ToolType;

    /// Get tool nodes
    fn nodes(&self) -> &[ToolNode];

    /// Render the tool
    fn render(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport);

    /// Hit test - check if point is on/near the tool
    fn hit_test(&self, x: f64, y: f64, viewport: &Viewport) -> bool;

    /// Serialize to JSON
    fn to_json(&self) -> Result<String, String>;

    /// Check if tool is complete (has all required nodes)
    fn is_complete(&self) -> bool;

    /// Get color
    fn color(&self) -> Color;

    /// Set color
    fn set_color(&mut self, color: Color);
}

/// Tool manager - manages all drawing tools
pub struct ToolManager {
    tools: Vec<Box<dyn ChartTool>>,
    active_tool_id: Option<String>,
    next_id: u32,
}

impl ToolManager {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            active_tool_id: None,
            next_id: 1,
        }
    }

    /// Add a new tool
    pub fn add_tool(&mut self, tool: Box<dyn ChartTool>) {
        self.tools.push(tool);
    }

    /// Get tool by ID
    pub fn get_tool(&self, id: &str) -> Option<&dyn ChartTool> {
        self.tools.iter().find(|t| t.id() == id).map(|t| t.as_ref())
    }

    /// Get mutable tool by ID
    pub fn get_tool_mut(&mut self, id: &str) -> Option<&mut Box<dyn ChartTool>> {
        self.tools.iter_mut().find(|t| t.id() == id)
    }

    /// Remove tool by ID
    pub fn remove_tool(&mut self, id: &str) -> bool {
        if let Some(pos) = self.tools.iter().position(|t| t.id() == id) {
            self.tools.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get all tools
    pub fn tools(&self) -> &[Box<dyn ChartTool>] {
        &self.tools
    }

    /// Get active tool ID
    pub fn active_tool_id(&self) -> Option<&str> {
        self.active_tool_id.as_deref()
    }

    /// Set active tool
    pub fn set_active_tool(&mut self, id: Option<String>) {
        self.active_tool_id = id;
    }

    /// Generate next tool ID
    pub fn generate_id(&mut self, prefix: &str) -> String {
        let id = format!("{}-{}", prefix, self.next_id);
        self.next_id += 1;
        id
    }

    /// Render all tools
    pub fn render_all(&self, renderer: &mut Canvas2DRenderer, viewport: &Viewport) {
        for tool in &self.tools {
            if tool.is_complete() {
                tool.render(renderer, viewport);
            }
        }
    }

    /// Hit test all tools
    pub fn hit_test(&self, x: f64, y: f64, viewport: &Viewport) -> Option<&str> {
        for tool in self.tools.iter().rev() {
            if tool.hit_test(x, y, viewport) {
                return Some(tool.id());
            }
        }
        None
    }

    /// Export all tools to JSON array
    pub fn export_tools(&self) -> Result<String, String> {
        let json_tools: Result<Vec<String>, String> =
            self.tools.iter().map(|t| t.to_json()).collect();

        match json_tools {
            Ok(tools) => {
                let combined = format!("[{}]", tools.join(","));
                Ok(combined)
            }
            Err(e) => Err(format!("Tool export error: {}", e)),
        }
    }

    /// Clear all tools
    pub fn clear(&mut self) {
        self.tools.clear();
        self.active_tool_id = None;
    }

    /// Get tool count
    pub fn count(&self) -> usize {
        self.tools.len()
    }
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}
