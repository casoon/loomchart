//! Drawing Data Structures
//! Phase 4: Task 7.1

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

fn get_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// Drawing type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawingType {
    TrendLine,
    HorizontalLine,
    VerticalLine,
    Rectangle,
    FibonacciRetracement,
}

/// Point in chart space (logical coordinates)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    /// Candle timestamp (logical X)
    pub timestamp: i64,
    /// Price value (logical Y)
    pub price: f64,
}

impl Point {
    pub fn new(timestamp: i64, price: f64) -> Self {
        Self { timestamp, price }
    }
}

/// Line style for drawings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
}

/// Drawing style configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingStyle {
    /// Line color (RGBA)
    pub color: (u8, u8, u8, u8),
    /// Line width in pixels
    pub width: f64,
    /// Line style
    pub line_style: LineStyle,
    /// Extend line to the left
    pub extend_left: bool,
    /// Extend line to the right
    pub extend_right: bool,
    /// Fill color for shapes (optional)
    pub fill_color: Option<(u8, u8, u8, u8)>,
}

impl Default for DrawingStyle {
    fn default() -> Self {
        Self {
            color: (33, 150, 243, 255), // Blue
            width: 2.0,
            line_style: LineStyle::Solid,
            extend_left: false,
            extend_right: false,
            fill_color: None,
        }
    }
}

/// Complete drawing object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drawing {
    /// Unique identifier
    pub id: String,
    /// Drawing type
    pub drawing_type: DrawingType,
    /// Control points
    pub points: Vec<Point>,
    /// Visual style
    pub style: DrawingStyle,
    /// Whether the drawing is locked (cannot be modified)
    pub locked: bool,
    /// Whether the drawing is visible
    pub visible: bool,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
}

impl Drawing {
    /// Create a new drawing
    pub fn new(drawing_type: DrawingType, points: Vec<Point>) -> Self {
        let now = get_timestamp();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            drawing_type,
            points,
            style: DrawingStyle::default(),
            locked: false,
            visible: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update drawing points
    pub fn update_points(&mut self, points: Vec<Point>) {
        self.points = points;
        self.updated_at = get_timestamp();
    }

    /// Update drawing style
    pub fn update_style(&mut self, style: DrawingStyle) {
        self.style = style;
        self.updated_at = get_timestamp();
    }

    /// Check if drawing has minimum required points
    pub fn is_valid(&self) -> bool {
        let min_points = match self.drawing_type {
            DrawingType::TrendLine => 2,
            DrawingType::HorizontalLine => 1,
            DrawingType::VerticalLine => 1,
            DrawingType::Rectangle => 2,
            DrawingType::FibonacciRetracement => 2,
        };

        self.points.len() >= min_points
    }
}
