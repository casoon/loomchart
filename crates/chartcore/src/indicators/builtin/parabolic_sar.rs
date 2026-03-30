// Parabolic SAR Indicator

use crate::plugins::*;
use crate::ta;

#[derive(Default)]
pub struct ParabolicSARIndicator;

impl IndicatorPlugin for ParabolicSARIndicator {
    fn id(&self) -> &str {
        "psar"
    }
    fn name(&self) -> &str {
        "Parabolic SAR"
    }
    fn category(&self) -> IndicatorCategory {
        IndicatorCategory::Trend
    }
    fn description(&self) -> &str {
        "Stop and reverse trailing stop system"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn inputs(&self) -> Vec<InputConfig> {
        vec![
            InputConfig::float("start", "Start AF", 0.02).tooltip("Starting acceleration factor"),
            InputConfig::float("increment", "Increment", 0.02).tooltip("AF increment"),
            InputConfig::float("max", "Maximum AF", 0.2).tooltip("Maximum acceleration factor"),
        ]
    }

    fn plots(&self) -> Vec<PlotConfig> {
        vec![PlotConfig::new("sar", "SAR", "#FF5722").line_width(1)]
    }

    fn calculate(&self, context: &CalculationContext) -> IndicatorResult {
        let start = context.input_float("start").unwrap_or(0.02);
        let increment = context.input_float("increment").unwrap_or(0.02);
        let max = context.input_float("max").unwrap_or(0.2);

        let high: Vec<f64> = context.candles.iter().map(|c| c.h).collect();
        let low: Vec<f64> = context.candles.iter().map(|c| c.l).collect();

        let sar = ta::parabolic_sar(&high, &low, start, max, increment);

        IndicatorResult::new("Parabolic SAR", "PSAR", true).add_plot("sar", sar)
    }
}
