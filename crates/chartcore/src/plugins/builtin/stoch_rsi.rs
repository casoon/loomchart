// Stochastic RSI Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct StochRSIIndicator;

impl IndicatorPlugin for StochRSIIndicator {
    fn id(&self) -> &str {
        "stoch_rsi"
    }
    fn name(&self) -> &str {
        "Stochastic RSI"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Oscillators
    }
    fn description(&self) -> &str {
        "Stochastic applied to RSI values"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("rsi_length", "RSI Length", 14)
                .min(1)
                .max(100),
            InputConfig::int("stoch_length", "Stoch Length", 14)
                .min(1)
                .max(100),
            InputConfig::int("k_smooth", "%K Smoothing", 3)
                .min(1)
                .max(10),
            InputConfig::int("d_smooth", "%D Smoothing", 3)
                .min(1)
                .max(10),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("k", "%K", "#2962FF").line_width(2),
            PlotConfig::new("d", "%D", "#FF6D00").line_width(2),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let rsi_length = context.input_int("rsi_length").unwrap_or(14) as usize;
        let stoch_length = context.input_int("stoch_length").unwrap_or(14) as usize;
        let k_smooth = context.input_int("k_smooth").unwrap_or(3) as usize;
        let d_smooth = context.input_int("d_smooth").unwrap_or(3) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let (k, d) = ta::stoch_rsi(&source, rsi_length, stoch_length, k_smooth, d_smooth);

        IndicatorResult::new("Stochastic RSI", "StochRSI", false)
            .add_plot("k", k)
            .add_plot("d", d)
    }
}
