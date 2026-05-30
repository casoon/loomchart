use crate::core::Viewport;
#[cfg(feature = "wasm")]
use crate::rendering::Canvas2DRenderer;
use crate::Color;
use serde::{Deserialize, Serialize};

pub mod fibonacci_retracement;
pub mod horizontal_line;
pub mod rectangle;
pub mod text_label;
pub mod trendline;
pub mod vertical_line;

// Re-export tool types for convenience
pub use fibonacci_retracement::FibonacciRetracement;
pub use horizontal_line::HorizontalLine;
pub use rectangle::Rectangle;
pub use text_label::TextLabel;
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
    Rectangle,
    FibonacciRetracement,
    TextLabel,
}

/// Base trait for all drawing tools
pub trait ChartTool: Send + Sync {
    /// Get tool ID
    fn id(&self) -> &str;

    /// Get tool type
    fn tool_type(&self) -> ToolType;

    /// Get tool nodes
    fn nodes(&self) -> &[ToolNode];

    /// Get mutable tool nodes
    fn nodes_mut(&mut self) -> &mut Vec<ToolNode>;

    /// Render the tool
    #[cfg(feature = "wasm")]
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

/// Envelope used for type-tagged serialization in to_json / from_json
#[derive(Serialize, Deserialize)]
struct ToolEnvelope {
    tool_type: ToolType,
    data: serde_json::Value,
}

/// Tool manager - manages all drawing tools
pub struct ToolManager {
    tools: Vec<Box<dyn ChartTool>>,
    active_tool_id: Option<String>,
    selected_id: Option<String>,
    next_id: u32,
}

impl ToolManager {
    pub fn new() -> Self {
        Self {
            tools: Vec::new(),
            active_tool_id: None,
            selected_id: None,
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
    #[cfg(feature = "wasm")]
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
        self.selected_id = None;
    }

    /// Get tool count
    pub fn count(&self) -> usize {
        self.tools.len()
    }

    // --- Selection state ---

    /// Select a tool by ID. Returns true if the tool exists.
    pub fn select(&mut self, id: &str) -> bool {
        if self.tools.iter().any(|t| t.id() == id) {
            self.selected_id = Some(id.to_string());
            true
        } else {
            false
        }
    }

    /// Deselect all tools.
    pub fn deselect_all(&mut self) {
        self.selected_id = None;
    }

    /// Return the currently selected tool ID, if any.
    pub fn selected_id(&self) -> Option<&str> {
        self.selected_id.as_deref()
    }

    /// Shift every node of the selected tool by (dt, dp).
    pub fn move_selected(&mut self, dt: i64, dp: f64) {
        if let Some(ref id) = self.selected_id.clone() {
            if let Some(tool) = self.tools.iter_mut().find(|t| t.id() == id) {
                for node in tool.nodes_mut().iter_mut() {
                    node.time += dt;
                    node.price += dp;
                }
            }
        }
    }

    /// Remove and return the selected tool, clearing selection.
    pub fn delete_selected(&mut self) -> Option<Box<dyn ChartTool>> {
        let id = self.selected_id.take()?;
        let pos = self.tools.iter().position(|t| t.id() == id)?;
        Some(self.tools.remove(pos))
    }

    // --- Snap behavior ---

    /// Snap (time, price) to the nearest OHLC point of a candle if within
    /// threshold_px pixels. Returns (time, price) of the snap target or the
    /// original point if nothing is close enough.
    pub fn snap_to_candle(
        &self,
        time: i64,
        price: f64,
        candles: &[crate::core::Candle],
        threshold_px: f64,
        viewport: &Viewport,
    ) -> (i64, f64) {
        let px = viewport.time_to_x(time);
        let py = viewport.price_to_y(price);

        let mut best_dist = threshold_px;
        let mut result = (time, price);

        for candle in candles {
            let cx = viewport.time_to_x(candle.time);
            // Check each OHLC level
            for &cp in &[candle.o, candle.h, candle.l, candle.c] {
                let cy = viewport.price_to_y(cp);
                let dist = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
                if dist < best_dist {
                    best_dist = dist;
                    result = (candle.time, cp);
                }
            }
        }

        result
    }

    // --- Serialization round-trip ---

    /// Serialize all tools to a JSON string.
    pub fn to_json(&self) -> Result<String, String> {
        let mut envelopes: Vec<ToolEnvelope> = Vec::with_capacity(self.tools.len());
        for tool in &self.tools {
            let raw = tool.to_json()?;
            let data: serde_json::Value = serde_json::from_str(&raw)
                .map_err(|e| format!("Failed to parse tool JSON: {}", e))?;
            envelopes.push(ToolEnvelope {
                tool_type: tool.tool_type(),
                data,
            });
        }
        serde_json::to_string(&envelopes).map_err(|e| format!("ToolManager serialization error: {}", e))
    }

    /// Restore a `ToolManager` from a JSON string produced by `to_json`.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let envelopes: Vec<ToolEnvelope> = serde_json::from_str(json)
            .map_err(|e| format!("ToolManager deserialization error: {}", e))?;

        let mut manager = Self::new();
        for env in envelopes {
            let tool: Box<dyn ChartTool> = match env.tool_type {
                ToolType::HorizontalLine => {
                    let t: HorizontalLine = serde_json::from_value(env.data)
                        .map_err(|e| format!("HorizontalLine deserialization error: {}", e))?;
                    Box::new(t)
                }
                ToolType::TrendLine => {
                    let t: TrendLine = serde_json::from_value(env.data)
                        .map_err(|e| format!("TrendLine deserialization error: {}", e))?;
                    Box::new(t)
                }
                ToolType::VerticalLine => {
                    let t: VerticalLine = serde_json::from_value(env.data)
                        .map_err(|e| format!("VerticalLine deserialization error: {}", e))?;
                    Box::new(t)
                }
                ToolType::Rectangle => {
                    let t: Rectangle = serde_json::from_value(env.data)
                        .map_err(|e| format!("Rectangle deserialization error: {}", e))?;
                    Box::new(t)
                }
                ToolType::FibonacciRetracement => {
                    let t: FibonacciRetracement = serde_json::from_value(env.data)
                        .map_err(|e| format!("FibonacciRetracement deserialization error: {}", e))?;
                    Box::new(t)
                }
                ToolType::TextLabel => {
                    let t: TextLabel = serde_json::from_value(env.data)
                        .map_err(|e| format!("TextLabel deserialization error: {}", e))?;
                    Box::new(t)
                }
            };
            manager.tools.push(tool);
        }
        Ok(manager)
    }
}

impl Default for ToolManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Candle, PriceRange, TimeRange, Viewport};

    fn make_viewport() -> Viewport {
        let mut vp = Viewport::new(800, 600);
        vp.time = TimeRange {
            start: 0,
            end: 10_000,
        };
        vp.price = PriceRange {
            min: 90.0,
            max: 110.0,
        };
        vp
    }

    fn make_hline(id: &str, price: f64) -> Box<dyn ChartTool> {
        Box::new(HorizontalLine::with_price(id.to_string(), 1000, price))
    }

    fn make_trendline(id: &str) -> Box<dyn ChartTool> {
        Box::new(TrendLine::with_nodes(
            id.to_string(),
            ToolNode::new(1000, 100.0),
            ToolNode::new(5000, 105.0),
        ))
    }

    // --- select / deselect cycle ---

    #[test]
    fn test_select_existing_tool() {
        let mut mgr = ToolManager::new();
        mgr.add_tool(make_hline("h1", 100.0));
        assert!(mgr.select("h1"));
        assert_eq!(mgr.selected_id(), Some("h1"));
    }

    #[test]
    fn test_select_nonexistent_returns_false() {
        let mut mgr = ToolManager::new();
        assert!(!mgr.select("nope"));
        assert_eq!(mgr.selected_id(), None);
    }

    #[test]
    fn test_deselect_all_clears_selection() {
        let mut mgr = ToolManager::new();
        mgr.add_tool(make_hline("h1", 100.0));
        mgr.select("h1");
        mgr.deselect_all();
        assert_eq!(mgr.selected_id(), None);
    }

    // --- move_selected shifts node coordinates ---

    #[test]
    fn test_move_selected_shifts_nodes() {
        let mut mgr = ToolManager::new();
        mgr.add_tool(make_trendline("t1"));
        mgr.select("t1");
        mgr.move_selected(500, 2.5);

        let tool = mgr.get_tool("t1").unwrap();
        let nodes = tool.nodes();
        assert_eq!(nodes[0].time, 1500);
        assert!((nodes[0].price - 102.5).abs() < 1e-10);
        assert_eq!(nodes[1].time, 5500);
        assert!((nodes[1].price - 107.5).abs() < 1e-10);
    }

    #[test]
    fn test_move_selected_noop_when_nothing_selected() {
        let mut mgr = ToolManager::new();
        mgr.add_tool(make_trendline("t1"));
        // no selection — should not panic or change anything
        mgr.move_selected(999, 99.0);
        let nodes = mgr.get_tool("t1").unwrap().nodes().to_vec();
        assert_eq!(nodes[0].time, 1000);
    }

    // --- delete_selected removes and returns the tool ---

    #[test]
    fn test_delete_selected_removes_and_returns() {
        let mut mgr = ToolManager::new();
        mgr.add_tool(make_hline("h1", 100.0));
        mgr.add_tool(make_hline("h2", 105.0));
        mgr.select("h1");

        let removed = mgr.delete_selected();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id(), "h1");
        assert_eq!(mgr.count(), 1);
        assert_eq!(mgr.selected_id(), None);
    }

    #[test]
    fn test_delete_selected_returns_none_when_nothing_selected() {
        let mut mgr = ToolManager::new();
        mgr.add_tool(make_hline("h1", 100.0));
        assert!(mgr.delete_selected().is_none());
        assert_eq!(mgr.count(), 1);
    }

    // --- snap_to_candle ---

    fn make_candle(time: i64, o: f64, h: f64, l: f64, c: f64) -> Candle {
        Candle::new(time, o, h, l, c, 1000.0)
    }

    #[test]
    fn test_snap_to_candle_returns_close_when_within_threshold() {
        let mgr = ToolManager::new();
        let vp = make_viewport();
        // candle at time=1000, close=100.0
        let candles = vec![make_candle(1000, 99.0, 101.0, 98.0, 100.0)];

        // Query very close to the candle close point
        let (snapped_time, snapped_price) = mgr.snap_to_candle(1000, 100.0, &candles, 20.0, &vp);
        assert_eq!(snapped_time, 1000);
        assert!((snapped_price - 100.0).abs() < 1e-10);
    }

    #[test]
    fn test_snap_to_candle_returns_original_when_outside_threshold() {
        let mgr = ToolManager::new();
        let vp = make_viewport();
        // candle at time=1000, all OHLC around 100
        let candles = vec![make_candle(1000, 99.0, 101.0, 98.0, 100.0)];

        // Query far away (time=9000, price=90) — pixel distance >> threshold
        let (snapped_time, snapped_price) =
            mgr.snap_to_candle(9000, 90.0, &candles, 5.0, &vp);
        assert_eq!(snapped_time, 9000);
        assert!((snapped_price - 90.0).abs() < 1e-10);
    }

    // --- serialization round-trip ---

    #[test]
    fn test_to_json_from_json_round_trip() {
        let mut mgr = ToolManager::new();
        mgr.add_tool(make_hline("h1", 100.0));
        mgr.add_tool(make_trendline("t1"));

        let json = mgr.to_json().expect("to_json failed");
        let restored = ToolManager::from_json(&json).expect("from_json failed");

        assert_eq!(restored.count(), 2);
        assert!(restored.get_tool("h1").is_some());
        assert!(restored.get_tool("t1").is_some());
        assert_eq!(
            restored.get_tool("h1").unwrap().tool_type(),
            ToolType::HorizontalLine
        );
        assert_eq!(
            restored.get_tool("t1").unwrap().tool_type(),
            ToolType::TrendLine
        );
    }
}
