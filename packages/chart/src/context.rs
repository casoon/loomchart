//! Drawing context for plugins.
//!
//! Provides the interface for plugins to draw on the chart.

use loom_core::{Price, Timestamp};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec, collections::BTreeMap};

#[cfg(feature = "std")]
use std::collections::HashMap;

use crate::primitives::*;

/// Layer identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct LayerId(pub u32);

impl LayerId {
    pub const BACKGROUND: LayerId = LayerId(0);
    pub const ZONES: LayerId = LayerId(100);
    pub const DRAWINGS: LayerId = LayerId(200);
    pub const INDICATORS: LayerId = LayerId(300);
    pub const LABELS: LayerId = LayerId(400);
    pub const OVERLAY: LayerId = LayerId(500);
    pub const TOP: LayerId = LayerId(1000);
}

/// Drawing with metadata
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DrawingEntry {
    pub id: DrawingId,
    pub drawing: Drawing,
    pub layer: LayerId,
    pub plugin_name: String,
    pub interactive: bool,
    pub visible: bool,
}

/// Buffer for collecting drawings
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DrawingBuffer {
    pub drawings: Vec<DrawingEntry>,
    next_id: u64,
}

impl DrawingBuffer {
    pub fn new() -> Self {
        Self {
            drawings: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a drawing
    pub fn add(&mut self, drawing: Drawing, layer: LayerId, plugin: &str) -> DrawingId {
        let id = DrawingId::new(self.next_id);
        self.next_id += 1;

        self.drawings.push(DrawingEntry {
            id,
            drawing,
            layer,
            plugin_name: String::from(plugin),
            interactive: false,
            visible: true,
        });

        id
    }

    /// Remove a drawing by ID
    pub fn remove(&mut self, id: DrawingId) -> bool {
        if let Some(pos) = self.drawings.iter().position(|d| d.id == id) {
            self.drawings.remove(pos);
            true
        } else {
            false
        }
    }

    /// Remove all drawings from a plugin
    pub fn remove_plugin_drawings(&mut self, plugin_name: &str) {
        self.drawings.retain(|d| d.plugin_name != plugin_name);
    }

    /// Get drawings sorted by layer
    pub fn sorted_by_layer(&self) -> Vec<&DrawingEntry> {
        let mut sorted: Vec<_> = self.drawings.iter().filter(|d| d.visible).collect();
        sorted.sort_by_key(|d| d.layer);
        sorted
    }

    /// Clear all drawings
    pub fn clear(&mut self) {
        self.drawings.clear();
    }
}

/// Drawing context provided to plugins
pub struct DrawingContext<'a> {
    /// Drawing buffer
    buffer: &'a mut DrawingBuffer,
    /// Current plugin name
    plugin_name: String,
    /// Current layer
    current_layer: LayerId,
    /// Visible time range
    pub visible_start: Timestamp,
    pub visible_end: Timestamp,
    /// Visible price range
    pub visible_low: Price,
    pub visible_high: Price,
    /// Chart dimensions in pixels
    pub chart_width: f32,
    pub chart_height: f32,
}

impl<'a> DrawingContext<'a> {
    pub fn new(
        buffer: &'a mut DrawingBuffer,
        plugin_name: impl Into<String>,
    ) -> Self {
        Self {
            buffer,
            plugin_name: plugin_name.into(),
            current_layer: LayerId::DRAWINGS,
            visible_start: 0,
            visible_end: i64::MAX,
            visible_low: 0.0,
            visible_high: f64::MAX,
            chart_width: 800.0,
            chart_height: 600.0,
        }
    }

    /// Set current layer for new drawings
    pub fn layer(&mut self, layer: LayerId) -> &mut Self {
        self.current_layer = layer;
        self
    }

    /// Draw any drawing type
    pub fn draw<D: IntoDrawing>(&mut self, drawing: D) -> DrawingId {
        self.buffer.add(drawing.into_drawing(), self.current_layer, &self.plugin_name)
    }

    /// Draw with explicit layer
    pub fn draw_on_layer<D: IntoDrawing>(&mut self, drawing: D, layer: LayerId) -> DrawingId {
        self.buffer.add(drawing.into_drawing(), layer, &self.plugin_name)
    }

    /// Remove a drawing
    pub fn remove(&mut self, id: DrawingId) {
        self.buffer.remove(id);
    }

    /// Remove all drawings from current plugin
    pub fn clear(&mut self) {
        self.buffer.remove_plugin_drawings(&self.plugin_name);
    }

    // ========================================================================
    // CONVENIENCE METHODS
    // ========================================================================

    /// Draw a horizontal line at price level
    pub fn horizontal_line(&mut self, price: Price) -> DrawingId {
        self.draw(HorizontalLine::new(price))
    }

    /// Draw a horizontal line with options
    pub fn horizontal_line_styled(
        &mut self,
        price: Price,
        color: Color,
        style: LineStyle,
    ) -> DrawingId {
        self.draw(HorizontalLine::new(price).color(color).style(style))
    }

    /// Draw a vertical line at time
    pub fn vertical_line(&mut self, time: Timestamp) -> DrawingId {
        self.draw(VerticalLine::new(time).into_drawing())
    }

    /// Draw a trendline between two points
    pub fn trendline(&mut self, start: Point, end: Point) -> DrawingId {
        self.draw(TrendLine::new(start, end))
    }

    /// Draw a label at a point
    pub fn label(&mut self, text: impl Into<String>, time: Timestamp, price: Price) -> DrawingId {
        self.draw(Label::new(text, time, price))
    }

    /// Draw a price zone
    pub fn price_zone(&mut self, top: Price, bottom: Price) -> DrawingId {
        self.draw(PriceZone::new(top, bottom))
    }

    /// Draw a price zone with label
    pub fn price_zone_labeled(
        &mut self,
        top: Price,
        bottom: Price,
        label: impl Into<String>,
        color: Color,
    ) -> DrawingId {
        self.draw(PriceZone::new(top, bottom).fill(color).label(label))
    }

    /// Draw an arrow up (buy signal)
    pub fn arrow_up(&mut self, time: Timestamp, price: Price) -> DrawingId {
        self.draw(Icon::arrow_up(time, price))
    }

    /// Draw an arrow down (sell signal)
    pub fn arrow_down(&mut self, time: Timestamp, price: Price) -> DrawingId {
        self.draw(Icon::arrow_down(time, price))
    }

    /// Draw a Fibonacci retracement
    pub fn fibonacci(&mut self, start: Point, end: Point) -> DrawingId {
        self.draw(FibonacciRetracement::new(start, end))
    }

    /// Draw a rectangle
    pub fn rectangle(&mut self, p1: Point, p2: Point) -> DrawingId {
        self.draw(Rectangle::new(p1, p2))
    }

    /// Draw a triangle
    pub fn triangle(&mut self, p1: Point, p2: Point, p3: Point) -> DrawingId {
        self.draw(Triangle::new(p1, p2, p3))
    }

    /// Draw a channel
    pub fn channel(&mut self, start: Point, end: Point, width: Price) -> DrawingId {
        self.draw(Channel::from_trendline(start, end, width))
    }

    /// Draw an icon/marker
    pub fn icon(&mut self, position: Point, icon_type: IconType, color: Color) -> DrawingId {
        self.draw(Icon::new(position, icon_type).color(color))
    }

    // ========================================================================
    // UTILITY METHODS
    // ========================================================================

    /// Check if a point is in visible range
    pub fn is_visible(&self, time: Timestamp, price: Price) -> bool {
        time >= self.visible_start
            && time <= self.visible_end
            && price >= self.visible_low
            && price <= self.visible_high
    }

    /// Check if a time is in visible range
    pub fn is_time_visible(&self, time: Timestamp) -> bool {
        time >= self.visible_start && time <= self.visible_end
    }

    /// Check if a price is in visible range
    pub fn is_price_visible(&self, price: Price) -> bool {
        price >= self.visible_low && price <= self.visible_high
    }

    /// Convert chart coordinates to pixel coordinates
    pub fn to_pixels(&self, time: Timestamp, price: Price) -> (f32, f32) {
        let x = (time - self.visible_start) as f32 / (self.visible_end - self.visible_start) as f32 * self.chart_width;
        let y = (1.0 - (price - self.visible_low) / (self.visible_high - self.visible_low)) as f32 * self.chart_height;
        (x, y)
    }
}

/// Builder for creating drawing contexts in tests
pub struct DrawingContextBuilder {
    visible_start: Timestamp,
    visible_end: Timestamp,
    visible_low: Price,
    visible_high: Price,
    chart_width: f32,
    chart_height: f32,
}

impl DrawingContextBuilder {
    pub fn new() -> Self {
        Self {
            visible_start: 0,
            visible_end: 1000000,
            visible_low: 0.0,
            visible_high: 100.0,
            chart_width: 800.0,
            chart_height: 600.0,
        }
    }

    pub fn time_range(mut self, start: Timestamp, end: Timestamp) -> Self {
        self.visible_start = start;
        self.visible_end = end;
        self
    }

    pub fn price_range(mut self, low: Price, high: Price) -> Self {
        self.visible_low = low;
        self.visible_high = high;
        self
    }

    pub fn dimensions(mut self, width: f32, height: f32) -> Self {
        self.chart_width = width;
        self.chart_height = height;
        self
    }

    pub fn build<'a>(self, buffer: &'a mut DrawingBuffer, plugin: &str) -> DrawingContext<'a> {
        let mut ctx = DrawingContext::new(buffer, plugin);
        ctx.visible_start = self.visible_start;
        ctx.visible_end = self.visible_end;
        ctx.visible_low = self.visible_low;
        ctx.visible_high = self.visible_high;
        ctx.chart_width = self.chart_width;
        ctx.chart_height = self.chart_height;
        ctx
    }
}

impl Default for DrawingContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}
