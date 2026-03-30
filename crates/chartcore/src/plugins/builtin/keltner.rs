// Keltner Channels Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct KeltnerIndicator;

impl IndicatorPlugin for KeltnerIndicator {
    fn id(&self) -> &str {
        "keltner"
    }
    fn name(&self) -> &str {
        "Keltner Channels"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::ChannelsAndBands
    }
    fn description(&self) -> &str {
        "Volatility envelope using ATR"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("length", "EMA Length", 20).min(1).max(200),
            InputConfig::int("atr_length", "ATR Length", 10)
                .min(1)
                .max(100),
            InputConfig::float("mult", "Multiplier", 2.0),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("upper", "Upper", "#2962FF").line_width(1),
            PlotConfig::new("basis", "Basis", "#2962FF").line_width(1),
            PlotConfig::new("lower", "Lower", "#2962FF").line_width(1),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let length = context.input_int("length").unwrap_or(20) as usize;
        let atr_length = context.input_int("atr_length").unwrap_or(10) as usize;
        let mult = context.input_float("mult").unwrap_or(2.0);

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();
        let close: Vec<f64> = context.candles.iter().map(|c| c.c).collect();

        let (basis, upper, lower) =
            ta::keltner_channels(&high, &low, &close, length, atr_length, mult, true);

        IndicatorResult::new(&format!("KC ({}, {})", length, mult), "KC", true)
            .add_plot("upper", upper)
            .add_plot("basis", basis)
            .add_plot("lower", lower)
    }
}
