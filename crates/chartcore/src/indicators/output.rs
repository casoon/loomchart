/// Standard output interface for technical indicators
///
/// All indicators must implement the Indicator trait and return
/// IndicatorOutput variants for rendering.
use crate::core::Candle;
use crate::primitives::Color;
use serde::{Deserialize, Serialize};

/// Standard output format for all indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum IndicatorOutput {
    /// Single line (e.g., SMA, EMA, RSI)
    SingleLine {
        values: Vec<Option<f64>>, // None for NaN/missing
        color: Color,
        width: f64,
        style: LineStyle,
    },

    /// Multiple lines (e.g., Bollinger Bands, Ichimoku)
    MultiLine { lines: Vec<LineData> },

    /// Histogram (e.g., MACD histogram, Volume)
    Histogram {
        values: Vec<f64>,
        positive_color: Color,
        negative_color: Color,
        zero_line: bool,
    },

    /// Cloud/Area between two lines (e.g., Ichimoku cloud)
    CloudArea {
        upper: Vec<f64>,
        lower: Vec<f64>,
        bullish_color: Color,
        bearish_color: Color,
        alpha: f64, // Opacity 0.0-1.0
    },

    /// Scatter plot (e.g., Pivot Points, Support/Resistance)
    Scatter {
        points: Vec<ScatterPoint>,
        color: Color,
        size: f64,
        shape: MarkerShape,
    },

    /// Bands (e.g., Bollinger Bands, Keltner Channel)
    Bands {
        middle: Vec<Option<f64>>,
        upper: Vec<Option<f64>>,
        lower: Vec<Option<f64>>,
        middle_color: Color,
        band_color: Color,
        fill_alpha: f64, // Fill opacity
    },
}

/// Line data for multi-line indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineData {
    pub values: Vec<Option<f64>>,
    pub color: Color,
    pub width: f64,
    pub style: LineStyle,
    pub label: String,
}

/// Line style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
}

/// Marker shape for scatter plots
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkerShape {
    Circle,
    Square,
    Triangle,
    Cross,
    Diamond,
}

/// Scatter plot point
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScatterPoint {
    pub index: usize, // Candle index
    pub value: f64,
}

/// Standard interface all indicators must implement
pub trait Indicator: Send + Sync {
    /// Calculate indicator values from candle data
    fn calculate(&self, candles: &[Candle]) -> IndicatorOutput;

    /// Get the scale range for this indicator (None = use price scale)
    /// Returns (min, max) for the indicator's value range
    fn get_scale_range(&self, candles: &[Candle]) -> Option<(f64, f64)>;

    /// Can this indicator be overlaid on the price chart?
    fn supports_overlay(&self) -> bool;

    /// Indicator name for display
    fn name(&self) -> &str;

    /// Short identifier (e.g., "rsi", "sma_20")
    fn id(&self) -> String;

    /// Indicator parameters for serialization
    fn get_params(&self) -> serde_json::Value;

    /// Update parameters (returns error if invalid)
    fn set_params(&mut self, params: serde_json::Value) -> Result<(), String>;

    /// Number of historical candles required for calculation
    fn required_candles(&self) -> usize {
        0
    }
}

impl LineStyle {
    /// Convert to dash pattern for canvas rendering
    pub fn to_dash_pattern(&self) -> Option<Vec<f64>> {
        match self {
            LineStyle::Solid => None,
            LineStyle::Dashed => Some(vec![5.0, 5.0]),
            LineStyle::Dotted => Some(vec![2.0, 3.0]),
        }
    }
}

impl LineData {
    /// Create new line data
    pub fn new(label: impl Into<String>, values: Vec<Option<f64>>, color: Color) -> Self {
        Self {
            label: label.into(),
            values,
            color,
            width: 1.5,
            style: LineStyle::Solid,
        }
    }

    /// Set line width
    pub fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Set line style
    pub fn with_style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }
}

impl ScatterPoint {
    /// Create new scatter point
    pub fn new(index: usize, value: f64) -> Self {
        Self { index, value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_style_dash_pattern() {
        assert_eq!(LineStyle::Solid.to_dash_pattern(), None);
        assert_eq!(LineStyle::Dashed.to_dash_pattern(), Some(vec![5.0, 5.0]));
        assert_eq!(LineStyle::Dotted.to_dash_pattern(), Some(vec![2.0, 3.0]));
    }

    #[test]
    fn test_line_data_builder() {
        let values = vec![Some(100.0), Some(101.0), None, Some(102.0)];
        let color = Color::rgb(255, 0, 0);

        let line = LineData::new("Test", values.clone(), color.clone())
            .with_width(2.0)
            .with_style(LineStyle::Dashed);

        assert_eq!(line.label, "Test");
        assert_eq!(line.values, values);
        assert_eq!(line.width, 2.0);
        assert_eq!(line.style, LineStyle::Dashed);
    }

    #[test]
    fn test_scatter_point() {
        let point = ScatterPoint::new(10, 150.5);
        assert_eq!(point.index, 10);
        assert_eq!(point.value, 150.5);
    }
}
