// Basic shapes and line styles

/// Point type (re-export from core for convenience)
pub use crate::core::Point;

/// Candle rendering style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "wasm", derive(serde::Serialize, serde::Deserialize))]
pub enum CandleStyle {
    /// Traditional candlestick with filled body
    Candlestick,
    /// OHLC bars with horizontal ticks
    OHLC,
    /// Hollow candlestick (outline only)
    Hollow,
}

impl Default for CandleStyle {
    fn default() -> Self {
        CandleStyle::Candlestick
    }
}

/// Line style enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
}

impl Default for LineStyle {
    fn default() -> Self {
        LineStyle::Solid
    }
}

/// Plot configuration for visualization
#[derive(Debug, Clone)]
pub struct PlotConfig {
    pub id: String,
    pub title: String,
    pub color: String,
    pub line_width: u8,
    pub line_style: LineStyle,
}

impl PlotConfig {
    pub fn new(id: &str, title: &str, color: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            color: color.to_string(),
            line_width: 2,
            line_style: LineStyle::Solid,
        }
    }

    pub fn line_width(mut self, width: u8) -> Self {
        self.line_width = width;
        self
    }

    pub fn dashed(mut self) -> Self {
        self.line_style = LineStyle::Dashed;
        self
    }

    pub fn dotted(mut self) -> Self {
        self.line_style = LineStyle::Dotted;
        self
    }
}
