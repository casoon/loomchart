// MACD Indicator (Moving Average Convergence Divergence)

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct MACDIndicator;

impl IndicatorPlugin for MACDIndicator {
    fn id(&self) -> &str {
        "macd"
    }
    fn name(&self) -> &str {
        "MACD"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Momentum
    }
    fn description(&self) -> &str {
        "Trend-following momentum indicator"
    }
    fn overlay(&self) -> bool {
        false
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::int("fast", "Fast Length", 12).min(1).max(100),
            InputConfig::int("slow", "Slow Length", 26).min(1).max(200),
            InputConfig::int("signal", "Signal Length", 9)
                .min(1)
                .max(50),
            InputConfig::source("source", "Source", SourceType::Close),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![
            PlotConfig::new("macd", "MACD", "#2962FF").line_width(2),
            PlotConfig::new("signal", "Signal", "#FF6D00").line_width(2),
            PlotConfig::new("histogram", "Histogram", "#26A69A").line_width(1),
        ]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let fast = context.input_int("fast").unwrap_or(12) as usize;
        let slow = context.input_int("slow").unwrap_or(26) as usize;
        let signal_len = context.input_int("signal").unwrap_or(9) as usize;
        let source_type = context.input_source("source").unwrap_or(SourceType::Close);
        let source = context.source(source_type);

        let (macd_line, signal_line, histogram) = ta::macd(&source, fast, slow, signal_len);

        IndicatorResult::new(
            &format!("MACD ({}, {}, {})", fast, slow, signal_len),
            "MACD",
            false,
        )
        .add_plot("macd", macd_line)
        .add_plot("signal", signal_line)
        .add_plot("histogram", histogram)
    }
}
