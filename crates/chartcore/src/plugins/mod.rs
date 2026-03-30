// Indicator Plugin System

pub mod builtin;
pub mod loader;
pub mod registry;
pub mod types;

pub use loader::WasmPluginLoader;
pub use registry::PluginRegistry;

use crate::core::Candle;
use std::collections::HashMap;

/// Indicator category for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorCategory {
    MovingAverages,
    Momentum,
    Oscillators,
    Trend,
    Volatility,
    Volume,
    ChannelsAndBands,
    Custom,
    Other,
}

impl IndicatorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MovingAverages => "Moving Averages",
            Self::Momentum => "Momentum",
            Self::Oscillators => "Oscillators",
            Self::Trend => "Trend",
            Self::Volatility => "Volatility",
            Self::Volume => "Volume",
            Self::ChannelsAndBands => "Channels & Bands",
            Self::Custom => "Custom",
            Self::Other => "Other",
        }
    }
}

/// Input type for indicator configuration
#[derive(Debug, Clone, PartialEq)]
pub enum InputType {
    Int {
        min: Option<i32>,
        max: Option<i32>,
    },
    Float {
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
    },
    Bool,
    String {
        options: Option<Vec<String>>,
    },
    Source,
    Color,
}

/// Input configuration for UI generation
#[derive(Debug, Clone)]
pub struct InputConfig {
    pub id: String,
    pub title: String,
    pub input_type: InputType,
    pub default: InputValue,
    pub tooltip: Option<String>,
}

impl InputConfig {
    pub fn int(id: &str, title: &str, default: i32) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            input_type: InputType::Int {
                min: None,
                max: None,
            },
            default: InputValue::Int(default),
            tooltip: None,
        }
    }

    pub fn float(id: &str, title: &str, default: f64) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            input_type: InputType::Float {
                min: None,
                max: None,
                step: None,
            },
            default: InputValue::Float(default),
            tooltip: None,
        }
    }

    pub fn source(id: &str, title: &str, default: SourceType) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            input_type: InputType::Source,
            default: InputValue::Source(default),
            tooltip: None,
        }
    }

    pub fn min<T: Into<f64>>(mut self, min: T) -> Self {
        let min_val = min.into();
        match &mut self.input_type {
            InputType::Int { min: ref mut m, .. } => *m = Some(min_val as i32),
            InputType::Float { min: ref mut m, .. } => *m = Some(min_val),
            _ => {}
        }
        self
    }

    pub fn max<T: Into<f64>>(mut self, max: T) -> Self {
        let max_val = max.into();
        match &mut self.input_type {
            InputType::Int { max: ref mut m, .. } => *m = Some(max_val as i32),
            InputType::Float { max: ref mut m, .. } => *m = Some(max_val),
            _ => {}
        }
        self
    }

    pub fn tooltip(mut self, tooltip: &str) -> Self {
        self.tooltip = Some(tooltip.to_string());
        self
    }
}

/// Input value (runtime)
#[derive(Debug, Clone, PartialEq)]
pub enum InputValue {
    Int(i32),
    Float(f64),
    Bool(bool),
    String(String),
    Source(SourceType),
    Color(String),
}

/// Source type for price data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceType {
    Open,
    High,
    Low,
    Close,
    HL2,   // (high + low) / 2
    HLC3,  // (high + low + close) / 3
    OHLC4, // (open + high + low + close) / 4
    HLCC4, // (high + low + close + close) / 4
    Volume,
}

impl SourceType {
    pub fn extract(&self, candle: &Candle) -> f64 {
        match self {
            Self::Open => candle.o,
            Self::High => candle.h,
            Self::Low => candle.l,
            Self::Close => candle.c,
            Self::HL2 => (candle.h + candle.l) / 2.0,
            Self::HLC3 => (candle.h + candle.l + candle.c) / 3.0,
            Self::OHLC4 => (candle.o + candle.h + candle.l + candle.c) / 4.0,
            Self::HLCC4 => (candle.h + candle.l + candle.c + candle.c) / 4.0,
            Self::Volume => candle.v,
        }
    }
}

/// Plot style types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotStyle {
    Line,
    Histogram,
    Area,
    Circles,
}

/// Plot configuration for visualization
#[derive(Debug, Clone)]
pub struct PlotConfig {
    pub id: String,
    pub title: String,
    pub color: String,
    pub line_width: u8,
    pub line_style: LineStyle,
    pub plot_style: PlotStyle,
}

impl PlotConfig {
    pub fn new(id: &str, title: &str, color: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            color: color.to_string(),
            line_width: 2,
            line_style: LineStyle::Solid,
            plot_style: PlotStyle::Line,
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

    pub fn style(mut self, style: &str) -> Self {
        self.line_style = match style {
            "dashed" => LineStyle::Dashed,
            "dotted" => LineStyle::Dotted,
            _ => LineStyle::Solid,
        };
        self
    }

    pub fn histogram(mut self) -> Self {
        self.plot_style = PlotStyle::Histogram;
        self
    }

    pub fn area(mut self) -> Self {
        self.plot_style = PlotStyle::Area;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
}

/// Calculation context (inputs for indicator calculation)
#[derive(Debug)]
pub struct CalculationContext<'a> {
    pub candles: &'a [Candle],
    pub inputs: HashMap<String, InputValue>,
}

impl<'a> CalculationContext<'a> {
    pub fn new(candles: &'a [Candle]) -> Self {
        Self {
            candles,
            inputs: HashMap::new(),
        }
    }

    pub fn with_input(mut self, key: &str, value: InputValue) -> Self {
        self.inputs.insert(key.to_string(), value);
        self
    }

    /// Get source series from candles
    pub fn source(&self, source_type: SourceType) -> Vec<f64> {
        self.candles
            .iter()
            .map(|c| source_type.extract(c))
            .collect()
    }

    /// Get open prices
    pub fn open(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.o).collect()
    }

    /// Get high prices
    pub fn high(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.h).collect()
    }

    /// Get low prices
    pub fn low(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.l).collect()
    }

    /// Get close prices
    pub fn close(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.c).collect()
    }

    /// Get volume
    pub fn volume(&self) -> Vec<f64> {
        self.candles.iter().map(|c| c.v).collect()
    }

    /// Get input as integer
    pub fn input_int(&self, key: &str) -> Option<i32> {
        match self.inputs.get(key) {
            Some(InputValue::Int(v)) => Some(*v),
            _ => None,
        }
    }

    /// Get input as float
    pub fn input_float(&self, key: &str) -> Option<f64> {
        match self.inputs.get(key) {
            Some(InputValue::Float(v)) => Some(*v),
            _ => None,
        }
    }

    /// Get input as source type
    pub fn input_source(&self, key: &str) -> Option<SourceType> {
        match self.inputs.get(key) {
            Some(InputValue::Source(v)) => Some(*v),
            _ => None,
        }
    }
}

/// Indicator calculation result
#[derive(Debug, Clone)]
pub struct IndicatorResult {
    pub plots: HashMap<String, Vec<Option<f64>>>,
    pub metadata: IndicatorMetadata,
}

impl IndicatorResult {
    pub fn new(title: &str, short_title: &str, overlay: bool) -> Self {
        Self {
            plots: HashMap::new(),
            metadata: IndicatorMetadata {
                title: title.to_string(),
                short_title: short_title.to_string(),
                overlay,
            },
        }
    }

    pub fn add_plot(mut self, id: &str, values: Vec<Option<f64>>) -> Self {
        self.plots.insert(id.to_string(), values);
        self
    }
}

#[derive(Debug, Clone)]
pub struct IndicatorMetadata {
    pub title: String,
    pub short_title: String,
    pub overlay: bool,
}

/// Main plugin trait (implemented by both built-in and WASM plugins)
pub trait IndicatorPlugin: Send + Sync {
    /// Unique identifier
    fn id(&self) -> &str;

    /// Human-readable name
    fn name(&self) -> &str;

    /// Category for organization
    fn category(&self) -> IndicatorCategory;

    /// Short description
    fn description(&self) -> &str {
        ""
    }

    /// Whether this indicator overlays on the price chart
    fn overlay(&self) -> bool {
        false
    }

    /// Input configuration for UI
    fn inputs(&self) -> Vec<InputConfig>;

    /// Plot configuration for visualization
    fn plots(&self) -> Vec<PlotConfig>;

    /// Calculate indicator values
    fn calculate(&self, context: &CalculationContext) -> IndicatorResult;

    /// Optional: Incremental calculation for real-time updates
    fn calculate_incremental(
        &self,
        _context: &CalculationContext,
        _new_candle: &Candle,
        _previous_result: &IndicatorResult,
    ) -> Option<IndicatorResult> {
        None // Default: not supported
    }

    /// Optional: Validate inputs
    fn validate_inputs(&self, _inputs: &HashMap<String, InputValue>) -> Result<(), Vec<String>> {
        Ok(())
    }

    /// Optional: Get dependencies (other indicator IDs)
    fn dependencies(&self, _inputs: &HashMap<String, InputValue>) -> Vec<String> {
        Vec::new()
    }
}
