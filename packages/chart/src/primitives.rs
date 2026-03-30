//! Drawing primitives for chart visualization.
//!
//! All the basic shapes, lines, labels, and annotations.

use loom_core::{Price, Timestamp};

#[cfg(not(feature = "std"))]
use alloc::{string::String, vec::Vec, boxed::Box};

/// Unique drawing identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DrawingId(pub u64);

impl DrawingId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// RGBA Color
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn with_alpha(mut self, alpha: f32) -> Self {
        self.a = alpha.clamp(0.0, 1.0);
        self
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    pub fn to_rgba_string(&self) -> String {
        format!("rgba({},{},{},{})", self.r, self.g, self.b, self.a)
    }

    // Predefined colors
    pub const WHITE: Color = Color::rgb(255, 255, 255);
    pub const BLACK: Color = Color::rgb(0, 0, 0);
    pub const RED: Color = Color::rgb(239, 83, 80);
    pub const GREEN: Color = Color::rgb(38, 166, 154);
    pub const BLUE: Color = Color::rgb(66, 165, 245);
    pub const YELLOW: Color = Color::rgb(255, 235, 59);
    pub const ORANGE: Color = Color::rgb(255, 152, 0);
    pub const PURPLE: Color = Color::rgb(171, 71, 188);
    pub const CYAN: Color = Color::rgb(0, 188, 212);
    pub const GRAY: Color = Color::rgb(158, 158, 158);

    // Trading specific
    pub const BULLISH: Color = Color::rgb(38, 166, 154);
    pub const BEARISH: Color = Color::rgb(239, 83, 80);
    pub const NEUTRAL: Color = Color::rgb(158, 158, 158);
    pub const SUPPORT: Color = Color::rgb(76, 175, 80);
    pub const RESISTANCE: Color = Color::rgb(244, 67, 54);
    pub const FIB_382: Color = Color::rgb(255, 235, 59);
    pub const FIB_500: Color = Color::rgb(255, 152, 0);
    pub const FIB_618: Color = Color::rgb(244, 67, 54);
}

/// Point on chart (time, price)
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Point {
    pub time: Timestamp,
    pub price: Price,
}

impl Point {
    pub const fn new(time: Timestamp, price: Price) -> Self {
        Self { time, price }
    }
}

/// Text anchor position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Anchor {
    TopLeft,
    TopCenter,
    TopRight,
    #[default]
    MiddleLeft,
    MiddleCenter,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

/// Line style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
    DashDot,
}

impl LineStyle {
    pub fn to_dash_array(&self) -> Option<&'static [f32]> {
        match self {
            Self::Solid => None,
            Self::Dashed => Some(&[8.0, 4.0]),
            Self::Dotted => Some(&[2.0, 2.0]),
            Self::DashDot => Some(&[8.0, 4.0, 2.0, 4.0]),
        }
    }
}

/// Fill pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FillPattern {
    #[default]
    Solid,
    Hatch,
    CrossHatch,
    Dots,
    None,
}

// ============================================================================
// LINE TYPES
// ============================================================================

/// Generic line properties
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LineProps {
    pub color: Color,
    pub width: f32,
    pub style: LineStyle,
}

impl Default for LineProps {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            width: 1.0,
            style: LineStyle::Solid,
        }
    }
}

/// Simple line between two points
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Line {
    pub start: Point,
    pub end: Point,
    pub props: LineProps,
}

impl Line {
    pub fn new(start: Point, end: Point) -> Self {
        Self {
            start,
            end,
            props: LineProps::default(),
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.props.color = color;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.props.width = width;
        self
    }

    pub fn style(mut self, style: LineStyle) -> Self {
        self.props.style = style;
        self
    }
}

/// Horizontal line at a price level
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HorizontalLine {
    pub price: Price,
    pub start_time: Option<Timestamp>,
    pub end_time: Option<Timestamp>,
    pub props: LineProps,
    pub label: Option<String>,
    pub show_price: bool,
}

impl HorizontalLine {
    pub fn new(price: Price) -> Self {
        Self {
            price,
            start_time: None,
            end_time: None,
            props: LineProps::default(),
            label: None,
            show_price: true,
        }
    }

    pub fn from_to(price: Price, start: Timestamp, end: Timestamp) -> Self {
        Self {
            price,
            start_time: Some(start),
            end_time: Some(end),
            props: LineProps::default(),
            label: None,
            show_price: true,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.props.color = color;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.props.width = width;
        self
    }

    pub fn style(mut self, style: LineStyle) -> Self {
        self.props.style = style;
        self
    }

    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }
}

/// Vertical line at a time
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VerticalLine {
    pub time: Timestamp,
    pub start_price: Option<Price>,
    pub end_price: Option<Price>,
    pub props: LineProps,
    pub label: Option<String>,
}

impl VerticalLine {
    pub fn new(time: Timestamp) -> Self {
        Self {
            time,
            start_price: None,
            end_price: None,
            props: LineProps::default(),
            label: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.props.color = color;
        self
    }

    pub fn style(mut self, style: LineStyle) -> Self {
        self.props.style = style;
        self
    }

    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }
}

/// Trend line (extends to edges)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrendLine {
    pub start: Point,
    pub end: Point,
    pub extend_left: bool,
    pub extend_right: bool,
    pub props: LineProps,
}

impl TrendLine {
    pub fn new(start: Point, end: Point) -> Self {
        Self {
            start,
            end,
            extend_left: false,
            extend_right: true,
            props: LineProps::default(),
        }
    }

    pub fn extend_both(mut self) -> Self {
        self.extend_left = true;
        self.extend_right = true;
        self
    }

    pub fn extend_right_only(mut self) -> Self {
        self.extend_left = false;
        self.extend_right = true;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.props.color = color;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.props.width = width;
        self
    }
}

/// Ray (half-infinite line)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ray {
    pub origin: Point,
    pub direction: Point,
    pub props: LineProps,
}

impl Ray {
    pub fn new(origin: Point, through: Point) -> Self {
        Self {
            origin,
            direction: through,
            props: LineProps::default(),
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.props.color = color;
        self
    }
}

/// Extended line (infinite in both directions)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExtendedLine {
    pub point1: Point,
    pub point2: Point,
    pub props: LineProps,
}

// ============================================================================
// LABELS
// ============================================================================

/// Text styling
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextStyle {
    pub font_size: f32,
    pub font_family: String,
    pub color: Color,
    pub background: Option<Color>,
    pub border_color: Option<Color>,
    pub padding: f32,
    pub anchor: Anchor,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 12.0,
            font_family: String::from("Arial"),
            color: Color::WHITE,
            background: None,
            border_color: None,
            padding: 4.0,
            anchor: Anchor::MiddleLeft,
        }
    }
}

/// Text label at a point
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Label {
    pub text: String,
    pub position: Point,
    pub style: TextStyle,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Label {
    pub fn new(text: impl Into<String>, time: Timestamp, price: Price) -> Self {
        Self {
            text: text.into(),
            position: Point::new(time, price),
            style: TextStyle::default(),
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    pub fn at_point(text: impl Into<String>, point: Point) -> Self {
        Self {
            text: text.into(),
            position: point,
            style: TextStyle::default(),
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.style.color = color;
        self
    }

    pub fn background(mut self, color: Color) -> Self {
        self.style.background = Some(color);
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.style.font_size = size;
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.style.anchor = anchor;
        self
    }

    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }
}

/// Price label (on Y-axis)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PriceLabel {
    pub price: Price,
    pub text: Option<String>,
    pub color: Color,
    pub background: Color,
}

impl PriceLabel {
    pub fn new(price: Price) -> Self {
        Self {
            price,
            text: None,
            color: Color::WHITE,
            background: Color::GRAY,
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    pub fn colors(mut self, text: Color, bg: Color) -> Self {
        self.color = text;
        self.background = bg;
        self
    }
}

/// Time label (on X-axis)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeLabel {
    pub time: Timestamp,
    pub text: Option<String>,
    pub color: Color,
    pub background: Color,
}

// ============================================================================
// SHAPES
// ============================================================================

/// Generic shape properties
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShapeProps {
    pub stroke_color: Color,
    pub stroke_width: f32,
    pub fill_color: Option<Color>,
    pub fill_pattern: FillPattern,
}

impl Default for ShapeProps {
    fn default() -> Self {
        Self {
            stroke_color: Color::WHITE,
            stroke_width: 1.0,
            fill_color: None,
            fill_pattern: FillPattern::Solid,
        }
    }
}

/// Rectangle shape
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rectangle {
    pub top_left: Point,
    pub bottom_right: Point,
    pub props: ShapeProps,
}

impl Rectangle {
    pub fn new(p1: Point, p2: Point) -> Self {
        Self {
            top_left: p1,
            bottom_right: p2,
            props: ShapeProps::default(),
        }
    }

    pub fn from_coords(time1: Timestamp, price1: Price, time2: Timestamp, price2: Price) -> Self {
        Self::new(Point::new(time1, price1), Point::new(time2, price2))
    }

    pub fn stroke(mut self, color: Color) -> Self {
        self.props.stroke_color = color;
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.props.fill_color = Some(color);
        self
    }

    pub fn stroke_width(mut self, width: f32) -> Self {
        self.props.stroke_width = width;
        self
    }
}

/// Triangle shape
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Triangle {
    pub p1: Point,
    pub p2: Point,
    pub p3: Point,
    pub props: ShapeProps,
}

impl Triangle {
    pub fn new(p1: Point, p2: Point, p3: Point) -> Self {
        Self {
            p1,
            p2,
            p3,
            props: ShapeProps::default(),
        }
    }

    pub fn stroke(mut self, color: Color) -> Self {
        self.props.stroke_color = color;
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.props.fill_color = Some(color);
        self
    }
}

/// Circle shape
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Circle {
    pub center: Point,
    pub radius_price: Price,  // Radius in price units
    pub radius_time: Timestamp, // Radius in time units (for ellipse-like appearance)
    pub props: ShapeProps,
}

impl Circle {
    pub fn new(center: Point, radius: Price) -> Self {
        Self {
            center,
            radius_price: radius,
            radius_time: 0,
            props: ShapeProps::default(),
        }
    }

    pub fn stroke(mut self, color: Color) -> Self {
        self.props.stroke_color = color;
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.props.fill_color = Some(color);
        self
    }
}

/// Ellipse shape
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ellipse {
    pub center: Point,
    pub radius_x: Timestamp,
    pub radius_y: Price,
    pub rotation: f32, // Degrees
    pub props: ShapeProps,
}

/// Arc shape
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Arc {
    pub center: Point,
    pub radius: Price,
    pub start_angle: f32, // Degrees
    pub end_angle: f32,
    pub props: ShapeProps,
}

// ============================================================================
// ARROWS & MARKERS
// ============================================================================

/// Arrow types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ArrowHead {
    #[default]
    None,
    Triangle,
    Open,
    Diamond,
    Circle,
}

/// Arrow
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Arrow {
    pub start: Point,
    pub end: Point,
    pub start_head: ArrowHead,
    pub end_head: ArrowHead,
    pub props: LineProps,
}

impl Arrow {
    pub fn new(start: Point, end: Point) -> Self {
        Self {
            start,
            end,
            start_head: ArrowHead::None,
            end_head: ArrowHead::Triangle,
            props: LineProps::default(),
        }
    }

    pub fn double_headed(mut self) -> Self {
        self.start_head = ArrowHead::Triangle;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.props.color = color;
        self
    }
}

/// Bracket (for highlighting ranges)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bracket {
    pub start: Point,
    pub end: Point,
    pub side: BracketSide,
    pub props: LineProps,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BracketSide {
    Left,
    Right,
    Top,
    Bottom,
}

// ============================================================================
// ZONES
// ============================================================================

/// Price zone (horizontal band)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PriceZone {
    pub top: Price,
    pub bottom: Price,
    pub start_time: Option<Timestamp>,
    pub end_time: Option<Timestamp>,
    pub fill_color: Color,
    pub border_color: Option<Color>,
    pub label: Option<String>,
}

impl PriceZone {
    pub fn new(top: Price, bottom: Price) -> Self {
        Self {
            top,
            bottom,
            start_time: None,
            end_time: None,
            fill_color: Color::GRAY.with_alpha(0.2),
            border_color: None,
            label: None,
        }
    }

    pub fn from_to(top: Price, bottom: Price, start: Timestamp, end: Timestamp) -> Self {
        Self {
            top,
            bottom,
            start_time: Some(start),
            end_time: Some(end),
            fill_color: Color::GRAY.with_alpha(0.2),
            border_color: None,
            label: None,
        }
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill_color = color;
        self
    }

    pub fn border(mut self, color: Color) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }
}

/// Time zone (vertical band)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TimeZone {
    pub start: Timestamp,
    pub end: Timestamp,
    pub fill_color: Color,
    pub border_color: Option<Color>,
    pub label: Option<String>,
}

impl TimeZone {
    pub fn new(start: Timestamp, end: Timestamp) -> Self {
        Self {
            start,
            end,
            fill_color: Color::GRAY.with_alpha(0.2),
            border_color: None,
            label: None,
        }
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill_color = color;
        self
    }

    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }
}

/// Generic zone type
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Zone {
    Price(PriceZone),
    Time(TimeZone),
}

// ============================================================================
// FIBONACCI
// ============================================================================

/// Fibonacci retracement
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FibonacciRetracement {
    pub start: Point,
    pub end: Point,
    pub levels: Vec<(f64, Color)>,
    pub show_labels: bool,
    pub extend_lines: bool,
    pub props: LineProps,
}

impl FibonacciRetracement {
    pub fn new(start: Point, end: Point) -> Self {
        Self {
            start,
            end,
            levels: vec![
                (0.0, Color::GRAY),
                (0.236, Color::GRAY),
                (0.382, Color::FIB_382),
                (0.5, Color::FIB_500),
                (0.618, Color::FIB_618),
                (0.786, Color::GRAY),
                (1.0, Color::GRAY),
            ],
            show_labels: true,
            extend_lines: false,
            props: LineProps::default(),
        }
    }

    /// Get price at a Fibonacci level
    pub fn price_at_level(&self, level: f64) -> Price {
        let range = self.end.price - self.start.price;
        self.start.price + range * level
    }
}

/// Fibonacci extension
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FibonacciExtension {
    pub point1: Point,
    pub point2: Point,
    pub point3: Point,
    pub levels: Vec<(f64, Color)>,
    pub show_labels: bool,
    pub props: LineProps,
}

impl FibonacciExtension {
    pub fn new(p1: Point, p2: Point, p3: Point) -> Self {
        Self {
            point1: p1,
            point2: p2,
            point3: p3,
            levels: vec![
                (0.618, Color::GRAY),
                (1.0, Color::FIB_382),
                (1.272, Color::FIB_500),
                (1.618, Color::FIB_618),
                (2.0, Color::GRAY),
                (2.618, Color::GRAY),
            ],
            show_labels: true,
            props: LineProps::default(),
        }
    }
}

/// Fibonacci fan
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FibonacciFan {
    pub start: Point,
    pub end: Point,
    pub levels: Vec<f64>,
    pub props: LineProps,
}

// ============================================================================
// CHANNELS & PATTERNS
// ============================================================================

/// Parallel channel
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Channel {
    pub top_start: Point,
    pub top_end: Point,
    pub bottom_start: Point,
    pub bottom_end: Point,
    pub fill_color: Option<Color>,
    pub props: LineProps,
}

impl Channel {
    /// Create from two points and a width
    pub fn from_trendline(start: Point, end: Point, width: Price) -> Self {
        Self {
            top_start: Point::new(start.time, start.price + width / 2.0),
            top_end: Point::new(end.time, end.price + width / 2.0),
            bottom_start: Point::new(start.time, start.price - width / 2.0),
            bottom_end: Point::new(end.time, end.price - width / 2.0),
            fill_color: None,
            props: LineProps::default(),
        }
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.fill_color = Some(color);
        self
    }
}

/// Andrews' Pitchfork
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PitchFork {
    pub handle: Point,
    pub left_prong: Point,
    pub right_prong: Point,
    pub show_median: bool,
    pub show_schiff: bool,
    pub props: LineProps,
}

// ============================================================================
// ICONS & MARKERS
// ============================================================================

/// Icon types for markers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IconType {
    // Arrows
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    // Shapes
    CircleFilled,
    CircleOutline,
    SquareFilled,
    SquareOutline,
    TriangleUp,
    TriangleDown,
    Diamond,
    Star,
    // Trading
    Flag,
    Cross,
    Check,
    Warning,
    Info,
    // Patterns
    DoubleTop,
    DoubleBottom,
    HeadShoulders,
}

/// Icon marker at a point
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Icon {
    pub position: Point,
    pub icon_type: IconType,
    pub size: f32,
    pub color: Color,
    pub tooltip: Option<String>,
}

impl Icon {
    pub fn new(position: Point, icon_type: IconType) -> Self {
        Self {
            position,
            icon_type,
            size: 16.0,
            color: Color::WHITE,
            tooltip: None,
        }
    }

    pub fn arrow_up(time: Timestamp, price: Price) -> Self {
        Self::new(Point::new(time, price), IconType::ArrowUp).color(Color::GREEN)
    }

    pub fn arrow_down(time: Timestamp, price: Price) -> Self {
        Self::new(Point::new(time, price), IconType::ArrowDown).color(Color::RED)
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }
}

// ============================================================================
// TEXT
// ============================================================================

/// Multi-line text box
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Text {
    pub text: String,
    pub position: Point,
    pub style: TextStyle,
    pub max_width: Option<f32>,
}

// ============================================================================
// PATH (for complex shapes)
// ============================================================================

/// Path command for complex drawings
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    QuadraticCurveTo { control: Point, end: Point },
    BezierCurveTo { control1: Point, control2: Point, end: Point },
    ArcTo { center: Point, radius: Price, start_angle: f32, end_angle: f32 },
    ClosePath,
}

/// Custom path drawing
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Path {
    pub commands: Vec<PathCommand>,
    pub props: ShapeProps,
}

impl Path {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            props: ShapeProps::default(),
        }
    }

    pub fn move_to(mut self, point: Point) -> Self {
        self.commands.push(PathCommand::MoveTo(point));
        self
    }

    pub fn line_to(mut self, point: Point) -> Self {
        self.commands.push(PathCommand::LineTo(point));
        self
    }

    pub fn close(mut self) -> Self {
        self.commands.push(PathCommand::ClosePath);
        self
    }

    pub fn stroke(mut self, color: Color) -> Self {
        self.props.stroke_color = color;
        self
    }

    pub fn fill(mut self, color: Color) -> Self {
        self.props.fill_color = Some(color);
        self
    }
}

impl Default for Path {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// UNIFIED DRAWING ENUM
// ============================================================================

/// All possible drawing types
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Drawing {
    Line(Line),
    HorizontalLine(HorizontalLine),
    VerticalLine(VerticalLine),
    TrendLine(TrendLine),
    Ray(Ray),
    ExtendedLine(ExtendedLine),
    Label(Label),
    PriceLabel(PriceLabel),
    TimeLabel(TimeLabel),
    Rectangle(Rectangle),
    Triangle(Triangle),
    Circle(Circle),
    Ellipse(Ellipse),
    Arc(Arc),
    Arrow(Arrow),
    Bracket(Bracket),
    PriceZone(PriceZone),
    TimeZone(TimeZone),
    FibonacciRetracement(FibonacciRetracement),
    FibonacciExtension(FibonacciExtension),
    FibonacciFan(FibonacciFan),
    Channel(Channel),
    PitchFork(PitchFork),
    Icon(Icon),
    Text(Text),
    Path(Path),
}

/// Trait for converting to Drawing
pub trait IntoDrawing {
    fn into_drawing(self) -> Drawing;
}

impl IntoDrawing for Line {
    fn into_drawing(self) -> Drawing { Drawing::Line(self) }
}

impl IntoDrawing for HorizontalLine {
    fn into_drawing(self) -> Drawing { Drawing::HorizontalLine(self) }
}

impl IntoDrawing for Label {
    fn into_drawing(self) -> Drawing { Drawing::Label(self) }
}

impl IntoDrawing for Rectangle {
    fn into_drawing(self) -> Drawing { Drawing::Rectangle(self) }
}

impl IntoDrawing for Triangle {
    fn into_drawing(self) -> Drawing { Drawing::Triangle(self) }
}

impl IntoDrawing for PriceZone {
    fn into_drawing(self) -> Drawing { Drawing::PriceZone(self) }
}

impl IntoDrawing for Icon {
    fn into_drawing(self) -> Drawing { Drawing::Icon(self) }
}

impl IntoDrawing for FibonacciRetracement {
    fn into_drawing(self) -> Drawing { Drawing::FibonacciRetracement(self) }
}

impl IntoDrawing for Arrow {
    fn into_drawing(self) -> Drawing { Drawing::Arrow(self) }
}

impl IntoDrawing for TrendLine {
    fn into_drawing(self) -> Drawing { Drawing::TrendLine(self) }
}

impl IntoDrawing for Channel {
    fn into_drawing(self) -> Drawing { Drawing::Channel(self) }
}

impl IntoDrawing for Path {
    fn into_drawing(self) -> Drawing { Drawing::Path(self) }
}
