// TSI Indicator (True Strength Index)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct TSIIndicator;

impl IndicatorPlugin for TSIIndicator {
    fn id(&self) -> &str {
        "tsi"
    }
    fn name(&self) -> &str {
        "True Strength Index"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }
    fn description(&self) -> &str {
        "Double-smoothed momentum oscillator"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("long", "Long Length", 25).min(1).max(100),
            InputConfig::int("short", "Short Length", 13).min(1).max(50),
            InputConfig::int("signal", "Signal Length", 13)
                .min(1)
                .max(50),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("tsi", "TSI", "#2962FF").line_width(2),
            PlotConfig::new("signal", "Signal", "#FF6D00").line_width(2),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let long = context.input_int("long").unwrap_or(25) as usize;
        let short = context.input_int("short").unwrap_or(13) as usize;
        let signal_len = context.input_int("signal").unwrap_or(13) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let (tsi, signal) = ta::tsi(&source, long, short, signal_len);

        IndicatorResult::new("True Strength Index", "TSI", false)
            .add_plot("tsi", tsi)
            .add_plot("signal", signal)
    }
}
