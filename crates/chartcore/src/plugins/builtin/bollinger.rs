// Bollinger Bands Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct BollingerBandsIndicator;

impl IndicatorPlugin for BollingerBandsIndicator {
    fn id(&self) -> &str {
        "bb"
    }
    fn name(&self) -> &str {
        "Bollinger Bands"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::ChannelsAndBands
    }
    fn description(&self) -> &str {
        "Volatility bands using standard deviation"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 20).min(1).max(200),
            InputConfig::float("mult", "StdDev Multiplier", 2.0)
                .tooltip("Standard deviation multiplier"),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("basis", "Basis", "#2962FF").line_width(1),
            PlotConfig::new("upper", "Upper", "#F23645").line_width(1),
            PlotConfig::new("lower", "Lower", "#089981").line_width(1),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(20) as usize;
        let mult = context.input_float("mult").unwrap_or(2.0);
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let (basis, upper, lower) = ta::bollinger_bands(&source, length, mult);

        IndicatorResult::new(&format!("BB ({}, {})", length, mult), "BB", true)
            .add_plot("basis", basis)
            .add_plot("upper", upper)
            .add_plot("lower", lower)
    }
}

#[derive(Default)]
pub struct BBBandwidthIndicator;

impl IndicatorPlugin for BBBandwidthIndicator {
    fn id(&self) -> &str {
        "bb_bandwidth"
    }
    fn name(&self) -> &str {
        "Bollinger Bandwidth"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
    }
    fn description(&self) -> &str {
        "Width of Bollinger Bands as percentage"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 20).min(1).max(200),
            InputConfig::float("mult", "StdDev Multiplier", 2.0),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("bandwidth", "Bandwidth", "#FF9800").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(20) as usize;
        let mult = context.input_float("mult").unwrap_or(2.0);
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let bandwidth = ta::bb_bandwidth(&source, length, mult);

        IndicatorResult::new("BB Bandwidth", "BBW", false).add_plot("bandwidth", bandwidth)
    }
}

#[derive(Default)]
pub struct BBPercentBIndicator;

impl IndicatorPlugin for BBPercentBIndicator {
    fn id(&self) -> &str {
        "bb_percent_b"
    }
    fn name(&self) -> &str {
        "Bollinger %B"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
    }
    fn description(&self) -> &str {
        "Where price is relative to the bands"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "Length", 20).min(1).max(200),
            InputConfig::float("mult", "StdDev Multiplier", 2.0),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("percentb", "%B", "#9C27B0").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(20) as usize;
        let mult = context.input_float("mult").unwrap_or(2.0);
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let percent_b = ta::bb_percent_b(&source, length, mult);

        IndicatorResult::new("BB %B", "%B", false).add_plot("percentb", percent_b)
    }
}

#[derive(Default)]
pub struct BBTrendIndicator;

impl IndicatorPlugin for BBTrendIndicator {
    fn id(&self) -> &str {
        "bbtrend"
    }
    fn name(&self) -> &str {
        "BB Trend"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Volatility
    }
    fn description(&self) -> &str {
        "Compares short and long Bollinger Band widths"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("short", "Short Period", 20)
                .min(1)
                .max(100),
            InputConfig::int("long", "Long Period", 50).min(1).max(200),
            InputConfig::float("mult", "StdDev Multiplier", 2.0),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("bbtrend", "BB Trend", "#E91E63").line_width(2)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let short = context.input_int("short").unwrap_or(20) as usize;
        let long = context.input_int("long").unwrap_or(50) as usize;
        let mult = context.input_float("mult").unwrap_or(2.0);
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let bbtrend = ta::bbtrend(&source, short, long, mult);

        IndicatorResult::new("BB Trend", "BBT", false).add_plot("bbtrend", bbtrend)
    }
}
