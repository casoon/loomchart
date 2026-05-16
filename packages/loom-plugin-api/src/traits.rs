use crate::types::{
    Candle, InputConfig, InputValue, PlotConfig, RenderCommand, Signal,
};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Calculation context (passed to plugins)
// ---------------------------------------------------------------------------

/// Context passed into `IndicatorPlugin::calculate`.
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

    pub fn input_int(&self, key: &str) -> Option<i32> {
        match self.inputs.get(key) {
            Some(InputValue::Int(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn input_float(&self, key: &str) -> Option<f64> {
        match self.inputs.get(key) {
            Some(InputValue::Float(v)) => Some(*v),
            _ => None,
        }
    }

    pub fn input_bool(&self, key: &str) -> Option<bool> {
        match self.inputs.get(key) {
            Some(InputValue::Bool(v)) => Some(*v),
            _ => None,
        }
    }
}

/// Result of an indicator calculation.
pub struct IndicatorResult {
    /// Map of plot-id → series values (None = no data at that candle index)
    pub plots: HashMap<String, Vec<Option<f64>>>,
    pub overlay: bool,
    pub title: String,
}

impl IndicatorResult {
    pub fn new(title: impl Into<String>, overlay: bool) -> Self {
        Self {
            plots: HashMap::new(),
            overlay,
            title: title.into(),
        }
    }

    pub fn add_plot(mut self, id: &str, values: Vec<Option<f64>>) -> Self {
        self.plots.insert(id.to_string(), values);
        self
    }
}

// ---------------------------------------------------------------------------
// Indicator plugin trait
// ---------------------------------------------------------------------------

/// Implement this trait to create a LoomChart indicator plugin.
pub trait IndicatorPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str { "" }
    fn overlay(&self) -> bool { false }
    fn inputs(&self) -> Vec<InputConfig>;
    fn plots(&self) -> Vec<PlotConfig>;
    fn calculate(&self, ctx: &CalculationContext) -> IndicatorResult;
    fn reset(&mut self) {}
}

// ---------------------------------------------------------------------------
// Strategy plugin trait
// ---------------------------------------------------------------------------

/// Context available to a strategy during `update`.
pub struct StrategyContext<'a> {
    pub candles: &'a [Candle],
    /// Indicator output values keyed by "{indicator_id}.{plot_id}"
    pub indicator_values: &'a HashMap<String, Vec<Option<f64>>>,
    pub inputs: HashMap<String, InputValue>,
}

/// Implement this trait to create a LoomChart strategy plugin.
pub trait StrategyPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str { "" }
    fn inputs(&self) -> Vec<InputConfig>;
    fn update(&mut self, candle: &Candle, ctx: &StrategyContext) -> Option<Signal>;
    fn reset(&mut self) {}
}

// ---------------------------------------------------------------------------
// Renderer plugin trait
// ---------------------------------------------------------------------------

/// Viewport information passed to renderer plugins.
pub struct Viewport {
    /// First visible candle index
    pub first_visible: usize,
    /// Last visible candle index
    pub last_visible: usize,
    /// Minimum price in view
    pub price_min: f64,
    /// Maximum price in view
    pub price_max: f64,
    /// Canvas width in pixels
    pub width: u32,
    /// Canvas height in pixels
    pub height: u32,
}

/// Implement this trait to draw custom overlays on the chart.
pub trait RendererPlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn render(&self, viewport: &Viewport, candles: &[Candle]) -> Vec<RenderCommand>;
}

// ---------------------------------------------------------------------------
// Data source plugin trait
// ---------------------------------------------------------------------------

/// A candle event from a data source plugin.
pub struct CandleEvent {
    pub symbol: String,
    pub candle: Candle,
    pub is_final: bool,
}

/// Implement this trait to create a custom data feed.
///
/// Note: async streaming requires the host to poll `next_candle` periodically.
pub trait DataSourcePlugin: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn supported_symbols(&self) -> Vec<String>;
    fn subscribe(&mut self, symbol: &str, timeframe_seconds: u32) -> Result<(), String>;
    fn unsubscribe(&mut self, symbol: &str);
    fn next_candle(&mut self) -> Option<CandleEvent>;
}
